//! Positions sub-client — portfolio & position queries, and on-chain position operations.

use crate::auth::AuthCredentials;
use crate::client::LightconeClient;
use crate::domain::position::builders::{
    DepositBuilder, DepositToGlobalBuilder, ExtendPositionTokensBuilder,
    GlobalToMarketDepositBuilder, InitPositionTokensBuilder, MergeBuilder, RedeemWinningsBuilder,
    WithdrawBuilder, WithdrawFromGlobalBuilder, WithdrawFromPositionBuilder,
};
use crate::domain::position::wire::{MarketPositionsResponse, PositionsResponse};
use crate::domain::position::{
    state::{sol_amount_to_lamports, WRAPPED_SOL_MINT_ADDRESS},
    DepositTokenBalancesSnapshot, WalletDepositBalancesState,
};
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::program::instructions;
use crate::program::types::{
    ClosePositionAltParams, ClosePositionTokenAccountsParams, DepositToGlobalAltContext,
    DepositToGlobalParams, ExtendPositionTokensParams, GlobalToMarketDepositParams,
    InitPositionTokensParams, RedeemWinningsParams, WithdrawConditionalFromPositionParams,
    WithdrawFromGlobalParams, WithdrawFromPositionParams,
};
use crate::shared::signing::SigningStrategy;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use std::str::FromStr;

fn deposit_token_balances_query(min_context_slot: Option<u64>) -> Vec<(&'static str, String)> {
    min_context_slot
        .map(|slot| vec![("min_context_slot", slot.to_string())])
        .unwrap_or_default()
}

fn wrapped_sol_accounts(wallet: &Pubkey) -> Result<(Pubkey, Pubkey), SdkError> {
    // State preflight and transaction construction deliberately share the
    // canonical Tokenkeg mint so conversion cannot target another token account.
    let mint = Pubkey::from_str(WRAPPED_SOL_MINT_ADDRESS)
        .map_err(|error| SdkError::Validation(format!("invalid wrapped SOL mint: {error}")))?;
    let token_program = spl_token_interface::id();
    let account = spl_associated_token_account_interface::address::get_associated_token_address_with_program_id(
        wallet,
        &mint,
        &token_program,
    );
    Ok((mint, account))
}

fn build_wrap_sol_transaction(wallet: Pubkey, lamports: u64) -> Result<Transaction, SdkError> {
    let token_program = spl_token_interface::id();
    let (mint, account) = wrapped_sol_accounts(&wallet)?;
    let create = spl_associated_token_account_interface::instruction::create_associated_token_account_idempotent(
        &wallet,
        &wallet,
        &mint,
        &token_program,
    );
    let transfer = solana_system_interface::instruction::transfer(&wallet, &account, lamports);
    let sync = spl_token_interface::instruction::sync_native(&token_program, &account)
        .map_err(|error| SdkError::Other(format!("failed to build SyncNative: {error}")))?;
    // Ordering is load-bearing: create or reuse the ATA, transfer exact
    // lamports, then make the token program observe them through SyncNative.
    Ok(Transaction::new_with_payer(
        &[create, transfer, sync],
        Some(&wallet),
    ))
}

fn build_unwrap_wsol_transaction(wallet: Pubkey) -> Result<Transaction, SdkError> {
    let token_program = spl_token_interface::id();
    let (_, account) = wrapped_sol_accounts(&wallet)?;
    // CloseAccount is the full-unwrap boundary: all token lamports plus account
    // rent return to the wallet, so this transaction has no amount parameter.
    let close = spl_token_interface::instruction::close_account(
        &token_program,
        &account,
        &wallet,
        &wallet,
        &[],
    )
    .map_err(|error| SdkError::Other(format!("failed to build CloseAccount: {error}")))?;
    Ok(Transaction::new_with_payer(&[close], Some(&wallet)))
}

fn validated_conversion_wallet(
    credentials: Option<&AuthCredentials>,
    state: &WalletDepositBalancesState,
) -> Result<Pubkey, SdkError> {
    // Cached identity is a signing trust boundary, not just display metadata.
    // Require an unexpired session and a complete baseline for the same wallet.
    let credentials = credentials
        .ok_or_else(|| SdkError::Validation("authenticated credentials are required".into()))?;
    if !credentials.is_authenticated() {
        return Err(SdkError::Validation(
            "authenticated credentials have expired".into(),
        ));
    }
    let state_wallet = state
        .wallet_address
        .as_ref()
        .ok_or_else(|| SdkError::Validation("wallet balance state is not initialized".into()))?;
    if state_wallet != &credentials.wallet_address {
        return Err(SdkError::Validation(
            "authenticated wallet does not match wallet balance state".into(),
        ));
    }
    if state.context_slot.is_none() || state.native_sol_balance.is_none() {
        return Err(SdkError::Validation(
            "wallet balance state is not initialized".into(),
        ));
    }
    credentials
        .wallet_address
        .to_pubkey()
        .map_err(SdkError::Validation)
}

pub struct Positions<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Positions<'a> {
    // ── PDA helpers ──────────────────────────────────────────────────────

    /// Get the Position PDA.
    pub fn pda(&self, owner: &Pubkey, market: &Pubkey) -> Pubkey {
        crate::program::pda::get_position_pda(owner, market, &self.client.program_id).0
    }

    // ── HTTP methods ─────────────────────────────────────────────────────

    /// Get all positions for a user across all markets.
    pub async fn get(&self, user_pubkey: &str) -> Result<PositionsResponse, SdkError> {
        let url = format!(
            "{}/api/users/{}/positions",
            self.client.http.base_url(),
            user_pubkey
        );
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Get positions for a user in a specific market.
    pub async fn get_for_market(
        &self,
        user_pubkey: &str,
        market_pubkey: &str,
    ) -> Result<MarketPositionsResponse, SdkError> {
        let url = format!(
            "{}/api/users/{}/markets/{}/positions",
            self.client.http.base_url(),
            user_pubkey,
            market_pubkey
        );
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Get all conditional-token positions for the authenticated user across
    /// every market. The wallet is resolved server-side from the auth cookie,
    /// so no parameter is required. Same response shape as
    /// [`Positions::get`]; empty `positions` array when the user has none.
    pub async fn positions(&self) -> Result<PositionsResponse, SdkError> {
        let url = format!("{}/api/users/positions", self.client.http.base_url());
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Same as [`Self::positions`], but forwards the supplied raw `Cookie` header (`privy-token` and/or `lightcone-token`) for
    /// this call instead of the SDK's process-wide token store.
    ///
    /// Intended for server-side cookie forwarding (SSR / server functions)
    /// where the per-request browser cookie can't propagate to the shared
    /// client. On WASM this is equivalent to [`Self::positions`] because the
    /// browser is already attaching the cookie via credentials mode.
    pub async fn positions_with_cookies(
        &self,
        cookie_header: &str,
    ) -> Result<PositionsResponse, SdkError> {
        let url = format!("{}/api/users/positions", self.client.http.base_url());
        self.client
            .http
            .get_with_cookies(&url, RetryPolicy::Idempotent, cookie_header)
            .await
    }

    /// Get the authenticated user's positions in a specific market. The
    /// wallet is resolved server-side from the auth cookie.
    pub async fn positions_for_market(
        &self,
        market_pubkey: &str,
    ) -> Result<MarketPositionsResponse, SdkError> {
        let url = format!(
            "{}/api/users/markets/{}/positions",
            self.client.http.base_url(),
            market_pubkey
        );
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Same as [`Self::positions_for_market`], but forwards the supplied raw
    /// `Cookie` header (`privy-token` and/or `lightcone-token`) for this call
    /// instead of the SDK's process-wide token store. For server-side cookie
    /// forwarding (SSR / server functions).
    pub async fn positions_for_market_with_cookies(
        &self,
        market_pubkey: &str,
        cookie_header: &str,
    ) -> Result<MarketPositionsResponse, SdkError> {
        let url = format!(
            "{}/api/users/markets/{}/positions",
            self.client.http.base_url(),
            market_pubkey
        );
        self.client
            .http
            .get_with_cookies(&url, RetryPolicy::Idempotent, cookie_header)
            .await
    }

    /// Fetch a complete authenticated SPL and native-SOL balance snapshot.
    ///
    /// `min_context_slot` lower-bounds the complete cross-component snapshot.
    /// Native SOL is required canonical nine-decimal text and remains separate
    /// from `balances`; apply the result through [`WalletDepositBalancesState`]
    /// when combining it with WebSocket updates.
    pub async fn deposit_token_balances(
        &self,
        min_context_slot: Option<u64>,
    ) -> Result<DepositTokenBalancesSnapshot, SdkError> {
        let url = format!(
            "{}/api/users/deposit-token-balances",
            self.client.http.base_url()
        );
        let query = deposit_token_balances_query(min_context_slot);
        self.client
            .http
            .get_with_query(&url, &query, RetryPolicy::Idempotent)
            .await
    }

    /// Same as [`Self::deposit_token_balances`], but forwards the supplied raw
    /// `Cookie` header (`privy-token` and/or `lightcone-token`) for this call
    /// instead of the SDK's process-wide token store.
    ///
    /// Intended for server-side cookie forwarding (SSR / server functions)
    /// where the per-request browser cookies can't propagate to the shared
    /// client. On WASM this is equivalent to
    /// [`Self::deposit_token_balances`] because the browser is already
    /// attaching cookies via credentials mode.
    pub async fn deposit_token_balances_with_cookies(
        &self,
        min_context_slot: Option<u64>,
        cookie_header: &str,
    ) -> Result<DepositTokenBalancesSnapshot, SdkError> {
        let url = format!(
            "{}/api/users/deposit-token-balances",
            self.client.http.base_url()
        );
        let query = deposit_token_balances_query(min_context_slot);
        self.client
            .http
            .get_with_cookies_and_query(&url, &query, RetryPolicy::Idempotent, cookie_header)
            .await
    }

    /// Wrap exact SOL into the authenticated wallet's canonical Tokenkeg WSOL ATA.
    ///
    /// The amount must be positive, exactly representable at nine decimals, fit
    /// Solana's `u64` lamport range, and not exceed cached native SOL. Credentials
    /// must be live and match an initialized state, and the configured signing
    /// strategy must control that wallet. The transaction creates the ATA
    /// idempotently, transfers, syncs, and returns its confirmed signature. Fee
    /// and rent reserves are left to chain execution, and state is not mutated.
    /// An error while confirming does not prove submission was rolled back;
    /// refresh from REST or WebSocket authority before retrying.
    pub async fn wrap_sol(
        &self,
        amount: &str,
        state: &WalletDepositBalancesState,
    ) -> Result<String, SdkError> {
        let (wallet, strategy) = self.conversion_wallet(state).await?;
        let lamports = sol_amount_to_lamports(amount)?;
        if lamports == 0 {
            return Err(SdkError::Validation(
                "wrap amount must be greater than zero".into(),
            ));
        }
        // Do not guess a fee or ATA-rent reserve from stale client state; an
        // equal-balance wrap is valid preflight and the chain remains authoritative.
        if lamports > state.native_sol_lamports()? {
            return Err(SdkError::Validation(
                "wrap amount exceeds cached native SOL balance".into(),
            ));
        }

        self.client
            .sign_and_submit_tx_confirmed_with_strategy(
                build_wrap_sol_transaction(wallet, lamports)?,
                strategy,
            )
            .await
    }

    /// Fully unwrap the authenticated wallet's canonical Tokenkeg WSOL ATA.
    ///
    /// Matching live credentials, a signing strategy controlling that wallet,
    /// and a positive cached canonical balance are required. Closing the account
    /// credits its entire token balance plus rent to the wallet; partial unwrap is
    /// intentionally unsupported. The returned string is the confirmed signature,
    /// and cached state remains unchanged. A confirmation error does not prove the
    /// account stayed open; refresh authoritative state before retrying.
    pub async fn unwrap_wsol(
        &self,
        state: &WalletDepositBalancesState,
    ) -> Result<String, SdkError> {
        let (wallet, strategy) = self.conversion_wallet(state).await?;
        if !state.has_positive_wsol()? {
            return Err(SdkError::Validation(
                "canonical WSOL balance must be greater than zero".into(),
            ));
        }

        self.client
            .sign_and_submit_tx_confirmed_with_strategy(
                build_unwrap_wsol_transaction(wallet)?,
                strategy,
            )
            .await
    }

    async fn conversion_wallet(
        &self,
        state: &WalletDepositBalancesState,
    ) -> Result<(Pubkey, SigningStrategy), SdkError> {
        // Keep all conversion entry points behind the same credential/state
        // identity check before constructing a wallet-authorized transaction.
        let credentials = self.client.auth().credentials().await;
        let wallet = validated_conversion_wallet(credentials.as_ref(), state)?;
        let strategy = self.client.signing_strategy().await.ok_or_else(|| {
            SdkError::Validation("signing strategy is not set on the client".into())
        })?;
        let signing_wallet = strategy.wallet_address().ok_or_else(|| {
            SdkError::Validation("signing strategy wallet identity is required".into())
        })?;
        if signing_wallet != wallet {
            return Err(SdkError::Validation(
                "signing strategy does not control authenticated wallet".into(),
            ));
        }
        Ok((wallet, strategy))
    }

    // ── On-chain instruction builders ───────────────────────────────────

    /// Build RedeemWinnings instruction.
    pub fn redeem_winnings_ix(
        &self,
        params: &RedeemWinningsParams,
        outcome_index: u8,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_redeem_winnings_ix(params, outcome_index, pid)
    }

    /// Build RedeemWinnings transaction.
    pub fn redeem_winnings_tx(
        &self,
        params: RedeemWinningsParams,
        outcome_index: u8,
    ) -> Result<Transaction, SdkError> {
        let ix = self.redeem_winnings_ix(&params, outcome_index);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build a conditional-token withdrawal from a position instruction.
    pub fn withdraw_conditional_from_position_ix(
        &self,
        params: &WithdrawConditionalFromPositionParams,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_withdraw_conditional_from_position_ix(params, pid)
    }

    /// Build a conditional-token withdrawal from a position transaction.
    pub fn withdraw_conditional_from_position_tx(
        &self,
        params: WithdrawConditionalFromPositionParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.withdraw_conditional_from_position_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build a conditional-token withdrawal from a position instruction.
    ///
    /// Compatibility wrapper for the previous SDK method name.
    pub fn withdraw_from_position_ix(&self, params: &WithdrawFromPositionParams) -> Instruction {
        self.withdraw_conditional_from_position_ix(params)
    }

    /// Build a conditional-token withdrawal from a position transaction.
    ///
    /// Compatibility wrapper for the previous SDK method name.
    pub fn withdraw_from_position_tx(
        &self,
        params: WithdrawFromPositionParams,
    ) -> Result<Transaction, SdkError> {
        self.withdraw_conditional_from_position_tx(params)
    }

    /// Build InitPositionTokens instruction.
    pub fn init_position_tokens_ix(
        &self,
        params: &InitPositionTokensParams,
        num_outcomes: u8,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_init_position_tokens_ix(params, num_outcomes, pid)
    }

    /// Build InitPositionTokens transaction.
    pub fn init_position_tokens_tx(
        &self,
        params: InitPositionTokensParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let ix = self.init_position_tokens_ix(&params, num_outcomes);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.payer)))
    }

    /// Build ExtendPositionTokens instruction.
    pub fn extend_position_tokens_ix(
        &self,
        params: &ExtendPositionTokensParams,
        num_outcomes: u8,
    ) -> Result<Instruction, SdkError> {
        let pid = &self.client.program_id;
        Ok(instructions::build_extend_position_tokens_ix(
            params,
            num_outcomes,
            pid,
        )?)
    }

    /// Build ExtendPositionTokens transaction.
    pub fn extend_position_tokens_tx(
        &self,
        params: ExtendPositionTokensParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let ix = self.extend_position_tokens_ix(&params, num_outcomes)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }

    /// Build ClosePositionAlt instruction.
    pub fn close_position_alt_ix(&self, params: &ClosePositionAltParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_close_position_alt_ix(params, pid)
    }

    /// Build ClosePositionAlt transaction.
    pub fn close_position_alt_tx(
        &self,
        params: ClosePositionAltParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.close_position_alt_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }

    /// Build ClosePositionTokenAccounts instruction.
    pub fn close_position_token_accounts_ix(
        &self,
        params: &ClosePositionTokenAccountsParams,
        num_outcomes: u8,
    ) -> Result<Instruction, SdkError> {
        let pid = &self.client.program_id;
        Ok(instructions::build_close_position_token_accounts_ix(
            params,
            num_outcomes,
            pid,
        )?)
    }

    /// Build ClosePositionTokenAccounts transaction.
    pub fn close_position_token_accounts_tx(
        &self,
        params: ClosePositionTokenAccountsParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let ix = self.close_position_token_accounts_ix(&params, num_outcomes)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }

    /// Build DepositToGlobal instruction.
    pub fn deposit_to_global_ix(&self, params: &DepositToGlobalParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_deposit_to_global_ix(params, pid)
    }

    /// Build DepositToGlobal instruction with user deposit ALT create/extend accounts.
    pub fn deposit_to_global_ix_with_alt(
        &self,
        params: &DepositToGlobalParams,
        alt_context: DepositToGlobalAltContext,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_deposit_to_global_ix_with_alt(params, alt_context, pid)
    }

    /// Build DepositToGlobal transaction.
    pub fn deposit_to_global_tx(
        &self,
        params: DepositToGlobalParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.deposit_to_global_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build DepositToGlobal transaction with user deposit ALT create/extend accounts.
    pub fn deposit_to_global_tx_with_alt(
        &self,
        params: DepositToGlobalParams,
        alt_context: DepositToGlobalAltContext,
    ) -> Result<Transaction, SdkError> {
        let ix = self.deposit_to_global_ix_with_alt(&params, alt_context);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build GlobalToMarketDeposit instruction.
    pub fn global_to_market_deposit_ix(
        &self,
        params: &GlobalToMarketDepositParams,
        num_outcomes: u8,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_global_to_market_deposit_ix(params, num_outcomes, pid)
    }

    /// Build GlobalToMarketDeposit transaction.
    pub fn global_to_market_deposit_tx(
        &self,
        params: GlobalToMarketDepositParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let ix = self.global_to_market_deposit_ix(&params, num_outcomes);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build WithdrawFromGlobal instruction.
    pub fn withdraw_from_global_ix(&self, params: &WithdrawFromGlobalParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_withdraw_from_global_ix(params, pid)
    }

    /// Build WithdrawFromGlobal transaction.
    pub fn withdraw_from_global_tx(
        &self,
        params: WithdrawFromGlobalParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.withdraw_from_global_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    // ── Builder factories ──────────────────────────────────────────────

    /// Create a deposit builder pre-seeded with the client's deposit source.
    ///
    /// Use `.build_ix()` or `.build_tx()` to produce the final instruction/transaction.
    pub async fn deposit(&self) -> DepositBuilder<'a> {
        let deposit_source = self.client.deposit_source().await;
        DepositBuilder::new(self.client, deposit_source)
    }

    /// Create a merge builder.
    ///
    /// Burns a complete set of conditional tokens and releases collateral.
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn merge(&self) -> MergeBuilder<'a> {
        MergeBuilder::new(self.client)
    }

    /// Create a withdraw builder pre-seeded with the client's deposit source.
    ///
    /// Dispatches based on deposit source:
    /// - **Global**: withdraws from global deposit pool
    /// - **Market**: withdraws conditional tokens from a position ATA
    ///
    /// Use `.build_ix()` or `.build_tx()` to produce the final instruction/transaction.
    pub async fn withdraw(&self) -> WithdrawBuilder<'a> {
        let deposit_source = self.client.deposit_source().await;
        WithdrawBuilder::new(self.client, deposit_source)
    }

    /// Create a redeem winnings builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn redeem_winnings(&self) -> RedeemWinningsBuilder<'a> {
        RedeemWinningsBuilder::new(self.client)
    }

    /// Create a conditional-token withdraw-from-position builder.
    /// Set `.num_outcomes(...)` before building because this path only receives a market pubkey.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn withdraw_from_position(&self) -> WithdrawFromPositionBuilder<'a> {
        WithdrawFromPositionBuilder::new(self.client)
    }

    /// Create a conditional-token withdraw-from-position builder.
    pub fn withdraw_conditional_from_position(&self) -> WithdrawFromPositionBuilder<'a> {
        WithdrawFromPositionBuilder::new(self.client)
    }

    /// Create an init-position-tokens builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn init_position_tokens(&self) -> InitPositionTokensBuilder<'a> {
        InitPositionTokensBuilder::new(self.client)
    }

    /// Create an extend-position-tokens builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn extend_position_tokens(&self) -> ExtendPositionTokensBuilder<'a> {
        ExtendPositionTokensBuilder::new(self.client)
    }

    /// Create a deposit-to-global builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn deposit_to_global(&self) -> DepositToGlobalBuilder<'a> {
        DepositToGlobalBuilder::new(self.client)
    }

    /// Create a withdraw-from-global builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn withdraw_from_global(&self) -> WithdrawFromGlobalBuilder<'a> {
        WithdrawFromGlobalBuilder::new(self.client)
    }

    /// Create a global-to-market deposit builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn global_to_market_deposit(&self) -> GlobalToMarketDepositBuilder<'a> {
        GlobalToMarketDepositBuilder::new(self.client)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// On-chain account fetchers (require RPC)
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "solana-rpc")]
impl<'a> Positions<'a> {
    /// Fetch a Position account (returns None if not found).
    pub async fn get_onchain(
        &self,
        owner: &Pubkey,
        market: &Pubkey,
    ) -> Result<Option<crate::program::accounts::Position>, SdkError> {
        let rpc = crate::rpc::resolve_solana_rpc(self.client).await?;
        let pda = self.pda(owner, market);
        match rpc.get_account(&pda).await {
            Ok(account) => Ok(Some(crate::program::accounts::Position::deserialize(
                &account.data,
            )?)),
            Err(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_unwrap_wsol_transaction, build_wrap_sol_transaction, deposit_token_balances_query,
        validated_conversion_wallet, wrapped_sol_accounts, WRAPPED_SOL_MINT_ADDRESS,
    };
    use crate::{auth::AuthCredentials, domain::position::WalletDepositBalancesState};
    use chrono::{Duration, Utc};
    use solana_pubkey::Pubkey;
    use solana_system_interface::instruction::SystemInstruction;
    use spl_token_interface::instruction::TokenInstruction;

    #[cfg(feature = "native-auth")]
    use {
        crate::client::LightconeClient,
        crate::domain::position::DepositTokenBalance,
        crate::error::SdkError,
        crate::shared::signing::{ExternalSigner, SigningStrategy},
        crate::shared::PubkeyStr,
        async_lock::RwLock,
        rust_decimal::Decimal,
        solana_keypair::Keypair,
        solana_signer::Signer,
        solana_transaction::Transaction,
        std::{
            collections::VecDeque,
            future::Future,
            pin::Pin,
            sync::{
                atomic::{AtomicUsize, Ordering},
                Arc, Mutex, OnceLock,
            },
        },
        tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpListener,
        },
    };

    #[cfg(feature = "native-auth")]
    async fn spawn_rpc_server(
        responses: Vec<serde_json::Value>,
    ) -> (String, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let responses = Arc::new(Mutex::new(VecDeque::from(responses)));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let server_responses = Arc::clone(&responses);
        let server_requests = Arc::clone(&requests);

        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    return;
                };
                let mut buffer = [0_u8; 16_384];
                let Ok(bytes_read) = socket.read(&mut buffer).await else {
                    return;
                };
                let request = String::from_utf8_lossy(&buffer[..bytes_read]).into_owned();
                server_requests.lock().unwrap().push(request);
                let body = server_responses
                    .lock()
                    .unwrap()
                    .pop_front()
                    .unwrap_or_else(|| {
                        serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": 1,
                            "error": {"code": -32000, "message": "unexpected request"}
                        })
                    })
                    .to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(response.as_bytes()).await;
            }
        });

        (format!("http://{address}"), requests)
    }

    #[cfg(feature = "native-auth")]
    fn conversion_client(rpc_url: &str) -> (LightconeClient, WalletDepositBalancesState) {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let wallet_address = PubkeyStr::from(wallet.to_string());
        let credentials = AuthCredentials {
            user_id: "user-a".into(),
            wallet_address: wallet_address.clone(),
            expires_at: Utc::now() + Duration::minutes(1),
        };
        let client = LightconeClient::builder()
            .auth(credentials)
            .native_signer(keypair)
            .rpc_url(rpc_url)
            .build()
            .unwrap();
        let state = WalletDepositBalancesState {
            wallet_address: Some(wallet_address),
            context_slot: Some(1),
            native_sol_balance: Some("2.000000000".into()),
            ..Default::default()
        };
        (client, state)
    }

    #[cfg(feature = "native-auth")]
    struct TestTransactionSigner {
        keypair: Arc<Keypair>,
        expose_identity: bool,
        transaction_calls: Arc<AtomicUsize>,
        strategy_swap: Option<(
            Arc<OnceLock<Arc<RwLock<Option<SigningStrategy>>>>>,
            SigningStrategy,
        )>,
    }

    #[cfg(feature = "native-auth")]
    impl ExternalSigner for TestTransactionSigner {
        fn wallet_address(&self) -> Option<Pubkey> {
            if let Some((target, replacement)) = &self.strategy_swap {
                let mut configured = target
                    .get()
                    .expect("strategy swap target must be initialized")
                    .try_write()
                    .expect("conversion must release its strategy read lock");
                *configured = Some(replacement.clone());
            }
            self.expose_identity.then(|| self.keypair.pubkey())
        }

        fn sign_message<'a>(
            &'a self,
            message: &'a [u8],
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + 'a>> {
            Box::pin(async move { Ok(message.to_vec()) })
        }

        fn sign_transaction<'a>(
            &'a self,
            tx_bytes: &'a [u8],
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + 'a>> {
            Box::pin(async move {
                let mut transaction: Transaction =
                    bincode::deserialize(tx_bytes).map_err(|error| error.to_string())?;
                let blockhash = transaction.message.recent_blockhash;
                transaction
                    .try_sign(&[self.keypair.as_ref()], blockhash)
                    .map_err(|error| error.to_string())?;
                self.transaction_calls.fetch_add(1, Ordering::SeqCst);
                bincode::serialize(&transaction).map_err(|error| error.to_string())
            })
        }
    }

    #[cfg(feature = "native-auth")]
    fn external_conversion_client(
        rpc_url: &str,
        expose_identity: bool,
    ) -> (
        LightconeClient,
        WalletDepositBalancesState,
        Arc<AtomicUsize>,
    ) {
        let keypair = Arc::new(Keypair::new());
        let wallet_address = PubkeyStr::from(keypair.pubkey().to_string());
        let credentials = AuthCredentials {
            user_id: "user-a".into(),
            wallet_address: wallet_address.clone(),
            expires_at: Utc::now() + Duration::minutes(1),
        };
        let transaction_calls = Arc::new(AtomicUsize::new(0));
        let signer = Arc::new(TestTransactionSigner {
            keypair,
            expose_identity,
            transaction_calls: Arc::clone(&transaction_calls),
            strategy_swap: None,
        });
        let client = LightconeClient::builder()
            .auth(credentials)
            .external_signer(signer)
            .rpc_url(rpc_url)
            .build()
            .unwrap();
        let state = WalletDepositBalancesState {
            wallet_address: Some(wallet_address),
            context_slot: Some(1),
            native_sol_balance: Some("2.000000000".into()),
            ..Default::default()
        };
        (client, state, transaction_calls)
    }

    #[cfg(feature = "native-auth")]
    fn set_wsol_balance(state: &mut WalletDepositBalancesState, idle: &str) {
        let mint = PubkeyStr::from(WRAPPED_SOL_MINT_ADDRESS);
        state.balances.insert(
            mint.clone(),
            DepositTokenBalance {
                mint,
                idle: idle.parse::<Decimal>().unwrap(),
                symbol: "WSOL".into(),
                name: "Wrapped SOL".into(),
                icon_url_low: None,
                icon_url_medium: None,
                icon_url_high: None,
            },
        );
    }

    #[test]
    fn deposit_token_balances_query_includes_optional_minimum_context_slot() {
        assert!(deposit_token_balances_query(None).is_empty());
        assert_eq!(
            deposit_token_balances_query(Some(1234)),
            vec![("min_context_slot", "1234".to_string())]
        );
    }

    #[test]
    fn wrap_transaction_uses_maintained_create_transfer_sync_builders() {
        let wallet = Pubkey::new_unique();
        let (_, account) = wrapped_sol_accounts(&wallet).unwrap();
        let transaction = build_wrap_sol_transaction(wallet, 123).unwrap();
        let instructions = &transaction.message.instructions;

        assert_eq!(instructions.len(), 3);
        assert_eq!(transaction.message.account_keys[0], wallet);
        assert_eq!(instructions[0].data, vec![1]);
        assert_eq!(
            transaction.message.account_keys[instructions[0].program_id_index as usize],
            spl_associated_token_account_interface::program::id()
        );
        assert_eq!(
            bincode::deserialize::<SystemInstruction>(&instructions[1].data).unwrap(),
            SystemInstruction::Transfer { lamports: 123 }
        );
        assert_eq!(
            transaction.message.account_keys[instructions[1].accounts[0] as usize],
            wallet
        );
        assert_eq!(
            transaction.message.account_keys[instructions[1].accounts[1] as usize],
            account
        );
        assert!(matches!(
            TokenInstruction::unpack(&instructions[2].data).unwrap(),
            TokenInstruction::SyncNative
        ));
        assert_eq!(
            transaction.message.account_keys[instructions[2].accounts[0] as usize],
            account
        );
    }

    #[test]
    fn unwrap_transaction_closes_only_the_canonical_wsol_account() {
        let wallet = Pubkey::new_unique();
        let (_, account) = wrapped_sol_accounts(&wallet).unwrap();
        let transaction = build_unwrap_wsol_transaction(wallet).unwrap();
        let instructions = &transaction.message.instructions;

        assert_eq!(instructions.len(), 1);
        assert!(matches!(
            TokenInstruction::unpack(&instructions[0].data).unwrap(),
            TokenInstruction::CloseAccount
        ));
        assert_eq!(transaction.message.account_keys[0], wallet);
        assert_eq!(
            transaction.message.account_keys[instructions[0].accounts[0] as usize],
            account
        );
        assert_eq!(
            transaction.message.account_keys[instructions[0].accounts[1] as usize],
            wallet
        );
        assert_eq!(
            transaction.message.account_keys[instructions[0].accounts[2] as usize],
            wallet
        );
    }

    #[test]
    fn conversion_wallet_requires_unexpired_matching_initialized_state() {
        let wallet = Pubkey::new_unique();
        let wallet_address = crate::shared::PubkeyStr::from(wallet.to_string());
        let credentials = AuthCredentials {
            user_id: "user-a".into(),
            wallet_address: wallet_address.clone(),
            expires_at: Utc::now() + Duration::minutes(1),
        };
        let state = WalletDepositBalancesState {
            wallet_address: Some(wallet_address),
            context_slot: Some(1),
            native_sol_balance: Some("1.000000000".into()),
            ..Default::default()
        };

        assert_eq!(
            validated_conversion_wallet(Some(&credentials), &state).unwrap(),
            wallet
        );
        assert!(validated_conversion_wallet(None, &state).is_err());

        let mut expired = credentials.clone();
        expired.expires_at = Utc::now() - Duration::minutes(1);
        assert!(validated_conversion_wallet(Some(&expired), &state).is_err());

        let mut mismatched = credentials.clone();
        mismatched.wallet_address = Pubkey::new_unique().to_string().into();
        assert!(validated_conversion_wallet(Some(&mismatched), &state).is_err());
        assert!(validated_conversion_wallet(
            Some(&credentials),
            &WalletDepositBalancesState::default()
        )
        .is_err());
    }

    #[cfg(feature = "native-auth")]
    #[tokio::test]
    async fn wrap_uses_confirmed_submission_and_preserves_cached_state() {
        let blockhash = solana_hash::Hash::default().to_string();
        let (rpc_url, requests) = spawn_rpc_server(vec![
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "context": {"slot": 1},
                    "value": {"blockhash": blockhash, "lastValidBlockHeight": 100}
                }
            }),
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": "confirmed-signature"}),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "context": {"slot": 42},
                    "value": [{
                        "slot": 42,
                        "confirmations": 1,
                        "err": null,
                        "confirmationStatus": "confirmed",
                        "status": {"Ok": null}
                    }]
                }
            }),
        ])
        .await;
        let (client, state) = conversion_client(&rpc_url);
        let before = state.clone();

        let signature = client.positions().wrap_sol("0.250000001", &state).await;

        assert_eq!(signature.unwrap(), "confirmed-signature");
        assert_eq!(state, before);
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 3);
        assert!(requests[0].contains("getLatestBlockhash"));
        assert!(requests[1].contains("sendTransaction"));
        assert!(requests[2].contains("getSignatureStatuses"));
    }

    #[cfg(feature = "native-auth")]
    #[tokio::test]
    async fn conversion_rejects_a_mismatched_native_signer_before_rpc() {
        let (client, state) = conversion_client("http://127.0.0.1:1");
        client
            .set_signing_strategy(SigningStrategy::Native(Arc::new(Keypair::new())))
            .await;

        let error = client
            .positions()
            .wrap_sol("0.1", &state)
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            SdkError::Validation(message)
                if message == "signing strategy does not control authenticated wallet"
        ));
    }

    #[cfg(feature = "native-auth")]
    #[tokio::test]
    async fn conversion_rejects_an_external_signer_without_wallet_identity() {
        let (client, state, transaction_calls) =
            external_conversion_client("http://127.0.0.1:1", false);

        let error = client
            .positions()
            .wrap_sol("0.1", &state)
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            SdkError::Validation(message)
                if message == "signing strategy wallet identity is required"
        ));
        assert_eq!(transaction_calls.load(Ordering::SeqCst), 0);
    }

    #[cfg(feature = "native-auth")]
    #[tokio::test]
    async fn conversion_accepts_an_external_signer_with_matching_wallet_identity() {
        let blockhash = solana_hash::Hash::default().to_string();
        let (rpc_url, _) = spawn_rpc_server(vec![
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "context": {"slot": 1},
                    "value": {"blockhash": blockhash, "lastValidBlockHeight": 100}
                }
            }),
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": "confirmed-signature"}),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "context": {"slot": 42},
                    "value": [{
                        "slot": 42,
                        "confirmations": 1,
                        "err": null,
                        "confirmationStatus": "confirmed",
                        "status": {"Ok": null}
                    }]
                }
            }),
        ])
        .await;
        let (client, state, transaction_calls) = external_conversion_client(&rpc_url, true);

        assert_eq!(
            client.positions().wrap_sol("0.1", &state).await.unwrap(),
            "confirmed-signature"
        );
        assert_eq!(transaction_calls.load(Ordering::SeqCst), 1);
    }

    #[cfg(feature = "native-auth")]
    #[tokio::test]
    async fn conversion_submission_uses_the_strategy_validated_before_a_swap() {
        let blockhash = solana_hash::Hash::default().to_string();
        let (rpc_url, _) = spawn_rpc_server(vec![
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "context": {"slot": 1},
                    "value": {"blockhash": blockhash, "lastValidBlockHeight": 100}
                }
            }),
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": "confirmed-signature"}),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "context": {"slot": 42},
                    "value": [{
                        "slot": 42,
                        "confirmations": 1,
                        "err": null,
                        "confirmationStatus": "confirmed",
                        "status": {"Ok": null}
                    }]
                }
            }),
        ])
        .await;
        let keypair = Arc::new(Keypair::new());
        let wallet_address = PubkeyStr::from(keypair.pubkey().to_string());
        let credentials = AuthCredentials {
            user_id: "user-a".into(),
            wallet_address: wallet_address.clone(),
            expires_at: Utc::now() + Duration::minutes(1),
        };
        let transaction_calls = Arc::new(AtomicUsize::new(0));
        let swap_target = Arc::new(OnceLock::new());
        let signer = Arc::new(TestTransactionSigner {
            keypair,
            expose_identity: true,
            transaction_calls: Arc::clone(&transaction_calls),
            strategy_swap: Some((
                Arc::clone(&swap_target),
                SigningStrategy::Native(Arc::new(Keypair::new())),
            )),
        });
        let client = LightconeClient::builder()
            .auth(credentials)
            .external_signer(signer)
            .rpc_url(&rpc_url)
            .build()
            .unwrap();
        assert!(swap_target
            .set(Arc::clone(&client.signing_strategy))
            .is_ok());
        let state = WalletDepositBalancesState {
            wallet_address: Some(wallet_address),
            context_slot: Some(1),
            native_sol_balance: Some("2.000000000".into()),
            ..Default::default()
        };

        assert_eq!(
            client.positions().wrap_sol("0.1", &state).await.unwrap(),
            "confirmed-signature"
        );
        assert_eq!(transaction_calls.load(Ordering::SeqCst), 1);
    }

    #[cfg(feature = "native-auth")]
    #[tokio::test]
    async fn unwrap_uses_confirmed_submission_and_preserves_cached_state() {
        let blockhash = solana_hash::Hash::default().to_string();
        let (rpc_url, requests) = spawn_rpc_server(vec![
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "context": {"slot": 1},
                    "value": {"blockhash": blockhash, "lastValidBlockHeight": 100}
                }
            }),
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": "confirmed-signature"}),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "context": {"slot": 42},
                    "value": [{
                        "slot": 42,
                        "confirmations": 1,
                        "err": null,
                        "confirmationStatus": "confirmed",
                        "status": {"Ok": null}
                    }]
                }
            }),
        ])
        .await;
        let (client, mut state) = conversion_client(&rpc_url);
        set_wsol_balance(&mut state, "0.500000000");
        let before = state.clone();

        let signature = client.positions().unwrap_wsol(&state).await;

        assert_eq!(signature.unwrap(), "confirmed-signature");
        assert_eq!(state, before);
        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 3);
        assert!(requests[0].contains("getLatestBlockhash"));
        assert!(requests[1].contains("sendTransaction"));
        assert!(requests[2].contains("getSignatureStatuses"));
    }

    #[cfg(feature = "native-auth")]
    #[tokio::test]
    async fn wrap_propagates_submission_failure_without_mutating_state() {
        let blockhash = solana_hash::Hash::default().to_string();
        let (rpc_url, _) = spawn_rpc_server(vec![
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "context": {"slot": 1},
                    "value": {"blockhash": blockhash, "lastValidBlockHeight": 100}
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": {"code": -32000, "message": "submission failed"}
            }),
        ])
        .await;
        let (client, state) = conversion_client(&rpc_url);
        let before = state.clone();

        let error = client
            .positions()
            .wrap_sol("0.250000001", &state)
            .await
            .unwrap_err();

        assert!(error.to_string().contains("submission failed"));
        assert_eq!(state, before);
    }

    #[cfg(feature = "native-auth")]
    #[tokio::test]
    async fn wrap_rejects_invalid_or_insufficient_amounts_before_rpc() {
        let (client, state) = conversion_client("http://127.0.0.1:9");

        for amount in ["0", "0.0000000001", "3.000000000", "18446744073.709551616"] {
            assert!(client.positions().wrap_sol(amount, &state).await.is_err());
        }
    }

    #[cfg(feature = "native-auth")]
    #[tokio::test]
    async fn unwrap_requires_positive_cached_wsol_before_rpc() {
        let (client, mut state) = conversion_client("http://127.0.0.1:9");

        assert!(client.positions().unwrap_wsol(&state).await.is_err());
        set_wsol_balance(&mut state, "0.000000000");
        assert!(client.positions().unwrap_wsol(&state).await.is_err());
    }
}
