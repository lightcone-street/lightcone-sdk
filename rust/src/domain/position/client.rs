//! Positions sub-client — portfolio & position queries, and on-chain position operations.

use crate::client::LightconeClient;
use crate::domain::position::wire::{MarketPositionsResponse, PositionsResponse};
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::program::instructions;
use crate::program::types::{
    DepositToGlobalParams, ExtendPositionTokensParams, GlobalToMarketDepositParams,
    InitPositionTokensParams, RedeemWinningsParams, WithdrawFromPositionParams,
};
use solana_transaction::Transaction;

pub struct Positions<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Positions<'a> {
    /// Get all positions for a user across all markets.
    pub async fn get(&self, user_pubkey: &str) -> Result<PositionsResponse, SdkError> {
        let url = format!(
            "{}/api/users/{}/positions",
            self.client.http.base_url(),
            user_pubkey
        );
        Ok(self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?)
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
        Ok(self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?)
    }

    // ── On-chain transaction builders ────────────────────────────────────

    /// Build RedeemWinnings transaction.
    pub fn redeem_winnings_ix(
        &self,
        params: RedeemWinningsParams,
        winning_outcome: u8,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_redeem_winnings_ix(&params, winning_outcome, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build WithdrawFromPosition transaction.
    pub fn withdraw_from_position_ix(
        &self,
        params: WithdrawFromPositionParams,
        is_token_2022: bool,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_withdraw_from_position_ix(&params, is_token_2022, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build InitPositionTokens transaction.
    pub fn init_position_tokens_ix(
        &self,
        params: InitPositionTokensParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_init_position_tokens_ix(&params, num_outcomes, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.payer)))
    }

    /// Build ExtendPositionTokens transaction.
    pub fn extend_position_tokens_ix(
        &self,
        params: ExtendPositionTokensParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_extend_position_tokens_ix(&params, num_outcomes, pid)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.payer)))
    }

    /// Build DepositToGlobal transaction.
    pub fn deposit_to_global_ix(
        &self,
        params: DepositToGlobalParams,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_deposit_to_global_ix(&params, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build GlobalToMarketDeposit transaction.
    pub fn global_to_market_deposit_ix(
        &self,
        params: GlobalToMarketDepositParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_global_to_market_deposit_ix(&params, num_outcomes, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }
}
