//! Admin sub-client — metadata, referral management, and on-chain admin operations.

use crate::client::LightconeClient;
use crate::domain::admin::{
    AdminEnvelope, AllocateCodesRequest, AllocateCodesResponse, CreateNotificationRequest,
    CreateNotificationResponse, DismissNotificationRequest, DismissNotificationResponse,
    RevokeRequest, RevokeResponse, UnifiedMetadataRequest, UnifiedMetadataResponse,
    UnrevokeRequest, UnrevokeResponse, WhitelistRequest, WhitelistResponse,
};
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::program::instructions;
use crate::program::types::{
    ActivateMarketParams, AddDepositMintParams, CreateMarketParams, CreateOrderbookParams,
    DepositAndSwapParams, MatchOrdersMultiParams, SetAuthorityParams, SettleMarketParams,
    WhitelistDepositTokenParams,
};
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;

pub struct Admin<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Admin<'a> {
    pub async fn upsert_metadata(
        &self,
        envelope: &AdminEnvelope<UnifiedMetadataRequest>,
    ) -> Result<UnifiedMetadataResponse, SdkError> {
        let url = format!("{}/api/admin/metadata", self.client.http.base_url());
        Ok(self
            .client
            .http
            .post(&url, envelope, RetryPolicy::None)
            .await?)
    }

    pub async fn allocate_codes(
        &self,
        envelope: &AdminEnvelope<AllocateCodesRequest>,
    ) -> Result<AllocateCodesResponse, SdkError> {
        let url = format!(
            "{}/api/admin/referral/allocate",
            self.client.http.base_url()
        );
        Ok(self
            .client
            .http
            .post(&url, envelope, RetryPolicy::None)
            .await?)
    }

    pub async fn whitelist(
        &self,
        envelope: &AdminEnvelope<WhitelistRequest>,
    ) -> Result<WhitelistResponse, SdkError> {
        let url = format!(
            "{}/api/admin/referral/whitelist",
            self.client.http.base_url()
        );
        Ok(self
            .client
            .http
            .post(&url, envelope, RetryPolicy::None)
            .await?)
    }

    pub async fn revoke(
        &self,
        envelope: &AdminEnvelope<RevokeRequest>,
    ) -> Result<RevokeResponse, SdkError> {
        let url = format!(
            "{}/api/admin/referral/revoke",
            self.client.http.base_url()
        );
        Ok(self
            .client
            .http
            .post(&url, envelope, RetryPolicy::None)
            .await?)
    }

    pub async fn unrevoke(
        &self,
        envelope: &AdminEnvelope<UnrevokeRequest>,
    ) -> Result<UnrevokeResponse, SdkError> {
        let url = format!(
            "{}/api/admin/referral/unrevoke",
            self.client.http.base_url()
        );
        Ok(self
            .client
            .http
            .post(&url, envelope, RetryPolicy::None)
            .await?)
    }

    pub async fn create_notification(
        &self,
        envelope: &AdminEnvelope<CreateNotificationRequest>,
    ) -> Result<CreateNotificationResponse, SdkError> {
        let url = format!(
            "{}/api/admin/notifications",
            self.client.http.base_url()
        );
        Ok(self
            .client
            .http
            .post(&url, envelope, RetryPolicy::None)
            .await?)
    }

    pub async fn dismiss_notification(
        &self,
        envelope: &AdminEnvelope<DismissNotificationRequest>,
    ) -> Result<DismissNotificationResponse, SdkError> {
        let url = format!(
            "{}/api/admin/notifications/dismiss",
            self.client.http.base_url()
        );
        Ok(self
            .client
            .http
            .post(&url, envelope, RetryPolicy::None)
            .await?)
    }

    // ── On-chain transaction builders ────────────────────────────────────

    /// Build Initialize transaction.
    pub fn initialize_ix(&self, authority: &Pubkey) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_initialize_ix(authority, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(authority)))
    }

    /// Build CreateMarket transaction.
    ///
    /// Async because it fetches the next market ID from on-chain state.
    #[cfg(feature = "solana-rpc")]
    pub async fn create_market_ix(
        &self,
        params: CreateMarketParams,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let rpc = crate::rpc::require_solana_rpc(self.client)?;
        let (exchange_pda, _) = crate::program::pda::get_exchange_pda(pid);
        let account = rpc
            .get_account(&exchange_pda)
            .await
            .map_err(|e| crate::program::error::SdkError::AccountNotFound(format!("Exchange: {}", e)))?;
        let exchange = crate::program::accounts::Exchange::deserialize(&account.data)?;
        let market_id = exchange.market_count;
        let ix = instructions::build_create_market_ix(&params, market_id, pid)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.authority)))
    }

    /// Build AddDepositMint transaction.
    pub fn add_deposit_mint_ix(
        &self,
        params: AddDepositMintParams,
        market: &Pubkey,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_add_deposit_mint_ix(&params, market, num_outcomes, pid)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.authority)))
    }

    /// Build ActivateMarket transaction.
    pub fn activate_market_ix(
        &self,
        params: ActivateMarketParams,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_activate_market_ix(&params, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.authority)))
    }

    /// Build SettleMarket transaction.
    pub fn settle_market_ix(
        &self,
        params: SettleMarketParams,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_settle_market_ix(&params, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.oracle)))
    }

    /// Build SetPaused transaction.
    pub fn set_paused_ix(
        &self,
        authority: &Pubkey,
        paused: bool,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_set_paused_ix(authority, paused, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(authority)))
    }

    /// Build SetOperator transaction.
    pub fn set_operator_ix(
        &self,
        authority: &Pubkey,
        new_operator: &Pubkey,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_set_operator_ix(authority, new_operator, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(authority)))
    }

    /// Build SetAuthority transaction.
    pub fn set_authority_ix(
        &self,
        params: SetAuthorityParams,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_set_authority_ix(&params, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.current_authority)))
    }

    /// Build WhitelistDepositToken transaction.
    pub fn whitelist_deposit_token_ix(
        &self,
        params: WhitelistDepositTokenParams,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_whitelist_deposit_token_ix(&params, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.authority)))
    }

    /// Build CreateOrderbook transaction.
    pub fn create_orderbook_ix(
        &self,
        params: CreateOrderbookParams,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_create_orderbook_ix(&params, pid);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.authority)))
    }

    /// Build MatchOrdersMulti transaction.
    pub fn match_orders_multi_ix(
        &self,
        params: MatchOrdersMultiParams,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_match_orders_multi_ix(&params, pid)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }

    /// Build DepositAndSwap transaction.
    pub fn deposit_and_swap_ix(
        &self,
        params: DepositAndSwapParams,
    ) -> Result<Transaction, SdkError> {
        let pid = &self.client.program_id;
        let ix = instructions::build_deposit_and_swap_ix(&params, pid)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }
}
