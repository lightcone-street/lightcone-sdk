//! Admin sub-client — metadata, referral management, and on-chain admin operations.

use crate::client::LightconeClient;
use crate::domain::admin::{
    AdminLoginRequest, AdminLoginResponse, AdminNonceResponse, AllocateCodesRequest,
    AllocateCodesResponse, CreateNotificationRequest, CreateNotificationResponse,
    DismissNotificationRequest, DismissNotificationResponse, RevokeRequest, RevokeResponse,
    UnifiedMetadataRequest, UnifiedMetadataResponse, UnrevokeRequest, UnrevokeResponse,
    WhitelistRequest, WhitelistResponse,
};
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::program::instructions;
use crate::program::types::{
    ActivateMarketParams, AddDepositMintParams, CreateMarketParams, CreateOrderbookParams,
    DepositAndSwapParams, MatchOrdersMultiParams, SetAuthorityParams, SettleMarketParams,
    WhitelistDepositTokenParams,
};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;

pub struct Admin<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Admin<'a> {
    // ── Admin auth ─────────────────────────────────────────────────────

    /// Fetch admin login nonce and message to sign.
    pub async fn get_admin_nonce(&self) -> Result<AdminNonceResponse, SdkError> {
        let url = format!("{}/api/admin/nonce", self.client.http.base_url());
        self.client.http.get(&url, RetryPolicy::None).await
    }

    /// Admin login — verifies signature and stores session cookie for subsequent admin requests.
    /// On native, the HTTP client auto-captures the `admin_token` cookie from Set-Cookie headers.
    /// On WASM, the browser handles cookie storage automatically.
    pub async fn admin_login(
        &self,
        message: &str,
        signature_bs58: &str,
        pubkey_bytes: &[u8],
    ) -> Result<AdminLoginResponse, SdkError> {
        let url = format!("{}/api/admin/login", self.client.http.base_url());
        let request = AdminLoginRequest {
            message: message.to_string(),
            signature_bs58: signature_bs58.to_string(),
            pubkey_bytes: pubkey_bytes.to_vec(),
        };
        self.client
            .http
            .post(&url, &request, RetryPolicy::None)
            .await
    }

    /// Admin logout — attempts to clear the server-side cookie and always clears the internal token.
    pub async fn admin_logout(&self) -> Result<(), SdkError> {
        let url = format!("{}/api/admin/logout", self.client.http.base_url());
        let _ = self
            .client
            .http
            .admin_post::<serde_json::Value, _>(&url, &serde_json::json!({}), RetryPolicy::None)
            .await;
        self.client.http.clear_admin_token().await;
        Ok(())
    }

    // ── Admin API methods ──────────────────────────────────────────────

    pub async fn upsert_metadata(
        &self,
        request: &UnifiedMetadataRequest,
    ) -> Result<UnifiedMetadataResponse, SdkError> {
        let url = format!("{}/api/admin/metadata", self.client.http.base_url());
        self.client
            .http
            .admin_post(&url, request, RetryPolicy::None)
            .await
    }

    pub async fn allocate_codes(
        &self,
        request: &AllocateCodesRequest,
    ) -> Result<AllocateCodesResponse, SdkError> {
        let url = format!(
            "{}/api/admin/referral/allocate",
            self.client.http.base_url()
        );
        self.client
            .http
            .admin_post(&url, request, RetryPolicy::None)
            .await
    }

    pub async fn whitelist(
        &self,
        request: &WhitelistRequest,
    ) -> Result<WhitelistResponse, SdkError> {
        let url = format!(
            "{}/api/admin/referral/whitelist",
            self.client.http.base_url()
        );
        self.client
            .http
            .admin_post(&url, request, RetryPolicy::None)
            .await
    }

    pub async fn revoke(&self, request: &RevokeRequest) -> Result<RevokeResponse, SdkError> {
        let url = format!("{}/api/admin/referral/revoke", self.client.http.base_url());
        self.client
            .http
            .admin_post(&url, request, RetryPolicy::None)
            .await
    }

    pub async fn unrevoke(&self, request: &UnrevokeRequest) -> Result<UnrevokeResponse, SdkError> {
        let url = format!(
            "{}/api/admin/referral/unrevoke",
            self.client.http.base_url()
        );
        self.client
            .http
            .admin_post(&url, request, RetryPolicy::None)
            .await
    }

    pub async fn create_notification(
        &self,
        request: &CreateNotificationRequest,
    ) -> Result<CreateNotificationResponse, SdkError> {
        let url = format!("{}/api/admin/notifications", self.client.http.base_url());
        self.client
            .http
            .admin_post(&url, request, RetryPolicy::None)
            .await
    }

    pub async fn dismiss_notification(
        &self,
        request: &DismissNotificationRequest,
    ) -> Result<DismissNotificationResponse, SdkError> {
        let url = format!(
            "{}/api/admin/notifications/dismiss",
            self.client.http.base_url()
        );
        self.client
            .http
            .admin_post(&url, request, RetryPolicy::None)
            .await
    }

    // ── On-chain instruction builders ───────────────────────────────────

    /// Build Initialize instruction.
    pub fn initialize_ix(&self, authority: &Pubkey) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_initialize_ix(authority, pid)
    }

    /// Build Initialize transaction.
    pub fn initialize_tx(&self, authority: &Pubkey) -> Result<Transaction, SdkError> {
        let ix = self.initialize_ix(authority);
        Ok(Transaction::new_with_payer(&[ix], Some(authority)))
    }

    /// Build CreateMarket instruction.
    ///
    /// Async because it fetches the next market ID from on-chain state.
    #[cfg(feature = "solana-rpc")]
    pub async fn create_market_ix(
        &self,
        params: CreateMarketParams,
    ) -> Result<Instruction, SdkError> {
        let pid = &self.client.program_id;
        let rpc = crate::rpc::require_solana_rpc(self.client)?;
        let (exchange_pda, _) = crate::program::pda::get_exchange_pda(pid);
        let account = rpc.get_account(&exchange_pda).await.map_err(|e| {
            crate::program::error::SdkError::AccountNotFound(format!("Exchange: {}", e))
        })?;
        let exchange = crate::program::accounts::Exchange::deserialize(&account.data)?;
        let market_id = exchange.market_count;
        Ok(instructions::build_create_market_ix(
            &params, market_id, pid,
        )?)
    }

    /// Build CreateMarket transaction.
    ///
    /// Async because it fetches the next market ID from on-chain state.
    #[cfg(feature = "solana-rpc")]
    pub async fn create_market_tx(
        &self,
        params: CreateMarketParams,
    ) -> Result<Transaction, SdkError> {
        let authority = params.authority;
        let ix = self.create_market_ix(params).await?;
        Ok(Transaction::new_with_payer(&[ix], Some(&authority)))
    }

    /// Build AddDepositMint instruction.
    pub fn add_deposit_mint_ix(
        &self,
        params: &AddDepositMintParams,
        market: &Pubkey,
        num_outcomes: u8,
    ) -> Result<Instruction, SdkError> {
        let pid = &self.client.program_id;
        Ok(instructions::build_add_deposit_mint_ix(
            params,
            market,
            num_outcomes,
            pid,
        )?)
    }

    /// Build AddDepositMint transaction.
    pub fn add_deposit_mint_tx(
        &self,
        params: AddDepositMintParams,
        market: &Pubkey,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let ix = self.add_deposit_mint_ix(&params, market, num_outcomes)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.authority)))
    }

    /// Build ActivateMarket instruction.
    pub fn activate_market_ix(&self, params: &ActivateMarketParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_activate_market_ix(params, pid)
    }

    /// Build ActivateMarket transaction.
    pub fn activate_market_tx(
        &self,
        params: ActivateMarketParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.activate_market_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.authority)))
    }

    /// Build SettleMarket instruction.
    pub fn settle_market_ix(&self, params: &SettleMarketParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_settle_market_ix(params, pid)
    }

    /// Build SettleMarket transaction.
    pub fn settle_market_tx(&self, params: SettleMarketParams) -> Result<Transaction, SdkError> {
        let ix = self.settle_market_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.oracle)))
    }

    /// Build SetPaused instruction.
    pub fn set_paused_ix(&self, authority: &Pubkey, paused: bool) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_set_paused_ix(authority, paused, pid)
    }

    /// Build SetPaused transaction.
    pub fn set_paused_tx(&self, authority: &Pubkey, paused: bool) -> Result<Transaction, SdkError> {
        let ix = self.set_paused_ix(authority, paused);
        Ok(Transaction::new_with_payer(&[ix], Some(authority)))
    }

    /// Build SetOperator instruction.
    pub fn set_operator_ix(&self, authority: &Pubkey, new_operator: &Pubkey) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_set_operator_ix(authority, new_operator, pid)
    }

    /// Build SetOperator transaction.
    pub fn set_operator_tx(
        &self,
        authority: &Pubkey,
        new_operator: &Pubkey,
    ) -> Result<Transaction, SdkError> {
        let ix = self.set_operator_ix(authority, new_operator);
        Ok(Transaction::new_with_payer(&[ix], Some(authority)))
    }

    /// Build SetAuthority instruction.
    pub fn set_authority_ix(&self, params: &SetAuthorityParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_set_authority_ix(params, pid)
    }

    /// Build SetAuthority transaction.
    pub fn set_authority_tx(&self, params: SetAuthorityParams) -> Result<Transaction, SdkError> {
        let ix = self.set_authority_ix(&params);
        Ok(Transaction::new_with_payer(
            &[ix],
            Some(&params.current_authority),
        ))
    }

    /// Build WhitelistDepositToken instruction.
    pub fn whitelist_deposit_token_ix(&self, params: &WhitelistDepositTokenParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_whitelist_deposit_token_ix(params, pid)
    }

    /// Build WhitelistDepositToken transaction.
    pub fn whitelist_deposit_token_tx(
        &self,
        params: WhitelistDepositTokenParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.whitelist_deposit_token_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.authority)))
    }

    /// Build CreateOrderbook instruction.
    pub fn create_orderbook_ix(&self, params: &CreateOrderbookParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_create_orderbook_ix(params, pid)
    }

    /// Build CreateOrderbook transaction.
    pub fn create_orderbook_tx(
        &self,
        params: CreateOrderbookParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.create_orderbook_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.authority)))
    }

    /// Build MatchOrdersMulti instruction.
    pub fn match_orders_multi_ix(
        &self,
        params: &MatchOrdersMultiParams,
    ) -> Result<Instruction, SdkError> {
        let pid = &self.client.program_id;
        Ok(instructions::build_match_orders_multi_ix(params, pid)?)
    }

    /// Build MatchOrdersMulti transaction.
    pub fn match_orders_multi_tx(
        &self,
        params: MatchOrdersMultiParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.match_orders_multi_ix(&params)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }

    /// Build DepositAndSwap instruction.
    pub fn deposit_and_swap_ix(
        &self,
        params: &DepositAndSwapParams,
    ) -> Result<Instruction, SdkError> {
        let pid = &self.client.program_id;
        Ok(instructions::build_deposit_and_swap_ix(params, pid)?)
    }

    /// Build DepositAndSwap transaction.
    pub fn deposit_and_swap_tx(
        &self,
        params: DepositAndSwapParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.deposit_and_swap_ix(&params)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }
}
