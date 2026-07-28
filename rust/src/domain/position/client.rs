//! Positions sub-client — portfolio & position queries, and on-chain position operations.

use crate::client::LightconeClient;
use crate::domain::position::builders::{
    DepositBuilder, DepositToGlobalBuilder, ExtendPositionTokensBuilder,
    GlobalToMarketDepositBuilder, InitPositionTokensBuilder, MergeBuilder, RedeemWinningsBuilder,
    WithdrawBuilder, WithdrawFromGlobalBuilder, WithdrawFromPositionBuilder,
};
use crate::domain::position::wire::{MarketPositionsResponse, PositionsResponse};
use crate::domain::position::DepositTokenBalancesSnapshot;
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::program::instructions;
use crate::program::types::{
    ClosePositionAltParams, ClosePositionTokenAccountsParams, DepositToGlobalAltContext,
    DepositToGlobalParams, ExtendPositionTokensParams, GlobalToMarketDepositParams,
    InitPositionTokensParams, RedeemWinningsParams, WithdrawConditionalFromPositionParams,
    WithdrawFromGlobalParams, WithdrawFromPositionParams,
};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;

fn deposit_token_balances_query(min_context_slot: Option<u64>) -> Vec<(&'static str, String)> {
    min_context_slot
        .map(|slot| vec![("min_context_slot", slot.to_string())])
        .unwrap_or_default()
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

    /// Get a confirmed-slot snapshot of the authenticated user's SPL
    /// deposit-token balances.
    ///
    /// When `min_context_slot` is provided, the backend only returns cached
    /// data if it was observed at or after that slot and otherwise asks
    /// Solana RPC for a sufficiently recent view.
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
    use super::deposit_token_balances_query;

    #[test]
    fn deposit_token_balances_query_includes_optional_minimum_context_slot() {
        assert!(deposit_token_balances_query(None).is_empty());
        assert_eq!(
            deposit_token_balances_query(Some(1234)),
            vec![("min_context_slot", "1234".to_string())]
        );
    }
}
