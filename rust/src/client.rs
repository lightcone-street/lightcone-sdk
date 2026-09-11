//! High-level client — `LightconeClient` with nested sub-client accessors.
//!
//! Each domain has its own sub-client in `domain/<name>/client.rs`.
//! This module keeps the builder, auth state, and accessor methods.
//!
//! **Caching philosophy**: The SDK is stateless for HTTP data. Caching is the
//! consumer's responsibility (e.g. Dioxus server functions, CLI memoization).

use crate::auth::client::Auth;
use crate::auth::AuthCredentials;
use crate::domain::faucet::{FaucetRequest, FaucetResponse};
use crate::domain::market::client::Markets;
use crate::domain::metrics::client::Metrics;
use crate::domain::notification::client::Notifications;
use crate::domain::order::client::Orders;
use crate::domain::orderbook::client::Orderbooks;
use crate::domain::position::client::Positions;
use crate::domain::position::CanonicalWsolAccountInfo;
use crate::domain::price_history::client::PriceHistoryClient;
use crate::domain::referral::client::Referrals;
use crate::domain::trade::client::Trades;
use crate::env::LightconeEnv;
use crate::error::SdkError;
use crate::http::retry::RetryPolicy;
use crate::http::LightconeHttp;
use crate::program::transaction::{V1ResourceConfig, V1Transaction, V1TransactionContext};
use crate::rpc::Rpc;
use crate::rpc_failover::{
    is_infrastructure_error_http, with_failover, ActiveRpc, RpcFailoverState,
};
use crate::shared::signing::{ExternalSigner, SigningStrategy};
use crate::shared::OrderbookRules;
use crate::shared::{DepositSource, PubkeyStr};
use crate::ws::WsConfig;

#[cfg(feature = "solana-rpc")]
use solana_commitment_config::CommitmentConfig;
#[cfg(feature = "solana-rpc")]
use solana_rpc_client::nonblocking::rpc_client::RpcClient as SolanaRpcClient;

use async_lock::{OnceCell, RwLock};
use solana_pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::Arc;

type OrderbookRulesCell = Arc<OnceCell<OrderbookRules>>;

// Re-export sub-client types for convenience.
pub use crate::auth::client::Auth as AuthClient;
pub use crate::domain::market::client::{
    FavoriteMarketUpdate, FavoriteMarkets, GlobalDepositAssetsResult, Markets as MarketsClient,
    MarketsResult,
};
pub use crate::domain::metrics::client::Metrics as MetricsClient;
pub use crate::domain::notification::client::Notifications as NotificationsClient;
pub use crate::domain::order::client::Orders as OrdersClient;
pub use crate::domain::orderbook::client::Orderbooks as OrderbooksClient;
pub use crate::domain::position::client::Positions as PositionsClient;
pub use crate::domain::price_history::client::PriceHistoryClient as PriceHistorySubClient;
pub use crate::domain::referral::client::Referrals as ReferralsClient;
pub use crate::domain::trade::client::Trades as TradesClient;
pub use crate::rpc::Rpc as RpcClient;

/// The signer and sponsorship assertion captured together before asynchronous submission work.
///
/// One lock prevents an in-flight transaction from combining a signer from one
/// application flow with the Transaction Sponsorship Capability from another.
struct TransactionSigningContext {
    strategy: Option<SigningStrategy>,
    sponsorship_enabled: bool,
}

/// The primary entry point for the Lightcone SDK.
///
/// Provides nested sub-client accessors for each domain:
/// `client.markets()`, `client.orders()`, etc.
///
/// Market data remains stateless. Immutable orderbook trading rules are cached
/// because every signed order requires them.
pub struct LightconeClient {
    transaction_resources: Option<V1ResourceConfig>,
    pub(crate) http: LightconeHttp,
    pub(crate) ws_config: WsConfig,
    pub(crate) auth_credentials: Arc<RwLock<Option<AuthCredentials>>>,
    /// On-chain program ID (defaults to the canonical Lightcone program).
    pub(crate) program_id: Pubkey,
    /// Default deposit source for orders, deposits, and withdrawals.
    /// Per-call overrides take priority over this setting.
    pub(crate) deposit_source: Arc<RwLock<DepositSource>>,
    /// Mutable signer and trusted application assertion used by transaction submission.
    ///
    /// The capability defaults to false. Cloned clients share this context so each
    /// submission can capture a consistent signer/capability pair before yielding.
    transaction_signing_context: Arc<RwLock<TransactionSigningContext>>,
    /// Cached order nonce. When the user provides a nonce via `.nonce()` on an
    /// envelope, it is stored here. Subsequent orders that omit `.nonce()` will
    /// use this cached value, falling back to 0 if nothing has been cached.
    pub(crate) order_nonce: Arc<RwLock<Option<u64>>>,
    pub(crate) orderbook_rules: Arc<RwLock<HashMap<String, OrderbookRulesCell>>>,
    /// Primary Solana RPC URL for blockhash fetching and transaction submission.
    pub(crate) primary_rpc_url: Option<String>,
    /// Backup Solana RPC URL for automatic failover.
    pub(crate) backup_rpc_url: Option<String>,
    /// Tracks which RPC endpoint is active and cooldown state.
    pub(crate) rpc_failover_state: Arc<RwLock<RpcFailoverState>>,
    /// Primary Solana RPC client for on-chain reads (native only).
    #[cfg(feature = "solana-rpc")]
    pub(crate) primary_solana_rpc_client: Option<SolanaRpcClient>,
    /// Backup Solana RPC client for on-chain reads (native only).
    #[cfg(feature = "solana-rpc")]
    pub(crate) backup_solana_rpc_client: Option<SolanaRpcClient>,
}

impl LightconeClient {
    pub fn builder() -> LightconeClientBuilder {
        LightconeClientBuilder::default()
    }

    // ── Sub-client accessors ─────────────────────────────────────────────

    pub fn markets(&self) -> Markets<'_> {
        Markets { client: self }
    }

    pub fn orderbooks(&self) -> Orderbooks<'_> {
        Orderbooks { client: self }
    }

    pub fn orders(&self) -> Orders<'_> {
        Orders { client: self }
    }

    pub fn positions(&self) -> Positions<'_> {
        Positions { client: self }
    }

    pub fn trades(&self) -> Trades<'_> {
        Trades { client: self }
    }

    pub fn price_history(&self) -> PriceHistoryClient<'_> {
        PriceHistoryClient { client: self }
    }

    pub fn auth(&self) -> Auth<'_> {
        Auth { client: self }
    }

    pub fn referrals(&self) -> Referrals<'_> {
        Referrals { client: self }
    }

    pub fn notifications(&self) -> Notifications<'_> {
        Notifications { client: self }
    }

    /// Metrics sub-client — platform / market / orderbook / category / deposit-token
    /// volume metrics, market leaderboard, and time-series history.
    pub fn metrics(&self) -> Metrics<'_> {
        Metrics { client: self }
    }

    /// RPC sub-client — PDA helpers, account fetchers, and blockhash access.
    pub fn rpc(&self) -> Rpc<'_> {
        Rpc { client: self }
    }

    /// The HTTP transport this client is built on. Exposed so external crates
    /// can implement additional sub-clients against the same base URL, retry
    /// policies, and cookie sessions.
    pub fn http(&self) -> &LightconeHttp {
        &self.http
    }

    /// Get the WS config for creating a WebSocket connection.
    ///
    /// The WS client is intentionally not embedded in `LightconeClient`
    /// because WS connection lifetimes are typically managed at the
    /// application layer (e.g. tied to a UI component's lifecycle).
    pub fn ws_config(&self) -> &WsConfig {
        &self.ws_config
    }

    /// Create a new native WS client from the current config.
    #[cfg(feature = "ws-native")]
    pub fn ws_native(&self) -> crate::ws::native::WsClient {
        crate::ws::native::WsClient::new(self.ws_config.clone(), Some(self.http.auth_token_ref()))
    }

    /// Get the WS config for connecting with the WASM WsClient.
    ///
    /// Usage: `WsClient::connect(client.ws_config().clone(), |event| { ... })`
    #[cfg(feature = "ws-wasm")]
    pub fn ws_config_for_wasm(&self) -> &crate::ws::WsConfig {
        &self.ws_config
    }

    /// Get the program ID.
    pub fn program_id(&self) -> &Pubkey {
        &self.program_id
    }

    /// Get which RPC endpoint is currently active (Primary or Backup).
    pub async fn active_rpc(&self) -> ActiveRpc {
        self.rpc_failover_state.read().await.active()
    }

    /// Get the current `lightcone-token` cookie value, if any. Populated by the
    /// SDK after a successful login, then attached on every authed request.
    /// Useful for forwarding the token through the `_with_cookies`
    /// methods, or persisting the session across processes.
    ///
    /// Native only — on WASM the cookie lives in the browser's cookie jar
    /// and the SDK never sees it.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn auth_token(&self) -> Option<String> {
        self.http.auth_token_ref().read().await.clone()
    }

    /// Clear the cached `lightcone-token`. Subsequent authed calls will go out
    /// without a `Cookie` header (and 401) unless they use a
    /// `_with_cookies` variant.
    ///
    /// Native only — on WASM the cookie lives in the browser's cookie jar
    /// and the SDK never sees it.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn clear_auth_token(&self) {
        self.http.clear_auth_token().await;
    }

    // ── Deposit source ──────────────────────────────────────────────────

    /// Get the current deposit source setting.
    pub async fn deposit_source(&self) -> DepositSource {
        *self.deposit_source.read().await
    }

    /// Update the deposit source at runtime.
    pub async fn set_deposit_source(&self, source: DepositSource) {
        *self.deposit_source.write().await = source;
    }

    /// Resolve deposit source with priority: per-call override > client setting.
    pub async fn resolve_deposit_source(
        &self,
        override_source: Option<DepositSource>,
    ) -> DepositSource {
        match override_source {
            Some(source) => source,
            None => self.deposit_source().await,
        }
    }

    // ── Nonce cache ────────────────────────────────────────────────────

    /// Get the cached order nonce, if one has been set.
    pub async fn order_nonce(&self) -> Option<u64> {
        *self.order_nonce.read().await
    }

    /// Cache an order nonce. This value will be used as the default nonce
    /// for subsequent orders that don't explicitly call `.nonce()`.
    pub async fn set_order_nonce(&self, nonce: u64) {
        *self.order_nonce.write().await = Some(nonce);
    }

    /// Clear the cached nonce (e.g. on logout).
    pub async fn clear_order_nonce(&self) {
        *self.order_nonce.write().await = None;
    }

    // ── Signing strategy ────────────────────────────────────────────────

    /// Capture one consistent signer and sponsorship assertion before async work.
    async fn transaction_signing_snapshot(&self) -> (Option<SigningStrategy>, bool) {
        let context = self.transaction_signing_context.read().await;
        (context.strategy.clone(), context.sponsorship_enabled)
    }

    /// Get the current signing strategy, if set.
    pub async fn signing_strategy(&self) -> Option<SigningStrategy> {
        self.transaction_signing_context
            .read()
            .await
            .strategy
            .clone()
    }

    /// Set the signing strategy at runtime.
    ///
    /// Common use: set during login when the wallet type is known.
    pub async fn set_signing_strategy(&self, strategy: SigningStrategy) {
        self.transaction_signing_context.write().await.strategy = Some(strategy);
    }

    /// Clear the signing strategy (e.g. on logout).
    pub async fn clear_signing_strategy(&self) {
        self.transaction_signing_context.write().await.strategy = None;
    }

    /// Return whether the active application flow asserts transaction fee sponsorship.
    pub async fn transaction_sponsorship_enabled(&self) -> bool {
        self.transaction_signing_context
            .read()
            .await
            .sponsorship_enabled
    }

    /// Replace the client-wide Transaction Sponsorship Capability at runtime.
    ///
    /// Enabling this capability is a trusted application assertion for external
    /// signing. Shared submission rejects it when the active strategy is a local
    /// keypair because the SDK cannot provide sponsorship for that path.
    pub async fn set_transaction_sponsorship_enabled(&self, enabled: bool) {
        self.transaction_signing_context
            .write()
            .await
            .sponsorship_enabled = enabled;
    }

    /// Atomically replace the signer and Transaction Sponsorship Capability.
    ///
    /// Applications use this at account-session boundaries so a submission sees
    /// either the old context or the new context, never a cross-session pair.
    pub async fn set_transaction_signing_context(
        &self,
        strategy: SigningStrategy,
        sponsorship_enabled: bool,
    ) {
        *self.transaction_signing_context.write().await = TransactionSigningContext {
            strategy: Some(strategy),
            sponsorship_enabled,
        };
    }

    /// Atomically remove the signer and disable transaction sponsorship.
    pub async fn clear_transaction_signing_context(&self) {
        *self.transaction_signing_context.write().await = TransactionSigningContext {
            strategy: None,
            sponsorship_enabled: false,
        };
    }

    /// Register the credential restorer consulted when a request fails with
    /// HTTP 401: it attempts to restore credentials (e.g. refresh the app's
    /// auth session so the auth cookie is valid again); on success the
    /// transport replays the request once IF it declared itself retry-safe
    /// (mutations with `RetryPolicy::None` are never auto-replayed). See
    /// [`crate::http::CredentialRestorer`]. Without a restorer, 401s
    /// propagate to callers unchanged.
    ///
    /// Common use: set once at app startup, alongside the signing strategy.
    pub async fn set_credential_restorer(
        &self,
        restorer: std::sync::Arc<dyn crate::http::CredentialRestorer>,
    ) {
        self.http.set_credential_restorer(restorer).await;
    }

    /// Remove the credential restorer (e.g. in tests); 401s propagate again.
    pub async fn clear_credential_restorer(&self) {
        self.http.clear_credential_restorer().await;
    }

    // ── Faucet (testnet only) ──────────────────────────────────────────

    /// Request testnet SOL and whitelisted deposit tokens for a wallet.
    ///
    /// Only active on environments whose backend has the faucet enabled
    /// (typically local and staging). Returns the mint tx signature plus the
    /// SOL and token amounts transferred.
    ///
    /// `POST /api/claim`
    pub async fn claim(&self, wallet_address: &PubkeyStr) -> Result<FaucetResponse, SdkError> {
        let url = format!("{}/api/claim", self.http.base_url());
        let request = FaucetRequest {
            wallet_address: wallet_address.clone(),
        };
        self.http.post(&url, &request, RetryPolicy::None).await
    }

    // ── RPC helpers (HTTP-based, works on all platforms) ─────────────────

    /// Execute a JSON-RPC call with fast retry + failover.
    async fn rpc_call_with_failover<T: serde::de::DeserializeOwned>(
        &self,
        body: &serde_json::Value,
    ) -> Result<T, SdkError> {
        let primary = self.primary_rpc_url.as_deref();
        let backup = self.backup_rpc_url.as_deref();

        if primary.is_none() && backup.is_none() {
            return Err(SdkError::Validation(
                "rpc_url is not configured on the client".into(),
            ));
        }

        with_failover(
            &self.rpc_failover_state,
            |target| {
                let url = match target {
                    ActiveRpc::Primary => primary.or(backup),
                    ActiveRpc::Backup => backup.or(primary),
                }
                .unwrap();
                Box::pin(self.http.raw_post::<T, _>(url, body))
            },
            backup.is_some(),
            is_infrastructure_error_http,
        )
        .await
        .map_err(SdkError::Http)
    }

    /// Send one JSON-RPC request to the active endpoint and return its response.
    ///
    /// Scheduled primary recovery runs before endpoint selection. The request is
    /// not retried and does not switch endpoints. Signed-transaction callers use
    /// this path because a transport failure can occur after RPC acceptance.
    async fn rpc_call_once<T: serde::de::DeserializeOwned>(
        &self,
        body: &serde_json::Value,
    ) -> Result<T, SdkError> {
        let primary = self.primary_rpc_url.as_deref();
        let backup = self.backup_rpc_url.as_deref();
        let active = {
            let mut state = self.rpc_failover_state.write().await;
            state.maybe_recover_to_primary();
            state.active()
        };
        let url = match active {
            ActiveRpc::Primary => primary.or(backup),
            ActiveRpc::Backup => backup.or(primary),
        }
        .ok_or_else(|| SdkError::Validation("rpc_url is not configured on the client".into()))?;

        self.http.raw_post(url, body).await.map_err(SdkError::Http)
    }

    /// Fetch the latest blockhash via JSON-RPC POST.
    ///
    /// Works on all platforms (native + WASM). Uses the active RPC URL,
    /// with automatic failover to the backup if configured.
    pub async fn get_latest_blockhash(&self) -> Result<solana_hash::Hash, SdkError> {
        let (blockhash, _last_valid_block_height) = self.get_latest_blockhash_with_height().await?;
        Ok(blockhash)
    }

    /// Fetch the latest blockhash together with its `lastValidBlockHeight`
    /// via JSON-RPC POST.
    ///
    /// The blockhash is requested at `confirmed` commitment — the freshest
    /// hash that is safe to build on, which maximizes the ~150-block validity
    /// window. The returned height is the last block height at which the
    /// blockhash is still valid: once the chain moves past it, a transaction
    /// built on the blockhash can never land, which is what makes expiry
    /// detection in [`Self::confirm_signature`] safe.
    ///
    /// Works on all platforms (native + WASM). Uses the active RPC URL,
    /// with automatic failover to the backup if configured.
    pub async fn get_latest_blockhash_with_height(
        &self,
    ) -> Result<(solana_hash::Hash, u64), SdkError> {
        let body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "getLatestBlockhash",
            "params": [{ "commitment": "confirmed" }]
        });

        let response: serde_json::Value = self.rpc_call_with_failover(&body).await?;

        let blockhash_str = response["result"]["value"]["blockhash"]
            .as_str()
            .ok_or_else(|| SdkError::Other("missing blockhash in RPC response".into()))?;
        let blockhash = blockhash_str
            .parse::<solana_hash::Hash>()
            .map_err(|error| SdkError::Other(format!("invalid blockhash: {error}")))?;

        let last_valid_block_height = response["result"]["value"]["lastValidBlockHeight"]
            .as_u64()
            .ok_or_else(|| {
                SdkError::Other("missing lastValidBlockHeight in RPC response".into())
            })?;

        Ok((blockhash, last_valid_block_height))
    }

    /// Fetch the current block height at `confirmed` commitment via JSON-RPC POST.
    pub async fn get_block_height(&self) -> Result<u64, SdkError> {
        let body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "getBlockHeight",
            "params": [{ "commitment": "confirmed" }]
        });

        let response: serde_json::Value = self.rpc_call_with_failover(&body).await?;

        if let Some(error) = response.get("error") {
            return Err(SdkError::Other(format!("RPC error: {error}")));
        }

        response["result"]
            .as_u64()
            .ok_or_else(|| SdkError::Other("missing block height in RPC response".into()))
    }

    /// Return whether an account exists at confirmed commitment.
    ///
    /// This HTTP JSON-RPC path is available to native and WASM consumers and
    /// deliberately distinguishes a missing account (`false`) from an RPC
    /// failure (`Err`) so transaction planning never guesses account presence.
    pub async fn account_exists(&self, address: &Pubkey) -> Result<bool, SdkError> {
        let body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "getAccountInfo",
            "params": [
                address.to_string(),
                { "commitment": "confirmed", "encoding": "base64" }
            ]
        });
        let response: serde_json::Value = self.rpc_call_with_failover(&body).await?;
        if let Some(error) = response.get("error") {
            return Err(SdkError::Other(format!("RPC error: {error}")));
        }
        let value = response
            .get("result")
            .and_then(|result| result.get("value"))
            .ok_or_else(|| SdkError::Other("missing account value in RPC response".into()))?;
        Ok(!value.is_null())
    }

    /// Return exact confirmed facts for the Trading Wallet's canonical WSOL account.
    ///
    /// `address` must equal the supplied Trading Wallet's Tokenkeg native-mint
    /// ATA. Missing accounts return `None`. A present account must have the legacy
    /// Token Program owner, native mint, Trading Wallet authority, initialized
    /// state, native reserve, and no foreign close authority. The decoded token
    /// amount plus native reserve must fit `u64` and cannot exceed the RPC account
    /// balance. All three returned fields come from the same `getAccountInfo`
    /// response. Malformed, incompatible, or unavailable responses return an
    /// error instead of being treated as absence.
    pub async fn canonical_wsol_account_info(
        &self,
        address: &Pubkey,
        wallet: &Pubkey,
    ) -> Result<Option<CanonicalWsolAccountInfo>, SdkError> {
        use solana_program_pack::Pack;

        let expected_address = spl_associated_token_account_interface::address::get_associated_token_address_with_program_id(
            wallet,
            &spl_token_interface::native_mint::id(),
            &spl_token_interface::id(),
        );
        if *address != expected_address {
            return Err(SdkError::Validation(
                "canonical WSOL address is not the Trading Wallet's Tokenkeg native-mint ATA"
                    .into(),
            ));
        }

        let body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "getAccountInfo",
            "params": [
                address.to_string(),
                { "commitment": "confirmed", "encoding": "base64" }
            ]
        });
        let response: serde_json::Value = self.rpc_call_with_failover(&body).await?;
        if let Some(error) = response.get("error") {
            return Err(SdkError::Other(format!("RPC error: {error}")));
        }
        let value = response
            .get("result")
            .and_then(|result| result.get("value"))
            .ok_or_else(|| SdkError::Other("missing account value in RPC response".into()))?;
        if value.is_null() {
            return Ok(None);
        }
        let account_lamports = value
            .get("lamports")
            .and_then(serde_json::Value::as_u64)
            .ok_or_else(|| {
                SdkError::Validation(
                    "canonical WSOL account lamports are missing or invalid".into(),
                )
            })?;
        let owner = value
            .get("owner")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| SdkError::Validation("canonical WSOL account owner is missing".into()))?
            .parse::<Pubkey>()
            .map_err(|error| {
                SdkError::Validation(format!("canonical WSOL account owner is invalid: {error}"))
            })?;
        if owner != spl_token_interface::id() {
            return Err(SdkError::Validation(
                "canonical WSOL account is not owned by the legacy Token Program".into(),
            ));
        }
        let encoded = value
            .pointer("/data/0")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| SdkError::Validation("canonical WSOL account data is missing".into()))?;
        let encoding = value
            .pointer("/data/1")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                SdkError::Validation("canonical WSOL account data encoding is missing".into())
            })?;
        if encoding != "base64" {
            return Err(SdkError::Validation(
                "canonical WSOL account data is not base64 encoded".into(),
            ));
        }
        let data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
            .map_err(|error| {
                SdkError::Validation(format!("canonical WSOL account data is invalid: {error}"))
            })?;
        let account = spl_token_interface::state::Account::unpack(&data).map_err(|error| {
            SdkError::Validation(format!("canonical WSOL token account is invalid: {error}"))
        })?;
        if account.mint != spl_token_interface::native_mint::id()
            || account.owner != *wallet
            || account.state != spl_token_interface::state::AccountState::Initialized
        {
            return Err(SdkError::Validation(
                "canonical WSOL token account has incompatible mint, authority, or native state"
                    .into(),
            ));
        }
        let native_reserve = account.is_native.ok_or_else(|| {
            SdkError::Validation(
                "canonical WSOL token account has incompatible mint, authority, or native state"
                    .into(),
            )
        })?;
        if account.close_authority.is_some() && !account.close_authority.contains(wallet) {
            return Err(SdkError::Validation(
                "canonical WSOL close authority does not match Trading Wallet".into(),
            ));
        }
        let accounted_lamports = account.amount.checked_add(native_reserve).ok_or_else(|| {
            SdkError::Validation(
                "canonical WSOL token amount plus native reserve overflows u64".into(),
            )
        })?;
        if accounted_lamports > account_lamports {
            return Err(SdkError::Validation(
                "canonical WSOL accounted lamports exceed RPC account lamports".into(),
            ));
        }
        Ok(Some(CanonicalWsolAccountInfo {
            account_lamports,
            token_amount_lamports: account.amount,
            native_reserve_lamports: native_reserve,
        }))
    }

    /// Return whether the wallet's valid canonical Tokenkeg WSOL account exists.
    ///
    /// This compatibility surface delegates to exact inspection so callers keep
    /// the shipped boolean API while sharing all ownership and token-state checks.
    pub async fn canonical_wsol_account_exists(
        &self,
        address: &Pubkey,
        wallet: &Pubkey,
    ) -> Result<bool, SdkError> {
        Ok(self
            .canonical_wsol_account_info(address, wallet)
            .await?
            .is_some())
    }

    /// Fetch the current rent-exempt minimum, in lamports, for `data_len` account bytes.
    pub async fn minimum_balance_for_rent_exemption(
        &self,
        data_len: usize,
    ) -> Result<u64, SdkError> {
        let body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "getMinimumBalanceForRentExemption",
            "params": [data_len, { "commitment": "confirmed" }]
        });
        let response: serde_json::Value = self.rpc_call_with_failover(&body).await?;
        if let Some(error) = response.get("error") {
            return Err(SdkError::Other(format!("RPC error: {error}")));
        }
        response["result"]
            .as_u64()
            .ok_or_else(|| SdkError::Other("missing rent exemption value in RPC response".into()))
    }

    /// Return the live fee in lamports without changing the prepared v1 message.
    ///
    /// `getFeeForMessage` returning null is an unavailable estimate, not a zero
    /// fee. Callers must therefore fail closed rather than falling back to a
    /// configured reserve floor.
    pub async fn prepare_and_estimate_transaction_fee(
        &self,
        transaction: &V1Transaction,
    ) -> Result<u64, SdkError> {
        self.estimate_prepared_transaction_fee(transaction).await
    }

    /// Return the live fee in lamports for a message that already carries the
    /// blockhash the caller will sign. This never mutates the prepared transaction;
    /// a null RPC estimate fails closed rather than becoming a zero fee.
    pub async fn estimate_prepared_transaction_fee(
        &self,
        transaction: &V1Transaction,
    ) -> Result<u64, SdkError> {
        if transaction.message().lifetime_specifier == solana_hash::Hash::default() {
            return Err(SdkError::Validation(
                "prepared transaction is missing a recent blockhash".into(),
            ));
        }
        let message = transaction.message_bytes()?;
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, message);
        let body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "getFeeForMessage",
            "params": [encoded, { "commitment": "confirmed" }]
        });
        let response: serde_json::Value = self.rpc_call_with_failover(&body).await?;
        if let Some(error) = response.get("error") {
            return Err(SdkError::Other(format!("RPC error: {error}")));
        }
        response["result"]["value"]
            .as_u64()
            .ok_or_else(|| SdkError::Other("transaction fee estimate is unavailable".into()))
    }

    /// Return the confirmed Native SOL Balance for `fee_payer`, in lamports.
    pub async fn get_balance_lamports(&self, fee_payer: &Pubkey) -> Result<u64, SdkError> {
        let body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "getBalance",
            "params": [fee_payer.to_string(), { "commitment": "confirmed" }]
        });
        let response: serde_json::Value = self.rpc_call_with_failover(&body).await?;
        if let Some(error) = response.get("error") {
            return Err(SdkError::Other(format!("RPC error: {error}")));
        }
        response["result"]["value"]
            .as_u64()
            .ok_or_else(|| SdkError::Other("fee-payer balance is unavailable".into()))
    }

    /// Reject proven fee shortfalls before signing while preserving submission on unknown evidence.
    ///
    /// The prepared message supplies the exact fee and declared fee payer. Fee
    /// or balance lookup failure is deliberately best-effort and returns `Ok`;
    /// planner-owned SOL admission remains fail-closed before reaching this path.
    /// The signer and sponsorship value were captured together before RPC work.
    async fn preflight_transaction_fee_funding(
        &self,
        transaction: &V1Transaction,
        strategy: &SigningStrategy,
        sponsorship_enabled: bool,
    ) -> Result<(), SdkError> {
        self.validate_transaction_fee_funding_context(transaction, strategy, sponsorship_enabled)?;
        let fee_payer = transaction.message().account_keys.first().ok_or_else(|| {
            SdkError::Validation("transaction is missing a declared fee payer".into())
        })?;

        if sponsorship_enabled {
            return Ok(());
        }

        let required_lamports = match self.estimate_prepared_transaction_fee(transaction).await {
            Ok(required_lamports) => required_lamports,
            Err(_) => return Ok(()),
        };
        let available_lamports = match self.get_balance_lamports(fee_payer).await {
            Ok(available_lamports) => available_lamports,
            Err(_) => return Ok(()),
        };
        if available_lamports < required_lamports {
            return Err(SdkError::InsufficientSolForTransactionFees {
                available_lamports,
                required_lamports,
            });
        }
        Ok(())
    }

    /// Reject invalid payer and sponsorship combinations before submission can yield.
    ///
    /// Unsponsored known signers must control the payer being classified. Sponsored
    /// external flows may use a different payer, while native sponsorship is rejected
    /// before blockhash RPC or caller-transaction mutation.
    fn validate_transaction_fee_funding_context(
        &self,
        transaction: &V1Transaction,
        strategy: &SigningStrategy,
        sponsorship_enabled: bool,
    ) -> Result<(), SdkError> {
        let fee_payer = transaction.message().account_keys.first().ok_or_else(|| {
            SdkError::Validation("transaction is missing a declared fee payer".into())
        })?;
        if sponsorship_enabled {
            if strategy.is_local_keypair() {
                return Err(SdkError::Validation(
                    "transaction sponsorship is not supported with local-keypair signing".into(),
                ));
            }
            return Ok(());
        }
        if strategy
            .wallet_address()
            .is_some_and(|signing_wallet| signing_wallet != *fee_payer)
        {
            return Err(SdkError::Validation(
                "signing strategy does not control transaction fee payer".into(),
            ));
        }
        Ok(())
    }

    /// Fetch the statuses of recently submitted transactions via JSON-RPC POST.
    ///
    /// Returns one entry per signature, in order; `None` means the cluster has
    /// not seen the signature (or it has aged out of the recent-status cache).
    pub async fn get_signature_statuses(
        &self,
        signatures: &[String],
    ) -> Result<Vec<Option<TransactionStatus>>, SdkError> {
        self.get_signature_statuses_inner(signatures, false).await
    }

    /// Like [`Self::get_signature_statuses`], but also searches ledger history
    /// for signatures that have aged out of the recent-status cache.
    pub async fn get_signature_statuses_with_history(
        &self,
        signatures: &[String],
    ) -> Result<Vec<Option<TransactionStatus>>, SdkError> {
        self.get_signature_statuses_inner(signatures, true).await
    }

    async fn get_signature_statuses_inner(
        &self,
        signatures: &[String],
        search_transaction_history: bool,
    ) -> Result<Vec<Option<TransactionStatus>>, SdkError> {
        let body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "getSignatureStatuses",
            "params": [
                signatures,
                { "searchTransactionHistory": search_transaction_history }
            ]
        });

        let response: serde_json::Value = self.rpc_call_with_failover(&body).await?;

        if let Some(error) = response.get("error") {
            return Err(SdkError::Other(format!("RPC error: {error}")));
        }

        serde_json::from_value(response["result"]["value"].clone()).map_err(SdkError::Serde)
    }

    /// Wait until `signature` reaches `confirmed` commitment, or fail with a
    /// terminal error.
    ///
    /// Polls `getSignatureStatuses` (with automatic RPC failover) until the
    /// cluster reports the transaction as `confirmed` or `finalized`.
    /// `last_valid_block_height` bounds the wait: pass the height returned
    /// alongside the transaction's blockhash, or `None` when the submitted
    /// transaction's original expiry was not retained — expiry is then never
    /// reported and only the poll
    /// cap ends the wait. Terminal outcomes:
    ///
    /// - [`SdkError::TransactionFailed`] — the transaction landed but errored
    ///   on-chain; resubmitting the same transaction would fail again.
    /// - [`SdkError::TransactionExpired`] — the chain moved past
    ///   `last_valid_block_height` on consecutive height samples and a
    ///   history-searching status check still cannot see the signature; the
    ///   transaction cannot newly land. Reconcile its signature and authoritative
    ///   state before rebuilding; one RPC's absent history is not global proof.
    /// - [`SdkError::ConfirmationTimeout`] — the outcome could not be
    ///   determined (persistent RPC errors or the poll cap); check the
    ///   signature on-chain before resubmitting.
    pub async fn confirm_signature(
        &self,
        signature: &str,
        last_valid_block_height: Option<u64>,
    ) -> Result<(), SdkError> {
        self.confirm_signature_status(signature, last_valid_block_height)
            .await
            .map(|_| ())
    }

    /// Same as [`Self::confirm_signature`], but returns the confirmed
    /// transaction status so callers can use its processing slot.
    pub async fn confirm_signature_status(
        &self,
        signature: &str,
        last_valid_block_height: Option<u64>,
    ) -> Result<TransactionStatus, SdkError> {
        let signatures = [signature.to_string()];
        let mut consecutive_failures: u32 = 0;
        let mut over_bound_samples: u32 = 0;

        for _ in 0..MAX_CONFIRMATION_POLLS {
            match self.get_signature_statuses(&signatures).await {
                Err(_error) => {
                    consecutive_failures += 1;
                    // A failed poll is a gap in expiry evidence — restart it.
                    over_bound_samples = 0;
                    // Transport errors are deliberately not interpolated into
                    // the logs: their rendered text can carry RPC endpoint
                    // URLs with embedded provider credentials.
                    if consecutive_failures >= MAX_CONSECUTIVE_POLL_FAILURES {
                        tracing::warn!(
                            "Giving up confirming {signature} after {consecutive_failures} failed status polls"
                        );
                        return Err(SdkError::ConfirmationTimeout {
                            signature: signature.to_string(),
                        });
                    }
                    tracing::warn!(
                        "Signature status poll for {signature} failed ({consecutive_failures} consecutive)"
                    );
                }
                Ok(statuses) => {
                    consecutive_failures = 0;
                    match statuses.into_iter().next().flatten() {
                        Some(status) if status.is_confirmed() => {
                            return match status.err.as_ref() {
                                Some(err) => Err(SdkError::TransactionFailed {
                                    signature: signature.to_string(),
                                    error: err.to_string(),
                                }),
                                None => Ok(status),
                            };
                        }
                        // Seen but below `confirmed` — keep waiting (failed
                        // transactions land in blocks like any other, so an
                        // on-chain error is also reported once confirmed) and
                        // restart expiry evidence: a sighting means the
                        // transaction is live, so expiry must be re-proven
                        // from scratch afterwards.
                        Some(_) => {
                            over_bound_samples = 0;
                        }
                        // Unseen. When the expiry bound is unknown, only the
                        // poll cap ends the wait.
                        None => {
                            if let Some(last_valid_block_height) = last_valid_block_height {
                                // Sample the block height. Expiry requires
                                // `EXPIRY_HEIGHT_SAMPLES` consecutive over-bound
                                // samples (a single reading can come from a
                                // forward-skewed node, and each sample follows a
                                // fresh unseen status), then is still verified
                                // against ledger history before being declared.
                                over_bound_samples = match self.get_block_height().await {
                                    Ok(block_height) if block_height > last_valid_block_height => {
                                        over_bound_samples + 1
                                    }
                                    // Under-bound reading, or height unavailable —
                                    // expiry evidence must be strictly consecutive
                                    // over-bound readings.
                                    _ => 0,
                                };
                                if over_bound_samples >= EXPIRY_HEIGHT_SAMPLES {
                                    // Search ledger history before declaring
                                    // expiry — the recent-status cache can evict
                                    // landed transactions. A successful absent
                                    // lookup is expiry evidence for this RPC,
                                    // not a promise of global resubmit safety.
                                    // On a failed lookup, keep polling
                                    // until the cap.
                                    if let Ok(history) =
                                        self.get_signature_statuses_with_history(&signatures).await
                                    {
                                        match history.into_iter().next().flatten() {
                                            None => {
                                                return Err(SdkError::TransactionExpired {
                                                    signature: signature.to_string(),
                                                });
                                            }
                                            Some(landed) if landed.is_confirmed() => {
                                                return match landed.err.as_ref() {
                                                    Some(err) => Err(SdkError::TransactionFailed {
                                                        signature: signature.to_string(),
                                                        error: err.to_string(),
                                                    }),
                                                    None => Ok(landed),
                                                };
                                            }
                                            // Landed but below `confirmed` —
                                            // keep waiting and restart expiry
                                            // evidence.
                                            Some(_) => {
                                                over_bound_samples = 0;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            futures_timer::Delay::new(CONFIRMATION_POLL_INTERVAL).await;
        }

        Err(SdkError::ConfirmationTimeout {
            signature: signature.to_string(),
        })
    }

    /// Sign and submit a transaction using the client's signing strategy.
    ///
    /// Preserves the blockhash, expiry, and resources supplied during compilation.
    /// Before signing an unsponsored transaction, a best-effort preflight returns
    /// [`SdkError::InsufficientSolForTransactionFees`] only when the exact fee and
    /// confirmed fee-payer balance prove a shortfall; unavailable evidence proceeds.
    /// Returns as soon as the RPC accepts the transaction — inclusion is not
    /// awaited. When follow-up work depends on this transaction's on-chain
    /// effects, use [`Self::sign_and_submit_tx_confirmed`] instead.
    ///
    /// - **Native**: signs locally with keypair, submits via RPC `sendTransaction`
    /// - **WalletAdapter**: signs via external signer, submits via RPC `sendTransaction`
    pub async fn sign_and_submit_tx(&self, tx: V1Transaction) -> Result<String, SdkError> {
        let (strategy, sponsorship_enabled) = self.transaction_signing_snapshot().await;
        let strategy = strategy.ok_or_else(|| {
            SdkError::Validation("signing strategy is not set on the client".into())
        })?;
        let (signature, _last_valid_block_height) = self
            .sign_and_submit_tx_inner(tx, strategy, sponsorship_enabled)
            .await?;
        Ok(signature)
    }

    /// Sign and submit a transaction, then wait until it reaches `confirmed`
    /// commitment on-chain.
    ///
    /// Sequential flows should prefer this over [`Self::sign_and_submit_tx`]:
    /// a transaction that depends on a prior transaction's state is only safe
    /// to send once that prior transaction has confirmed. See
    /// [`Self::confirm_signature`] for the terminal error taxonomy.
    ///
    /// Confirmation retains the original blockhash expiry for both native and
    /// wallet signing. A wallet response that changes the message is rejected.
    pub async fn sign_and_submit_tx_confirmed(
        &self,
        tx: V1Transaction,
    ) -> Result<String, SdkError> {
        self.sign_and_submit_tx_confirmed_with_slot(tx)
            .await
            .map(|confirmed| confirmed.signature)
    }

    /// Sign and submit a transaction, wait for confirmed commitment, and
    /// return both its signature and processing slot.
    pub async fn sign_and_submit_tx_confirmed_with_slot(
        &self,
        tx: V1Transaction,
    ) -> Result<ConfirmedTransaction, SdkError> {
        let (strategy, sponsorship_enabled) = self.transaction_signing_snapshot().await;
        let strategy = strategy.ok_or_else(|| {
            SdkError::Validation("signing strategy is not set on the client".into())
        })?;
        let (signature, last_valid_block_height) = self
            .sign_and_submit_tx_inner(tx, strategy, sponsorship_enabled)
            .await?;
        let status = self
            .confirm_signature_status(&signature, last_valid_block_height)
            .await?;
        Ok(ConfirmedTransaction {
            signature,
            slot: status.slot,
        })
    }

    /// Sign, submit once, and confirm the exact fee-prepared v1 message.
    /// Its blockhash expiry is retained from planning through confirmation.
    pub async fn sign_and_submit_prepared_tx_confirmed_with_slot(
        &self,
        tx: V1Transaction,
    ) -> Result<ConfirmedTransaction, SdkError> {
        self.sign_and_submit_tx_confirmed_with_slot(tx).await
    }

    /// Fetch a confirmed blockhash with explicit caller-selected v1 resources.
    pub async fn transaction_context_with_resources(
        &self,
        resources: V1ResourceConfig,
    ) -> Result<V1TransactionContext, SdkError> {
        resources.validate()?;
        let (blockhash, last_valid_block_height) = self.get_latest_blockhash_with_height().await?;
        Ok(V1TransactionContext {
            blockhash,
            last_valid_block_height,
            resources,
        })
    }

    /// Fetch context using the resources explicitly set on the client builder.
    /// Planners and fluent submitters fail if no resource policy was configured.
    pub async fn transaction_context(&self) -> Result<V1TransactionContext, SdkError> {
        let resources = self.transaction_resources.ok_or_else(|| SdkError::Validation(
            "v1 transaction resources are required; set transaction_resources on the client builder".into()
        ))?;
        self.transaction_context_with_resources(resources).await
    }

    /// Require v1 activation in the feature account's own confirmed bank snapshot.
    pub async fn ensure_v1_supported(&self) -> Result<(), SdkError> {
        let response: serde_json::Value = self
            .rpc_call_with_failover(&serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "getAccountInfo",
                "params": ["txv1aq4pp281K9um3tnPgkfX8UqtFT6wcVW3hNezGLL", {
                    "commitment": "confirmed", "encoding": "base64"
                }]
            }))
            .await?;
        validate_v1_feature_response(&response)
    }

    /// Simulate the exact signed v1 message, including signatures and blockhash.
    /// A provider that cannot decode v1 fails here, before submission.
    pub async fn simulate_transaction(
        &self,
        tx: &V1Transaction,
    ) -> Result<TransactionSimulation, SdkError> {
        tx.verify_signatures()?;
        self.ensure_v1_supported().await?;
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            tx.to_wire_bytes()?,
        );
        let response: serde_json::Value = self
            .rpc_call_with_failover(&serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "simulateTransaction",
                "params": [encoded, {
                    "encoding": "base64", "commitment": "confirmed",
                    "sigVerify": true, "replaceRecentBlockhash": false
                }]
            }))
            .await?;
        if let Some(error) = response.get("error") {
            return Err(SdkError::Other(format!("v1 simulation RPC error: {error}")));
        }
        let result = response
            .get("result")
            .ok_or_else(|| SdkError::Other("missing simulation result".into()))?;
        let value = result
            .get("value")
            .ok_or_else(|| SdkError::Other("missing simulation value".into()))?;
        let error = value
            .get("err")
            .ok_or_else(|| SdkError::Other("missing simulation status".into()))?;
        if !error.is_null() {
            return Err(SdkError::Other(format!(
                "v1 simulation failed: {error}; logs: {}",
                value["logs"]
            )));
        }
        Ok(TransactionSimulation {
            slot: result["context"]["slot"]
                .as_u64()
                .ok_or_else(|| SdkError::Other("missing simulation slot".into()))?,
            units_consumed: value["unitsConsumed"].as_u64(),
            loaded_accounts_data_size: value["loadedAccountsDataSize"].as_u64(),
            logs: serde_json::from_value(value["logs"].clone()).unwrap_or_default(),
        })
    }

    /// Validate funding, sign the exact message, simulate, and submit once.
    async fn sign_and_submit_tx_inner(
        &self,
        tx: V1Transaction,
        strategy: SigningStrategy,
        sponsorship_enabled: bool,
    ) -> Result<(String, Option<u64>), SdkError> {
        self.validate_transaction_fee_funding_context(&tx, &strategy, sponsorship_enabled)?;
        if !sponsorship_enabled && strategy.wallet_address().is_none() {
            return Err(SdkError::Validation(
                "signing strategy wallet identity is required".into(),
            ));
        }
        self.preflight_transaction_fee_funding(&tx, &strategy, sponsorship_enabled)
            .await?;
        self.ensure_v1_supported().await?;
        let height = tx.context().last_valid_block_height;
        let signed = match strategy {
            #[cfg(feature = "native-auth")]
            SigningStrategy::Native(keypair) => tx.sign(&[keypair.as_ref()])?,
            SigningStrategy::WalletAdapter(signer) => {
                let bytes = signer
                    .sign_transaction(&tx.to_wire_bytes()?)
                    .await
                    .map_err(crate::shared::signing::classify_signer_error)?;
                tx.accept_signed_bytes(&bytes)?
            }
        };
        let signature = self.submit_signed_transaction(&signed).await?;
        Ok((signature, Some(height)))
    }

    /// Submit an already signed v1 transaction. The message is never rebuilt.
    /// Transport errors retain the signature and expiry for reconciliation.
    pub async fn submit_signed_transaction(&self, tx: &V1Transaction) -> Result<String, SdkError> {
        tx.verify_signatures()?;
        self.simulate_transaction(tx).await?;
        let signature = tx.as_versioned().signatures[0].to_string();
        let encoded = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            tx.to_wire_bytes()?,
        );
        let response: serde_json::Value = self
            .rpc_call_once(&serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "sendTransaction",
                "params": [encoded, {
                    "encoding": "base64", "skipPreflight": false,
                    "preflightCommitment": "confirmed", "maxRetries": 0
                }]
            }))
            .await
            .map_err(|_| SdkError::SubmissionUnknown {
                signature: signature.clone(),
                last_valid_block_height: tx.context().last_valid_block_height,
                reason: "transport failure while submitting transaction".into(),
            })?;
        if let Some(rejection) = rpc_submission_rejection(&response, &signature) {
            return Err(rejection);
        }
        if !response["error"].is_null() || response["result"].as_str() != Some(signature.as_str()) {
            return Err(SdkError::SubmissionUnknown {
                signature,
                last_valid_block_height: tx.context().last_valid_block_height,
                reason: "RPC did not acknowledge the expected signature".into(),
            });
        }
        Ok(signature)
    }
}

/// Recognize request and preflight rejections that precede the node's send queue.
/// AlreadyProcessed and unknown provider/internal errors still require reconciliation.
fn rpc_submission_rejection(response: &serde_json::Value, signature: &str) -> Option<SdkError> {
    let error = response.get("error")?;
    let code = error.get("code")?.as_i64()?;
    let reason = error.get("message")?.as_str()?;
    if response.get("result").is_some() {
        return None;
    }
    let message = reason.to_ascii_lowercase();
    if error["data"]["err"] == "AlreadyProcessed"
        || message.contains("alreadyprocessed")
        || message.contains("already processed")
        || message.contains("already been processed")
    {
        return None;
    }
    matches!(
        code,
        -32700
            | -32600
            | -32601
            | -32602
            | -32002
            | -32003
            | -32005
            | -32006
            | -32013
            | -32015
            | -32016
    )
    .then(|| SdkError::SubmissionRejected {
        signature: signature.into(),
        code,
        reason: reason.into(),
    })
}

/// Validate the feature owner, allocation, activation slot, and response bank together.
fn validate_v1_feature_response(response: &serde_json::Value) -> Result<(), SdkError> {
    let unavailable = || {
        SdkError::Validation(
            "Solana v1 is unavailable: feature account missing, malformed, or inactive".into(),
        )
    };
    if response.get("error").is_some() {
        return Err(unavailable());
    }
    let slot = response["result"]["context"]["slot"]
        .as_u64()
        .ok_or_else(unavailable)?;
    let account = &response["result"]["value"];
    if account["owner"].as_str() != Some(solana_sdk_ids::feature::ID.to_string().as_str())
        || account["executable"].as_bool() != Some(false)
    {
        return Err(unavailable());
    }
    if account["data"][1].as_str() != Some("base64") {
        return Err(unavailable());
    }
    let bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        account["data"][0].as_str().ok_or_else(unavailable)?,
    )
    .map_err(|_| unavailable())?;
    if bytes.len() < solana_feature_gate_interface::Feature::size_of() {
        return Err(unavailable());
    }
    let feature: solana_feature_gate_interface::Feature =
        bincode::deserialize(&bytes).map_err(|_| unavailable())?;
    if !matches!(feature.activated_at, Some(activation) if activation <= slot) {
        return Err(unavailable());
    }
    Ok(())
}

/// Measurements from simulation of the exact signed v1 transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionSimulation {
    pub slot: u64,
    pub units_consumed: Option<u64>,
    /// Loaded account data in bytes.
    pub loaded_accounts_data_size: Option<u64>,
    pub logs: Vec<String>,
}

// ── Transaction confirmation ─────────────────────────────────────────────────

/// Interval between polls while awaiting transaction confirmation.
const CONFIRMATION_POLL_INTERVAL: std::time::Duration = std::time::Duration::from_millis(800);

/// Hard cap on confirmation poll iterations (~90 s at the poll interval) — a
/// backstop for when block-height expiry cannot be observed (e.g. a
/// failed-over RPC node with a skewed view of the chain).
const MAX_CONFIRMATION_POLLS: u32 = 110;

/// Consecutive failed polls tolerated before the outcome is declared unknown.
const MAX_CONSECUTIVE_POLL_FAILURES: u32 = 3;

/// Consecutive over-bound block-height samples required before expiry may be
/// declared — a single reading can come from a forward-skewed RPC node.
const EXPIRY_HEIGHT_SAMPLES: u32 = 2;

/// Status of a submitted transaction, as reported by `getSignatureStatuses`.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionStatus {
    /// The slot the transaction was processed in.
    pub slot: u64,
    /// Confirmations since the transaction was processed; `None` once rooted.
    pub confirmations: Option<u64>,
    /// The on-chain error, present when the transaction landed but failed.
    pub err: Option<serde_json::Value>,
    /// Cluster confirmation level: `processed`, `confirmed`, or `finalized`.
    pub confirmation_status: Option<String>,
}

/// A successfully submitted transaction and the slot where it was confirmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmedTransaction {
    pub signature: String,
    pub slot: u64,
}

impl TransactionStatus {
    /// True once the cluster has voted the transaction to `confirmed` or beyond.
    pub fn is_confirmed(&self) -> bool {
        matches!(
            self.confirmation_status.as_deref(),
            Some("confirmed") | Some("finalized")
        )
    }
}

/// Clone client resources while sharing the mutable transaction-signing context.
///
/// Runtime signer or sponsorship updates through either clone are immediately visible
/// to the other, while each submission keeps the pair it captured before yielding.
impl Clone for LightconeClient {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            transaction_resources: self.transaction_resources,
            ws_config: self.ws_config.clone(),
            auth_credentials: self.auth_credentials.clone(),
            program_id: self.program_id,
            deposit_source: self.deposit_source.clone(),
            order_nonce: self.order_nonce.clone(),
            orderbook_rules: self.orderbook_rules.clone(),
            transaction_signing_context: self.transaction_signing_context.clone(),
            primary_rpc_url: self.primary_rpc_url.clone(),
            backup_rpc_url: self.backup_rpc_url.clone(),
            rpc_failover_state: self.rpc_failover_state.clone(),
            #[cfg(feature = "solana-rpc")]
            primary_solana_rpc_client: self.primary_solana_rpc_client.as_ref().map(|rpc_client| {
                SolanaRpcClient::new_with_commitment(
                    rpc_client.url(),
                    CommitmentConfig::confirmed(),
                )
            }),
            #[cfg(feature = "solana-rpc")]
            backup_solana_rpc_client: self.backup_solana_rpc_client.as_ref().map(|rpc_client| {
                SolanaRpcClient::new_with_commitment(
                    rpc_client.url(),
                    CommitmentConfig::confirmed(),
                )
            }),
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Builder
// ═════════════════════════════════════════════════════════════════════════════

pub struct LightconeClientBuilder {
    transaction_resources: Option<V1ResourceConfig>,
    base_url: String,
    ws_url: String,
    auth_credentials: Option<AuthCredentials>,
    program_id: Pubkey,
    deposit_source: DepositSource,
    signing_strategy: Option<SigningStrategy>,
    transaction_sponsorship_enabled: bool,
    primary_rpc_url: Option<String>,
    backup_rpc_url: Option<String>,
}

impl Default for LightconeClientBuilder {
    fn default() -> Self {
        let environment = LightconeEnv::default();
        Self {
            base_url: environment.api_url().to_string(),
            transaction_resources: None,
            ws_url: environment.ws_url().to_string(),
            auth_credentials: None,
            program_id: environment.program_id(),
            deposit_source: DepositSource::Global,
            signing_strategy: None,
            transaction_sponsorship_enabled: false,
            primary_rpc_url: Some(environment.rpc_url().to_string()),
            backup_rpc_url: None,
        }
    }
}

impl LightconeClientBuilder {
    /// Set explicit v1 resources for SOL planners and fluent submitters.
    /// Priority fees are total lamports per transaction.
    pub fn transaction_resources(mut self, resources: V1ResourceConfig) -> Self {
        self.transaction_resources = Some(resources);
        self
    }

    /// Set the deployment environment. Configures the API URL, WebSocket URL,
    /// RPC URL, and program ID for the given environment.
    ///
    /// Individual URL overrides (e.g. `.base_url()`) take precedence when
    /// called **after** `.env()`.
    pub fn env(mut self, environment: LightconeEnv) -> Self {
        self.base_url = environment.api_url().to_string();
        self.ws_url = environment.ws_url().to_string();
        self.program_id = environment.program_id();
        self.primary_rpc_url = Some(environment.rpc_url().to_string());
        self
    }

    pub fn base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    pub fn ws_url(mut self, url: &str) -> Self {
        self.ws_url = url.to_string();
        self
    }

    /// Pre-set authentication credentials on construction.
    pub fn auth(mut self, credentials: AuthCredentials) -> Self {
        self.auth_credentials = Some(credentials);
        self
    }

    /// Set a custom on-chain program ID (defaults to the canonical Lightcone program).
    pub fn program_id(mut self, program_id: Pubkey) -> Self {
        self.program_id = program_id;
        self
    }

    /// Set the default deposit source for orders, deposits, and withdrawals.
    /// Defaults to `DepositSource::Global`. Can be overridden per-call.
    pub fn deposit_source(mut self, source: DepositSource) -> Self {
        self.deposit_source = source;
        self
    }

    /// Set a native keypair for signing orders, cancels, and transactions.
    /// Intended for CLI tools, bots, and market makers.
    #[cfg(feature = "native-auth")]
    pub fn native_signer(mut self, keypair: solana_keypair::Keypair) -> Self {
        self.signing_strategy = Some(SigningStrategy::Native(Arc::new(keypair)));
        self
    }

    /// Set an external signer for signing orders, cancels, and transactions.
    /// Intended for browser wallet adapters. Implement the `ExternalSigner` trait
    /// to bridge your wallet adapter to the SDK. Unsponsored transaction
    /// submission requires its `wallet_address()` to return the fee payer.
    pub fn external_signer(mut self, signer: Arc<dyn ExternalSigner>) -> Self {
        self.signing_strategy = Some(SigningStrategy::WalletAdapter(signer));
        self
    }

    /// Set the initial client-wide Transaction Sponsorship Capability.
    ///
    /// The default is false. A true value is a trusted application assertion
    /// for external signing and is rejected if a local keypair submits a transaction.
    pub fn transaction_sponsorship(mut self, enabled: bool) -> Self {
        self.transaction_sponsorship_enabled = enabled;
        self
    }

    /// Set the primary Solana RPC URL for blockhash fetching, transaction
    /// submission, and on-chain reads (when `solana-rpc` feature is enabled).
    pub fn rpc_url(mut self, url: &str) -> Self {
        self.primary_rpc_url = Some(url.to_string());
        self
    }

    /// Set a backup Solana RPC URL for automatic failover. When the primary
    /// RPC returns infrastructure errors (connection failures, timeouts,
    /// 502/503/504), the SDK switches to this URL and stays on it until a
    /// 120 s cooldown elapses.
    pub fn backup_rpc_url(mut self, url: &str) -> Self {
        self.backup_rpc_url = Some(url.to_string());
        self
    }

    pub fn build(self) -> Result<LightconeClient, SdkError> {
        if let Some(resources) = self.transaction_resources {
            resources.validate()?;
        }
        Ok(LightconeClient {
            http: LightconeHttp::new(&self.base_url),
            transaction_resources: self.transaction_resources,
            ws_config: WsConfig {
                url: self.ws_url,
                ..WsConfig::default()
            },
            auth_credentials: Arc::new(RwLock::new(self.auth_credentials)),
            program_id: self.program_id,
            deposit_source: Arc::new(RwLock::new(self.deposit_source)),
            order_nonce: Arc::new(RwLock::new(None)),
            orderbook_rules: Arc::new(RwLock::new(HashMap::new())),
            transaction_signing_context: Arc::new(RwLock::new(TransactionSigningContext {
                strategy: self.signing_strategy,
                sponsorship_enabled: self.transaction_sponsorship_enabled,
            })),
            #[cfg(feature = "solana-rpc")]
            primary_solana_rpc_client: self.primary_rpc_url.as_ref().map(|url| {
                SolanaRpcClient::new_with_commitment(url.clone(), CommitmentConfig::confirmed())
            }),
            #[cfg(feature = "solana-rpc")]
            backup_solana_rpc_client: self.backup_rpc_url.as_ref().map(|url| {
                SolanaRpcClient::new_with_commitment(url.clone(), CommitmentConfig::confirmed())
            }),
            primary_rpc_url: self.primary_rpc_url,
            backup_rpc_url: self.backup_rpc_url,
            rpc_failover_state: Arc::new(RwLock::new(RpcFailoverState::new())),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "native")]
    fn test_transaction(payer: &Pubkey) -> V1Transaction {
        V1Transaction::compile(
            &[solana_system_interface::instruction::transfer(
                payer,
                &Pubkey::new_unique(),
                1,
            )],
            payer,
            &crate::program::transaction::test_context(),
        )
        .unwrap()
    }

    #[cfg(feature = "native")]
    use {
        solana_keypair::Keypair,
        solana_signer::Signer,
        std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpListener,
            sync::Notify,
        },
    };

    #[cfg(feature = "native")]
    struct RecordingExternalSigner {
        wallet: Pubkey,
        transaction_calls: Arc<AtomicUsize>,
    }

    #[cfg(feature = "native")]
    impl ExternalSigner for RecordingExternalSigner {
        fn wallet_address(&self) -> Option<Pubkey> {
            Some(self.wallet)
        }

        fn sign_message<'a>(
            &'a self,
            message: &'a [u8],
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + 'a>>
        {
            Box::pin(async move { Ok(message.to_vec()) })
        }

        fn sign_transaction<'a>(
            &'a self,
            tx_bytes: &'a [u8],
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + 'a>>
        {
            self.transaction_calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move { Ok(tx_bytes.to_vec()) })
        }
    }

    #[cfg(feature = "native")]
    async fn spawn_rpc_server(
        fee_lamports: Option<u64>,
        balance_lamports: Option<u64>,
        blockhash_gate: Option<(Arc<Notify>, Arc<Notify>)>,
    ) -> Result<(String, Arc<AtomicUsize>), std::io::Error> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        let attempts = Arc::new(AtomicUsize::new(0));
        let server_attempts = Arc::clone(&attempts);
        let latest_blockhash = solana_hash::Hash::new_unique().to_string();

        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    return;
                };
                let attempts = Arc::clone(&server_attempts);
                let latest_blockhash = latest_blockhash.clone();
                let blockhash_gate = blockhash_gate.clone();
                tokio::spawn(async move {
                    let mut request = [0_u8; 4096];
                    let _ = socket.read(&mut request).await;
                    attempts.fetch_add(1, Ordering::SeqCst);
                    let request = String::from_utf8_lossy(&request);
                    let (status, body) = if request.contains("getLatestBlockhash") {
                        (
                            "200 OK",
                            format!(
                                r#"{{"jsonrpc":"2.0","id":1,"result":{{"context":{{"slot":1}},"value":{{"blockhash":"{latest_blockhash}","lastValidBlockHeight":100}}}}}}"#
                            ),
                        )
                    } else if request.contains("getFeeForMessage") {
                        if let Some((started, release)) = blockhash_gate {
                            started.notify_one();
                            release.notified().await;
                        }

                        match fee_lamports {
                            Some(fee_lamports) => (
                                "200 OK",
                                format!(
                                    r#"{{"jsonrpc":"2.0","id":1,"result":{{"context":{{"slot":1}},"value":{fee_lamports}}}}}"#
                                ),
                            ),
                            None => (
                                "503 Service Unavailable",
                                r#"{"error":"fee unavailable"}"#.to_string(),
                            ),
                        }
                    } else if request.contains("getBalance") {
                        match balance_lamports {
                            Some(balance_lamports) => (
                                "200 OK",
                                format!(
                                    r#"{{"jsonrpc":"2.0","id":1,"result":{{"context":{{"slot":1}},"value":{balance_lamports}}}}}"#
                                ),
                            ),
                            None => (
                                "503 Service Unavailable",
                                r#"{"error":"balance unavailable"}"#.to_string(),
                            ),
                        }
                    } else if request.contains("getAccountInfo") {
                        ("200 OK", feature_response(Some(1), 10).to_string())
                    } else if request.contains("simulateTransaction") {
                        ("200 OK", serde_json::json!({"result":{"context":{"slot":10},"value":{"err":null,"logs":[]}}}).to_string())
                    } else {
                        (
                            "503 Service Unavailable",
                            r#"{"error":"unavailable"}"#.to_string(),
                        )
                    };
                    let response = format!(
                        "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    let _ = socket.write_all(response.as_bytes()).await;
                });
            }
        });

        Ok((format!("http://{address}"), attempts))
    }

    #[cfg(feature = "native")]
    async fn spawn_failing_rpc_server(
        fee_lamports: Option<u64>,
        balance_lamports: Option<u64>,
    ) -> Result<(String, Arc<AtomicUsize>), std::io::Error> {
        spawn_rpc_server(fee_lamports, balance_lamports, None).await
    }

    #[test]
    fn transaction_status_parses_full_envelope() {
        let value = serde_json::json!({
            "slot": 393226687u64,
            "confirmations": 12,
            "err": null,
            "confirmationStatus": "confirmed",
            "status": { "Ok": null }
        });
        let status: TransactionStatus = serde_json::from_value(value).unwrap();
        assert_eq!(status.slot, 393226687);
        assert_eq!(status.confirmations, Some(12));
        assert!(status.err.is_none());
        assert!(status.is_confirmed());
    }

    #[test]
    fn transaction_status_parses_rooted_failure() {
        let value = serde_json::json!({
            "slot": 5,
            "confirmations": null,
            "err": { "InstructionError": [0, { "Custom": 42 }] },
            "confirmationStatus": "finalized"
        });
        let status: TransactionStatus = serde_json::from_value(value).unwrap();
        assert_eq!(status.confirmations, None);
        assert!(status.err.is_some());
        assert!(status.is_confirmed());
    }

    #[test]
    fn processed_status_is_not_confirmed() {
        let value = serde_json::json!({
            "slot": 5,
            "confirmations": 0,
            "err": null,
            "confirmationStatus": "processed"
        });
        let status: TransactionStatus = serde_json::from_value(value).unwrap();
        assert!(!status.is_confirmed());
    }

    #[test]
    fn signature_statuses_envelope_parses_unseen_entries() {
        let value = serde_json::json!([
            null,
            { "slot": 9, "confirmations": 1, "err": null, "confirmationStatus": "confirmed" }
        ]);
        let statuses: Vec<Option<TransactionStatus>> = serde_json::from_value(value).unwrap();
        assert!(statuses[0].is_none());
        assert!(statuses[1].as_ref().unwrap().is_confirmed());
    }

    #[tokio::test]
    async fn transaction_sponsorship_defaults_false_and_is_shared_by_clones(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = LightconeClient::builder().build()?;
        assert!(!client.transaction_sponsorship_enabled().await);

        let clone = client.clone();
        clone.set_transaction_sponsorship_enabled(true).await;

        assert!(client.transaction_sponsorship_enabled().await);
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn prepared_submission_returns_typed_fee_error_before_signing(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (rpc_url, attempts) = spawn_failing_rpc_server(Some(5_000), Some(4_999)).await?;
        let keypair = Keypair::new();
        let payer = keypair.pubkey();
        let client = LightconeClient::builder()
            .rpc_url(&rpc_url)
            .native_signer(keypair)
            .build()?;
        let transaction = test_transaction(&payer);

        let error = client
            .sign_and_submit_prepared_tx_confirmed_with_slot(transaction)
            .await
            .unwrap_err();

        assert!(matches!(
            error,
            SdkError::InsufficientSolForTransactionFees {
                available_lamports: 4_999,
                required_lamports: 5_000,
            }
        ));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn ordinary_submission_routes_through_the_typed_funding_guard(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (rpc_url, attempts) = spawn_failing_rpc_server(Some(5_000), Some(4_999)).await?;
        let keypair = Keypair::new();
        let payer = keypair.pubkey();
        let client = LightconeClient::builder()
            .rpc_url(&rpc_url)
            .native_signer(keypair)
            .build()?;
        let transaction = test_transaction(&payer);

        let error = client.sign_and_submit_tx(transaction).await.unwrap_err();

        assert!(matches!(
            error,
            SdkError::InsufficientSolForTransactionFees {
                available_lamports: 4_999,
                required_lamports: 5_000,
            }
        ));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn sponsored_local_keypair_submission_is_rejected_before_rpc(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let keypair = Keypair::new();
        let payer = Pubkey::new_unique();
        let client = LightconeClient::builder()
            .native_signer(keypair)
            .transaction_sponsorship(true)
            .build()?;
        let transaction = test_transaction(&payer);

        let error = client
            .sign_and_submit_prepared_tx_confirmed_with_slot(transaction)
            .await
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "Validation error: transaction sponsorship is not supported with local-keypair signing"
        );
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn sponsored_prepared_external_signer_may_differ_from_fee_payer(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (rpc_url, attempts) = spawn_failing_rpc_server(Some(5_000), Some(5_000)).await?;
        let transaction_calls = Arc::new(AtomicUsize::new(0));
        let signer = RecordingExternalSigner {
            wallet: Pubkey::new_unique(),
            transaction_calls: Arc::clone(&transaction_calls),
        };
        let client = LightconeClient::builder()
            .rpc_url(&rpc_url)
            .external_signer(Arc::new(signer))
            .transaction_sponsorship(true)
            .build()?;
        let payer = Pubkey::new_unique();
        let transaction = test_transaction(&payer);

        let error = client
            .sign_and_submit_prepared_tx_confirmed_with_slot(transaction)
            .await
            .unwrap_err();
        assert_eq!(
            transaction_calls.load(Ordering::SeqCst),
            1,
            "unexpected error before signing: {error}"
        );
        assert_eq!(
            attempts.load(Ordering::SeqCst),
            1,
            "unexpected error before submission: {error}"
        );
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn ordinary_submission_rejects_invalid_signing_context_before_rpc(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let keypair = Keypair::new();
        let payer = keypair.pubkey();
        let client = LightconeClient::builder()
            .rpc_url("http://127.0.0.1:1")
            .native_signer(keypair)
            .transaction_sponsorship(true)
            .build()?;
        let transaction = test_transaction(&payer);

        let error = client.sign_and_submit_tx(transaction).await.unwrap_err();
        assert_eq!(
            error.to_string(),
            "Validation error: transaction sponsorship is not supported with local-keypair signing"
        );

        let keypair = Keypair::new();
        let client = LightconeClient::builder()
            .rpc_url("http://127.0.0.1:1")
            .native_signer(keypair)
            .build()?;
        let wrong_payer = Pubkey::new_unique();
        let transaction = test_transaction(&wrong_payer);
        let error = client.sign_and_submit_tx(transaction).await.unwrap_err();
        assert_eq!(
            error.to_string(),
            "Validation error: signing strategy does not control transaction fee payer"
        );
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn generic_preflight_continues_when_funded_or_evidence_is_unavailable(
    ) -> Result<(), Box<dyn std::error::Error>> {
        for (fee_lamports, balance_lamports) in [
            (Some(5_000), Some(5_000)),
            (Some(5_000), Some(5_001)),
            (None, Some(5_000)),
            (Some(5_000), None),
        ] {
            let (rpc_url, _attempts) =
                spawn_failing_rpc_server(fee_lamports, balance_lamports).await?;
            let keypair = Keypair::new();
            let payer = keypair.pubkey();
            let client = LightconeClient::builder()
                .rpc_url(&rpc_url)
                .native_signer(keypair)
                .build()?;
            let strategy = client.signing_strategy().await.unwrap();
            let transaction = test_transaction(&payer);

            client
                .preflight_transaction_fee_funding(&transaction, &strategy, false)
                .await?;
        }
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn submission_preflight_keeps_its_captured_sponsorship_value(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let blockhash_started = Arc::new(Notify::new());
        let release_blockhash = Arc::new(Notify::new());
        let (rpc_url, attempts) = spawn_rpc_server(
            Some(5_000),
            Some(4_999),
            Some((
                Arc::clone(&blockhash_started),
                Arc::clone(&release_blockhash),
            )),
        )
        .await?;
        let keypair = Keypair::new();
        let payer = keypair.pubkey();
        let client = LightconeClient::builder()
            .rpc_url(&rpc_url)
            .native_signer(keypair)
            .build()?;
        let transaction = test_transaction(&payer);
        let submission_client = client.clone();
        let submission = submission_client.sign_and_submit_tx(transaction);
        let change_sponsorship = async {
            blockhash_started.notified().await;
            client.set_transaction_sponsorship_enabled(true).await;
            release_blockhash.notify_one();
        };
        let (result, ()) = tokio::join!(submission, change_sponsorship);
        let error = result.unwrap_err();

        assert!(matches!(
            error,
            SdkError::InsufficientSolForTransactionFees {
                available_lamports: 4_999,
                required_lamports: 5_000,
            }
        ));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn prepared_submission_transport_failure_is_sent_once(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (primary_rpc_url, primary_attempts) =
            spawn_failing_rpc_server(Some(5_000), Some(5_000)).await?;
        let (backup_rpc_url, backup_attempts) =
            spawn_failing_rpc_server(Some(5_000), Some(5_000)).await?;
        let keypair = Keypair::new();
        let payer = keypair.pubkey();
        let client = LightconeClient::builder()
            .rpc_url(&primary_rpc_url)
            .backup_rpc_url(&backup_rpc_url)
            .native_signer(keypair)
            .build()?;
        let transaction = test_transaction(&payer);

        assert!(client
            .sign_and_submit_prepared_tx_confirmed_with_slot(transaction)
            .await
            .is_err());
        assert_eq!(primary_attempts.load(Ordering::SeqCst), 6);
        assert_eq!(backup_attempts.load(Ordering::SeqCst), 0);
        Ok(())
    }

    fn feature_response(activation: Option<u64>, slot: u64) -> serde_json::Value {
        let mut data = bincode::serialize(&solana_feature_gate_interface::Feature {
            activated_at: activation,
        })
        .unwrap();
        data.resize(solana_feature_gate_interface::Feature::size_of(), 0);
        serde_json::json!({"jsonrpc":"2.0","id":1,"result":{
            "context":{"slot":slot},"value":{
                "owner": solana_sdk_ids::feature::ID.to_string(), "executable":false,
                "data":[base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data),"base64"]
            }
        }})
    }

    #[test]
    fn feature_activation_is_validated_in_the_response_bank() {
        assert!(validate_v1_feature_response(&feature_response(Some(10), 10)).is_ok());
        assert!(validate_v1_feature_response(&feature_response(None, 10)).is_err());
        assert!(validate_v1_feature_response(&feature_response(Some(11), 10)).is_err());
        assert!(validate_v1_feature_response(
            &serde_json::json!({"result":{"context":{"slot":10},"value":null}})
        )
        .is_err());
        let mut foreign = feature_response(Some(10), 10);
        foreign["result"]["value"]["owner"] = serde_json::json!(Pubkey::new_unique().to_string());
        assert!(validate_v1_feature_response(&foreign).is_err());
        let mut malformed = feature_response(Some(10), 10);
        malformed["result"]["value"]["data"][0] = serde_json::json!("AQ==");
        assert!(validate_v1_feature_response(&malformed).is_err());
    }

    #[tokio::test]
    async fn planners_require_an_explicit_resource_policy() {
        let client = LightconeClient::builder()
            .rpc_url("http://127.0.0.1:1")
            .build()
            .unwrap();
        assert!(client
            .transaction_context()
            .await
            .unwrap_err()
            .to_string()
            .contains("resources are required"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn signed_v1_submission_simulates_and_confirms_the_exact_message() {
        use base64::Engine;
        let payer = Keypair::new();
        let tx = test_transaction(&payer.pubkey());
        let signed = tx.sign(&[&payer]).unwrap();
        let expected_bytes = signed.to_wire_bytes().unwrap();
        let expected_signature = signed.as_versioned().signatures[0].to_string();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server_signature = expected_signature.clone();
        let server = tokio::spawn(async move {
            let mut calls = Vec::new();
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                let body = loop {
                    let mut buffer = [0_u8; 8192];
                    let count = socket.read(&mut buffer).await.unwrap();
                    assert!(count > 0);
                    bytes.extend_from_slice(&buffer[..count]);
                    if let Some(split) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                        let header = String::from_utf8_lossy(&bytes[..split]).to_lowercase();
                        let length: usize = header
                            .lines()
                            .find_map(|line| line.strip_prefix("content-length:"))
                            .unwrap()
                            .trim()
                            .parse()
                            .unwrap();
                        if bytes.len() >= split + 4 + length {
                            break serde_json::from_slice::<serde_json::Value>(
                                &bytes[split + 4..split + 4 + length],
                            )
                            .unwrap();
                        }
                    }
                };
                let method = body["method"].as_str().unwrap();
                calls.push(method.to_string());
                let response = match method {
                    "getFeeForMessage" => {
                        let bytes = base64::engine::general_purpose::STANDARD
                            .decode(body["params"][0].as_str().unwrap())
                            .unwrap();
                        let message: solana_message::VersionedMessage =
                            wincode::deserialize(&bytes).unwrap();
                        assert!(matches!(message, solana_message::VersionedMessage::V1(_)));
                        serde_json::json!({"result":{"value":5000}})
                    }
                    "getBalance" => serde_json::json!({"result":{"value":10000000}}),
                    "getAccountInfo" => feature_response(Some(1), 10),
                    "simulateTransaction" | "sendTransaction" => {
                        let bytes = base64::engine::general_purpose::STANDARD
                            .decode(body["params"][0].as_str().unwrap())
                            .unwrap();
                        assert_eq!(bytes, expected_bytes);
                        if method == "simulateTransaction" {
                            assert_eq!(body["params"][1]["sigVerify"], true);
                            assert_eq!(body["params"][1]["replaceRecentBlockhash"], false);
                            serde_json::json!({"result":{"context":{"slot":10},"value":{"err":null,"logs":[],"unitsConsumed":100,"loadedAccountsDataSize":128}}})
                        } else {
                            assert_eq!(body["params"][1]["skipPreflight"], false);
                            assert_eq!(body["params"][1]["maxRetries"], 0);
                            serde_json::json!({"result":server_signature})
                        }
                    }
                    "getSignatureStatuses" => {
                        serde_json::json!({"result":{"value":[{"slot":42,"confirmations":1,"err":null,"confirmationStatus":"confirmed"}]}})
                    }
                    other => panic!("unexpected RPC {other}"),
                };
                let response = response.to_string();
                socket.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}", response.len()).as_bytes()).await.unwrap();
                if method == "getSignatureStatuses" {
                    break calls;
                }
            }
        });
        let client = LightconeClient::builder()
            .rpc_url(&format!("http://{address}"))
            .native_signer(payer)
            .build()
            .unwrap();
        let confirmed = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            client.sign_and_submit_prepared_tx_confirmed_with_slot(tx),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(confirmed.signature, expected_signature);
        assert_eq!(confirmed.slot, 42);
        let calls = server.await.unwrap();
        assert_eq!(
            calls
                .iter()
                .filter(|method| *method == "sendTransaction")
                .count(),
            1
        );
        assert!(!calls.iter().any(|method| method == "getLatestBlockhash"));
    }

    #[test]
    fn rpc_rejections_exclude_existing_signatures_and_ambiguous_errors() {
        for code in [
            -32700, -32600, -32601, -32602, -32002, -32003, -32005, -32006, -32013, -32015, -32016,
        ] {
            let response = serde_json::json!({"error":{"code":code,"message":"rejected","data":{"err":"BlockhashNotFound"}}});
            assert!(
                matches!(rpc_submission_rejection(&response, "signature"), Some(SdkError::SubmissionRejected { code: actual, .. }) if actual == code)
            );
        }
        for response in [
            serde_json::json!({"error":{"code":-32002,"message":"failed","data":{"err":"AlreadyProcessed"}}}),
            serde_json::json!({"error":{"code":-32002,"message":"Transaction has already been processed"}}),
            serde_json::json!({"error":{"code":-32603,"message":"internal failure"}}),
            serde_json::json!({"error":{"code":-32099,"message":"provider failure"}}),
            serde_json::json!({"error":{"code":-32002}}),
            serde_json::json!({"result":"signature","error":{"code":-32002,"message":"conflicting response"}}),
            serde_json::json!({"result":null,"error":{"code":-32002,"message":"conflicting response"}}),
            serde_json::json!({"result":"signature","error":null}),
        ] {
            assert!(rpc_submission_rejection(&response, "signature").is_none());
        }
    }

    #[cfg(feature = "native")]
    async fn v1_rpc_test_server(
        handler: impl Fn(&serde_json::Value) -> (u16, Option<serde_json::Value>) + Send + 'static,
    ) -> (
        String,
        Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
        tokio::task::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        let recorded = calls.clone();
        let task = tokio::spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut bytes = Vec::new();
                let request: serde_json::Value = loop {
                    let mut buffer = [0; 8192];
                    let count = socket.read(&mut buffer).await.unwrap();
                    assert!(count > 0);
                    bytes.extend_from_slice(&buffer[..count]);
                    if let Some(split) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&bytes[..split]).to_lowercase();
                        let length: usize = headers
                            .lines()
                            .find_map(|line| line.strip_prefix("content-length:"))
                            .unwrap()
                            .trim()
                            .parse()
                            .unwrap();
                        if bytes.len() >= split + 4 + length {
                            break serde_json::from_slice(&bytes[split + 4..split + 4 + length])
                                .unwrap();
                        }
                    }
                };
                recorded.lock().unwrap().push(request.clone());
                let (status, response) = handler(&request);
                if let Some(response) = response {
                    let body = response.to_string();
                    socket.write_all(format!("HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
                }
            }
        });
        (url, calls, task)
    }

    #[cfg(feature = "native")]
    fn v1_read_response(request: &serde_json::Value) -> serde_json::Value {
        match request["method"].as_str().unwrap() {
            "getAccountInfo" => {
                assert_eq!(
                    request["params"][0],
                    "txv1aq4pp281K9um3tnPgkfX8UqtFT6wcVW3hNezGLL"
                );
                feature_response(Some(1), 10)
            }
            "simulateTransaction" => {
                assert_eq!(request["params"][1]["sigVerify"], true);
                assert_eq!(request["params"][1]["replaceRecentBlockhash"], false);
                serde_json::json!({"result":{"context":{"slot":10},"value":{"err":null,"logs":[]}}})
            }
            method => panic!("unexpected read {method}"),
        }
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn send_errors_are_classified_once_without_leaking_transport_credentials() {
        let payer = Keypair::new();
        let signed = test_transaction(&payer.pubkey()).sign(&[&payer]).unwrap();
        for (send_response, rejected) in [
            (None, false),
            (
                Some(
                    serde_json::json!({"error":{"code":-32002,"message":"preflight rejected","data":{"err":"BlockhashNotFound"}}}),
                ),
                true,
            ),
            (
                Some(
                    serde_json::json!({"error":{"code":-32002,"message":"already processed","data":{"err":"AlreadyProcessed"}}}),
                ),
                false,
            ),
            (
                Some(serde_json::json!({"error":{"code":-32603,"message":"internal error"}})),
                false,
            ),
            (
                Some(serde_json::json!({"result":"wrong signature","debug":"do-not-expose"})),
                false,
            ),
        ] {
            let (url, calls, server) = v1_rpc_test_server(move |request| {
                if request["method"] == "sendTransaction" {
                    assert_eq!(request["params"][1]["maxRetries"], 0);
                    assert_eq!(request["params"][1]["skipPreflight"], false);
                    (200, send_response.clone())
                } else {
                    (200, Some(v1_read_response(request)))
                }
            })
            .await;
            let client = LightconeClient::builder()
                .rpc_url(&format!("{url}/?api-key=do-not-expose"))
                .build()
                .unwrap();
            let error = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                client.submit_signed_transaction(&signed),
            )
            .await
            .unwrap()
            .unwrap_err();
            assert!(!format!("{error:?}").contains("do-not-expose"));
            if rejected {
                assert!(matches!(
                    error,
                    SdkError::SubmissionRejected { code: -32002, .. }
                ));
            } else {
                assert!(
                    matches!(error, SdkError::SubmissionUnknown { ref signature, last_valid_block_height: 100, .. } if signature == &signed.as_versioned().signatures[0].to_string())
                );
            }
            assert_eq!(
                calls
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|call| call["method"] == "sendTransaction")
                    .count(),
                1
            );
            server.abort();
        }
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn activation_and_simulation_reads_fail_over_without_changing_the_message() {
        let payer = Keypair::new();
        let signed = test_transaction(&payer.pubkey()).sign(&[&payer]).unwrap();
        for failing_method in ["getAccountInfo", "simulateTransaction"] {
            let (primary, primary_calls, primary_server) = v1_rpc_test_server(move |request| {
                if request["method"] == failing_method {
                    (503, Some(serde_json::json!({"error":"unavailable"})))
                } else {
                    (200, Some(v1_read_response(request)))
                }
            })
            .await;
            let (backup, backup_calls, backup_server) =
                v1_rpc_test_server(|request| (200, Some(v1_read_response(request)))).await;
            let client = LightconeClient::builder()
                .rpc_url(&primary)
                .backup_rpc_url(&backup)
                .build()
                .unwrap();
            tokio::time::timeout(
                std::time::Duration::from_secs(10),
                client.simulate_transaction(&signed),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(
                primary_calls
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|call| call["method"] == failing_method)
                    .count(),
                2
            );
            let backup_calls = backup_calls.lock().unwrap();
            assert!(backup_calls
                .iter()
                .any(|call| call["method"] == failing_method));
            let simulation = backup_calls
                .iter()
                .find(|call| call["method"] == "simulateTransaction")
                .unwrap();
            assert_eq!(
                simulation["params"][0],
                base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    signed.to_wire_bytes().unwrap()
                )
            );
            primary_server.abort();
            backup_server.abort();
        }
    }
}
