//! Positions sub-client — portfolio & position queries, and on-chain position operations.

use crate::client::LightconeClient;
use crate::domain::position::builders::{
    DepositBuilder, DepositToGlobalBuilder, ExtendPositionTokensBuilder,
    GlobalToMarketDepositBuilder, InitPositionTokensBuilder, MergeBuilder, RedeemWinningsBuilder,
    WithdrawBuilder, WithdrawFromGlobalBuilder, WithdrawFromPositionBuilder,
};
use crate::domain::position::wire::{MarketPositionsResponse, PositionsResponse};
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::program::instructions;
use crate::program::types::{
    DepositToGlobalParams, ExtendPositionTokensParams, GlobalToMarketDepositParams,
    InitPositionTokensParams, RedeemWinningsParams, WithdrawFromGlobalParams,
    WithdrawFromPositionParams,
};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;

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
        Ok(self.client.http.get(&url, RetryPolicy::Idempotent).await?)
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
        Ok(self.client.http.get(&url, RetryPolicy::Idempotent).await?)
    }

    // ── On-chain instruction builders ───────────────────────────────────

    /// Build RedeemWinnings instruction.
    pub fn redeem_winnings_ix(
        &self,
        params: &RedeemWinningsParams,
        winning_outcome: u8,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_redeem_winnings_ix(params, winning_outcome, pid)
    }

    /// Build RedeemWinnings transaction.
    pub fn redeem_winnings_tx(
        &self,
        params: RedeemWinningsParams,
        winning_outcome: u8,
    ) -> Result<Transaction, SdkError> {
        let ix = self.redeem_winnings_ix(&params, winning_outcome);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build WithdrawFromPosition instruction.
    pub fn withdraw_from_position_ix(
        &self,
        params: &WithdrawFromPositionParams,
        is_token_2022: bool,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_withdraw_from_position_ix(params, is_token_2022, pid)
    }

    /// Build WithdrawFromPosition transaction.
    pub fn withdraw_from_position_tx(
        &self,
        params: WithdrawFromPositionParams,
        is_token_2022: bool,
    ) -> Result<Transaction, SdkError> {
        let ix = self.withdraw_from_position_ix(&params, is_token_2022);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
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
        Ok(Transaction::new_with_payer(&[ix], Some(&params.payer)))
    }

    /// Build DepositToGlobal instruction.
    pub fn deposit_to_global_ix(&self, params: &DepositToGlobalParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_deposit_to_global_ix(params, pid)
    }

    /// Build DepositToGlobal transaction.
    pub fn deposit_to_global_tx(
        &self,
        params: DepositToGlobalParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.deposit_to_global_ix(&params);
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
    /// - **Market**: withdraws from position ATA
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

    /// Create a withdraw-from-position builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn withdraw_from_position(&self) -> WithdrawFromPositionBuilder<'a> {
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
        let rpc = crate::rpc::require_solana_rpc(self.client)?;
        let pda = self.pda(owner, market);
        match rpc.get_account(&pda).await {
            Ok(account) => Ok(Some(crate::program::accounts::Position::deserialize(
                &account.data,
            )?)),
            Err(_) => Ok(None),
        }
    }
}
