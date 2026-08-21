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
use crate::domain::price_history::client::PriceHistoryClient;
use crate::domain::referral::client::Referrals;
use crate::domain::trade::client::Trades;
use crate::env::LightconeEnv;
use crate::error::SdkError;
use crate::http::retry::RetryPolicy;
use crate::http::LightconeHttp;
use crate::rpc::Rpc;
use crate::rpc_failover::{
    is_infrastructure_error_http, with_failover, ActiveRpc, RpcFailoverState,
};
use crate::shared::signing::{ExternalSigner, SigningStrategy};
use crate::shared::OrderbookRules;
use crate::shared::{DepositSource, PubkeyStr};
use crate::ws::WsConfig;

#[cfg(feature = "solana-rpc")]
use solana_client::nonblocking::rpc_client::RpcClient as SolanaRpcClient;
#[cfg(feature = "solana-rpc")]
use solana_commitment_config::CommitmentConfig;

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

/// The primary entry point for the Lightcone SDK.
///
/// Provides nested sub-client accessors for each domain:
/// `client.markets()`, `client.orders()`, etc.
///
/// Market data remains stateless. Immutable orderbook trading rules are cached
/// because every signed order requires them.
pub struct LightconeClient {
    pub(crate) http: LightconeHttp,
    pub(crate) ws_config: WsConfig,
    pub(crate) auth_credentials: Arc<RwLock<Option<AuthCredentials>>>,
    /// On-chain program ID (defaults to the canonical Lightcone program).
    pub(crate) program_id: Pubkey,
    /// Default deposit source for orders, deposits, and withdrawals.
    /// Per-call overrides take priority over this setting.
    pub(crate) deposit_source: Arc<RwLock<DepositSource>>,
    /// Signing strategy for orders, cancels, and transactions.
    /// `None` means signing must be done manually (power-user mode).
    pub(crate) signing_strategy: Arc<RwLock<Option<SigningStrategy>>>,
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

    /// Get the current signing strategy, if set.
    pub async fn signing_strategy(&self) -> Option<SigningStrategy> {
        self.signing_strategy.read().await.clone()
    }

    /// Set the signing strategy at runtime.
    ///
    /// Common use: set during login when the wallet type is known.
    pub async fn set_signing_strategy(&self, strategy: SigningStrategy) {
        *self.signing_strategy.write().await = Some(strategy);
    }

    /// Clear the signing strategy (e.g. on logout).
    pub async fn clear_signing_strategy(&self) {
        *self.signing_strategy.write().await = None;
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

    /// Attach a fresh confirmed blockhash and return the message's live fee in lamports.
    ///
    /// `getFeeForMessage` returning null is an unavailable estimate, not a zero
    /// fee. Callers must therefore fail closed rather than falling back to a
    /// configured reserve floor.
    pub async fn prepare_and_estimate_transaction_fee(
        &self,
        transaction: &mut solana_transaction::Transaction,
    ) -> Result<u64, SdkError> {
        transaction.message.recent_blockhash = self.get_latest_blockhash().await?;
        self.estimate_prepared_transaction_fee(transaction).await
    }

    /// Return the live fee in lamports for a message that already carries the
    /// blockhash the caller will sign. This never mutates the prepared transaction;
    /// a null RPC estimate fails closed rather than becoming a zero fee.
    pub async fn estimate_prepared_transaction_fee(
        &self,
        transaction: &solana_transaction::Transaction,
    ) -> Result<u64, SdkError> {
        if transaction.message.recent_blockhash == solana_hash::Hash::default() {
            return Err(SdkError::Validation(
                "prepared transaction is missing a recent blockhash".into(),
            ));
        }
        let message = bincode::serialize(&transaction.message)
            .map_err(|error| SdkError::Other(format!("message serialization failed: {error}")))?;
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
    /// transaction's blockhash cannot be proven (e.g. an external signer may
    /// have replaced it) — expiry is then never reported and only the poll
    /// cap ends the wait. Terminal outcomes:
    ///
    /// - [`SdkError::TransactionFailed`] — the transaction landed but errored
    ///   on-chain; resubmitting the same transaction would fail again.
    /// - [`SdkError::TransactionExpired`] — the chain moved past
    ///   `last_valid_block_height` on consecutive height samples and a
    ///   history-searching status check still cannot see the signature; the
    ///   transaction can never land and is safe to resubmit.
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
                                    // landed transactions, and
                                    // `TransactionExpired` promises resubmit
                                    // safety. On a failed lookup, keep polling
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
    /// Fetches a recent blockhash automatically. The caller does not need to set it.
    /// Returns as soon as the RPC accepts the transaction — inclusion is not
    /// awaited. When follow-up work depends on this transaction's on-chain
    /// effects, use [`Self::sign_and_submit_tx_confirmed`] instead.
    ///
    /// - **Native**: signs locally with keypair, submits via RPC `sendTransaction`
    /// - **WalletAdapter**: signs via external signer, submits via RPC `sendTransaction`
    /// - **Privy**: serializes unsigned tx to base64, sends to backend for signing + submission
    pub async fn sign_and_submit_tx(
        &self,
        tx: solana_transaction::Transaction,
    ) -> Result<String, SdkError> {
        let strategy = self.signing_strategy().await.ok_or_else(|| {
            SdkError::Validation("signing strategy is not set on the client".into())
        })?;
        let (signature, _last_valid_block_height) =
            self.sign_and_submit_tx_inner(tx, strategy).await?;
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
    /// Expiry ([`SdkError::TransactionExpired`]) is only ever reported when
    /// the submitted transaction provably still carries the blockhash fetched
    /// here: always true for the native strategy, verified against the signed
    /// bytes for wallet-adapter signers (which may re-blockhash before
    /// signing). When unproven, a dropped transaction surfaces as
    /// [`SdkError::ConfirmationTimeout`] at the poll cap instead.
    pub async fn sign_and_submit_tx_confirmed(
        &self,
        tx: solana_transaction::Transaction,
    ) -> Result<String, SdkError> {
        self.sign_and_submit_tx_confirmed_with_slot(tx)
            .await
            .map(|confirmed| confirmed.signature)
    }

    /// Sign and submit a transaction, wait for confirmed commitment, and
    /// return both its signature and processing slot.
    pub async fn sign_and_submit_tx_confirmed_with_slot(
        &self,
        tx: solana_transaction::Transaction,
    ) -> Result<ConfirmedTransaction, SdkError> {
        let strategy = self.signing_strategy().await.ok_or_else(|| {
            SdkError::Validation("signing strategy is not set on the client".into())
        })?;
        let (signature, last_valid_block_height) =
            self.sign_and_submit_tx_inner(tx, strategy).await?;
        let status = self
            .confirm_signature_status(&signature, last_valid_block_height)
            .await?;
        Ok(ConfirmedTransaction {
            signature,
            slot: status.slot,
        })
    }

    /// Sign and confirm a transaction whose exact message was already used for
    /// fee estimation.
    ///
    /// Unlike the generic submission path, this preserves the prepared recent
    /// blockhash and rejects an external signer that changes any message field.
    /// Confirmation uses the bounded poll cap without a block-height expiry
    /// claim because the planner retained no `lastValidBlockHeight` metadata.
    pub async fn sign_and_submit_prepared_tx_confirmed_with_slot(
        &self,
        tx: solana_transaction::Transaction,
    ) -> Result<ConfirmedTransaction, SdkError> {
        if tx.message.recent_blockhash == solana_hash::Hash::default() {
            return Err(SdkError::Validation(
                "prepared transaction is missing a recent blockhash".into(),
            ));
        }
        let strategy = self.signing_strategy().await.ok_or_else(|| {
            SdkError::Validation("signing strategy is not set on the client".into())
        })?;
        let signature = self.sign_and_submit_prepared_tx_inner(tx, strategy).await?;
        let status = self.confirm_signature_status(&signature, None).await?;
        Ok(ConfirmedTransaction {
            signature,
            slot: status.slot,
        })
    }

    /// Shared submit path: sign, send, and return the signature together with
    /// the `lastValidBlockHeight` of the blockhash the submitted wire bytes
    /// are known to carry — `None` when that cannot be proven (an external
    /// signer replaced the blockhash, or the bytes could not be inspected).
    async fn sign_and_submit_tx_inner(
        &self,
        mut tx: solana_transaction::Transaction,
        strategy: SigningStrategy,
    ) -> Result<(String, Option<u64>), SdkError> {
        let (blockhash, last_valid_block_height) = self.get_latest_blockhash_with_height().await?;
        tx.message.recent_blockhash = blockhash;

        match strategy {
            #[cfg(feature = "native-auth")]
            SigningStrategy::Native(keypair) => {
                tx.try_sign(&[keypair.as_ref()], blockhash)
                    .map_err(|error| SdkError::Signing(error.to_string()))?;
                let signature = self.send_transaction_rpc(&tx).await?;
                Ok((signature, Some(last_valid_block_height)))
            }
            SigningStrategy::WalletAdapter(signer) => {
                let tx_bytes = bincode::serialize(&tx).map_err(|error| {
                    SdkError::Other(format!("tx serialization failed: {error}"))
                })?;
                let signed_bytes = signer
                    .sign_transaction(&tx_bytes)
                    .await
                    .map_err(crate::shared::signing::classify_signer_error)?;
                // External signers may re-blockhash before signing; only trust
                // the expiry bound when the signed bytes still carry the
                // blockhash fetched above.
                let signed_blockhash_unchanged =
                    bincode::deserialize::<solana_transaction::Transaction>(&signed_bytes)
                        .map(|signed_tx| signed_tx.message.recent_blockhash == blockhash)
                        .unwrap_or(false);
                if !signed_blockhash_unchanged {
                    tracing::warn!(
                        "Signer changed the transaction blockhash; confirming without an expiry bound"
                    );
                }
                // The signer returns fully signed tx bytes — send via base64
                let base64_tx = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    &signed_bytes,
                );
                let signature = self.send_raw_transaction_rpc(&base64_tx).await?;
                Ok((
                    signature,
                    signed_blockhash_unchanged.then_some(last_valid_block_height),
                ))
            }
        }
    }

    /// Sign and submit a prepared message without fetching or replacing its blockhash.
    ///
    /// Native signing preserves the message by construction. Wallet-adapter
    /// bytes are re-decoded and compared before submission; Privy is excluded
    /// because its final wire message is outside the SDK's inspection boundary.
    async fn sign_and_submit_prepared_tx_inner(
        &self,
        tx: solana_transaction::Transaction,
        strategy: SigningStrategy,
    ) -> Result<String, SdkError> {
        match strategy {
            #[cfg(feature = "native-auth")]
            SigningStrategy::Native(keypair) => {
                let mut tx = tx;
                let blockhash = tx.message.recent_blockhash;
                tx.try_sign(&[keypair.as_ref()], blockhash)
                    .map_err(|error| SdkError::Signing(error.to_string()))?;
                self.send_transaction_rpc(&tx).await
            }
            SigningStrategy::WalletAdapter(signer) => {
                let tx_bytes = bincode::serialize(&tx).map_err(|error| {
                    SdkError::Other(format!("tx serialization failed: {error}"))
                })?;
                let signed_bytes = signer
                    .sign_transaction(&tx_bytes)
                    .await
                    .map_err(crate::shared::signing::classify_signer_error)?;
                validate_prepared_signed_transaction(&tx, &signed_bytes)?;
                let base64_tx = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    &signed_bytes,
                );
                self.send_raw_transaction_rpc(&base64_tx).await
            }
        }
    }

    /// Submit a signed transaction via JSON-RPC `sendTransaction`.
    #[cfg(feature = "native-auth")]
    async fn send_transaction_rpc(
        &self,
        tx: &solana_transaction::Transaction,
    ) -> Result<String, SdkError> {
        let tx_bytes = bincode::serialize(tx)
            .map_err(|error| SdkError::Other(format!("tx serialization failed: {error}")))?;
        let base64_tx =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &tx_bytes);
        self.send_raw_transaction_rpc(&base64_tx).await
    }

    /// Submit a base64-encoded signed transaction via JSON-RPC `sendTransaction`.
    async fn send_raw_transaction_rpc(&self, base64_tx: &str) -> Result<String, SdkError> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [
                base64_tx,
                {
                    "encoding": "base64",
                    "preflightCommitment": "confirmed"
                }
            ]
        });

        let response: serde_json::Value = self.rpc_call_with_failover(&body).await?;

        if let Some(error) = response.get("error") {
            return Err(SdkError::Other(format!("RPC error: {error}")));
        }

        response["result"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| SdkError::Other("no signature in sendTransaction response".into()))
    }
}

/// Reject external signed bytes unless their message exactly matches preflight.
///
/// Signatures may differ, but fee, accounts, instructions, and blockhash are the
/// authority used by the planner and must survive the wallet boundary unchanged.
fn validate_prepared_signed_transaction(
    prepared: &solana_transaction::Transaction,
    signed_bytes: &[u8],
) -> Result<(), SdkError> {
    let signed = bincode::deserialize::<solana_transaction::Transaction>(signed_bytes)
        .map_err(|error| SdkError::Signing(format!("signed transaction is invalid: {error}")))?;
    if signed.message != prepared.message {
        return Err(SdkError::Validation(
            "wallet changed the fee-prepared transaction message".into(),
        ));
    }
    Ok(())
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

impl Clone for LightconeClient {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            ws_config: self.ws_config.clone(),
            auth_credentials: self.auth_credentials.clone(),
            program_id: self.program_id,
            deposit_source: self.deposit_source.clone(),
            order_nonce: self.order_nonce.clone(),
            orderbook_rules: self.orderbook_rules.clone(),
            signing_strategy: self.signing_strategy.clone(),
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
    base_url: String,
    ws_url: String,
    auth_credentials: Option<AuthCredentials>,
    program_id: Pubkey,
    deposit_source: DepositSource,
    signing_strategy: Option<SigningStrategy>,
    primary_rpc_url: Option<String>,
    backup_rpc_url: Option<String>,
}

impl Default for LightconeClientBuilder {
    fn default() -> Self {
        let environment = LightconeEnv::default();
        Self {
            base_url: environment.api_url().to_string(),
            ws_url: environment.ws_url().to_string(),
            auth_credentials: None,
            program_id: environment.program_id(),
            deposit_source: DepositSource::Global,
            signing_strategy: None,
            primary_rpc_url: Some(environment.rpc_url().to_string()),
            backup_rpc_url: None,
        }
    }
}

impl LightconeClientBuilder {
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
    /// to bridge your wallet adapter to the SDK.
    pub fn external_signer(mut self, signer: Arc<dyn ExternalSigner>) -> Self {
        self.signing_strategy = Some(SigningStrategy::WalletAdapter(signer));
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
        Ok(LightconeClient {
            http: LightconeHttp::new(&self.base_url),
            ws_config: WsConfig {
                url: self.ws_url,
                ..WsConfig::default()
            },
            auth_credentials: Arc::new(RwLock::new(self.auth_credentials)),
            program_id: self.program_id,
            deposit_source: Arc::new(RwLock::new(self.deposit_source)),
            order_nonce: Arc::new(RwLock::new(None)),
            orderbook_rules: Arc::new(RwLock::new(HashMap::new())),
            signing_strategy: Arc::new(RwLock::new(self.signing_strategy)),
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

    #[test]
    /// Rejects a signer-replaced blockhash before any prepared bytes are sent.
    fn prepared_submission_rejects_a_signer_blockhash_change() {
        let payer = Pubkey::new_unique();
        let mut prepared = solana_transaction::Transaction::new_with_payer(&[], Some(&payer));
        prepared.message.recent_blockhash = solana_hash::Hash::new_unique();
        let unchanged = bincode::serialize(&prepared).unwrap();
        assert!(validate_prepared_signed_transaction(&prepared, &unchanged).is_ok());

        let mut changed = prepared.clone();
        changed.message.recent_blockhash = solana_hash::Hash::new_unique();
        let changed = bincode::serialize(&changed).unwrap();
        assert!(matches!(
            validate_prepared_signed_transaction(&prepared, &changed),
            Err(SdkError::Validation(_))
        ));
    }
}
