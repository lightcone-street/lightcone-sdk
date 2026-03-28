//! High-level client — `LightconeClient` with nested sub-client accessors.
//!
//! Each domain has its own sub-client in `domain/<name>/client.rs`.
//! This module keeps the builder, auth state, and accessor methods.
//!
//! **Caching philosophy**: The SDK is stateless for HTTP data. Caching is the
//! consumer's responsibility (e.g. Dioxus server functions, CLI memoization).

use crate::auth::client::Auth;
use crate::auth::AuthCredentials;
use crate::domain::admin::client::Admin;
use crate::domain::market::client::Markets;
use crate::domain::notification::client::Notifications;
use crate::domain::order::client::Orders;
use crate::domain::orderbook::client::Orderbooks;
use crate::domain::position::client::Positions;
use crate::domain::price_history::client::PriceHistoryClient;
use crate::domain::referral::client::Referrals;
use crate::domain::trade::client::Trades;
use crate::env::LightconeEnv;
use crate::error::SdkError;
use crate::http::LightconeHttp;
use crate::privy::client::Privy;
use crate::rpc::Rpc;
use crate::shared::signing::{ExternalSigner, SigningStrategy};
use crate::shared::DepositSource;
use crate::ws::WsConfig;

#[cfg(feature = "solana-rpc")]
use solana_client::nonblocking::rpc_client::RpcClient as SolanaRpcClient;
#[cfg(feature = "solana-rpc")]
use solana_commitment_config::CommitmentConfig;

use async_lock::RwLock;
use solana_pubkey::Pubkey;
use std::sync::Arc;

// Re-export sub-client types for convenience.
pub use crate::auth::client::Auth as AuthClient;
pub use crate::domain::admin::client::Admin as AdminClient;
pub use crate::domain::market::client::{Markets as MarketsClient, MarketsResult};
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
/// The client is intentionally stateless for HTTP data — no market cache,
/// no slug index. The consumer manages caching at the application layer.
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
    /// Solana RPC URL for blockhash fetching and transaction submission.
    /// Used by `sign_and_submit_tx()` and `get_latest_blockhash()`.
    pub(crate) rpc_url: Option<String>,
    /// Optional Solana RPC client for on-chain reads (native only).
    #[cfg(feature = "solana-rpc")]
    pub(crate) solana_rpc_client: Option<SolanaRpcClient>,
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

    pub fn admin(&self) -> Admin<'_> {
        Admin { client: self }
    }

    pub fn auth(&self) -> Auth<'_> {
        Auth { client: self }
    }

    pub fn privy(&self) -> Privy<'_> {
        Privy { client: self }
    }

    pub fn referrals(&self) -> Referrals<'_> {
        Referrals { client: self }
    }

    pub fn notifications(&self) -> Notifications<'_> {
        Notifications { client: self }
    }

    /// RPC sub-client — PDA helpers, account fetchers, and blockhash access.
    pub fn rpc(&self) -> Rpc<'_> {
        Rpc { client: self }
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

    // ── RPC helpers (HTTP-based, works on all platforms) ─────────────────

    /// Get the configured RPC URL, or error if not set.
    pub(crate) fn require_rpc_url(&self) -> Result<&str, SdkError> {
        self.rpc_url
            .as_deref()
            .ok_or_else(|| SdkError::Validation("rpc_url is not configured on the client".into()))
    }

    /// Fetch the latest blockhash via JSON-RPC POST.
    ///
    /// Works on all platforms (native + WASM). Uses the `rpc_url` configured on the client.
    pub async fn get_latest_blockhash(&self) -> Result<solana_hash::Hash, SdkError> {
        let rpc_url = self.require_rpc_url()?;
        let body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "getLatestBlockhash",
            "params": []
        });

        let response: serde_json::Value = self
            .http
            .raw_post(rpc_url, &body)
            .await
            .map_err(|error| SdkError::Other(format!("blockhash RPC failed: {error}")))?;

        let blockhash_str = response["result"]["value"]["blockhash"]
            .as_str()
            .ok_or_else(|| SdkError::Other("missing blockhash in RPC response".into()))?;

        blockhash_str
            .parse::<solana_hash::Hash>()
            .map_err(|error| SdkError::Other(format!("invalid blockhash: {error}")))
    }

    /// Sign and submit a transaction using the client's signing strategy.
    ///
    /// Fetches a recent blockhash automatically. The caller does not need to set it.
    ///
    /// - **Native**: signs locally with keypair, submits via RPC `sendTransaction`
    /// - **WalletAdapter**: signs via external signer, submits via RPC `sendTransaction`
    /// - **Privy**: serializes unsigned tx to base64, sends to backend for signing + submission
    pub async fn sign_and_submit_tx(
        &self,
        mut tx: solana_transaction::Transaction,
    ) -> Result<String, SdkError> {
        let strategy = self.signing_strategy().await.ok_or_else(|| {
            SdkError::Validation("signing strategy is not set on the client".into())
        })?;

        let blockhash = self.get_latest_blockhash().await?;
        tx.message.recent_blockhash = blockhash;

        match strategy {
            #[cfg(feature = "native-auth")]
            SigningStrategy::Native(keypair) => {
                use solana_signer::Signer;
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
                // The signer returns fully signed tx bytes — send via base64
                let base64_tx = base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    &signed_bytes,
                );
                self.send_raw_transaction_rpc(&base64_tx).await
            }
            SigningStrategy::Privy { wallet_id } => {
                let tx_bytes = bincode::serialize(&tx).map_err(|error| {
                    SdkError::Other(format!("tx serialization failed: {error}"))
                })?;
                let base64_tx =
                    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &tx_bytes);
                let result = self
                    .privy()
                    .sign_and_send_tx(&wallet_id, &base64_tx)
                    .await?;
                Ok(result.hash)
            }
        }
    }

    /// Submit a signed transaction via JSON-RPC `sendTransaction`.
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
        let rpc_url = self.require_rpc_url()?;
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

        let response: serde_json::Value = self
            .http
            .raw_post(rpc_url, &body)
            .await
            .map_err(|error| SdkError::Other(format!("sendTransaction RPC failed: {error}")))?;

        if let Some(error) = response.get("error") {
            return Err(SdkError::Other(format!("RPC error: {error}")));
        }

        response["result"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| SdkError::Other("no signature in sendTransaction response".into()))
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
            signing_strategy: self.signing_strategy.clone(),
            rpc_url: self.rpc_url.clone(),
            #[cfg(feature = "solana-rpc")]
            solana_rpc_client: self.solana_rpc_client.as_ref().map(|rpc_client| {
                // SolanaRpcClient doesn't implement Clone; create a new one with the same URL.
                // This is a limitation — the cloned client shares no connection state.
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
    rpc_url: Option<String>,
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
            rpc_url: Some(environment.rpc_url().to_string()),
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
        self.rpc_url = Some(environment.rpc_url().to_string());
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

    /// Set a Privy embedded wallet ID for signing orders, cancels, and transactions.
    /// The backend signs on behalf of the user using the Privy wallet.
    pub fn privy_wallet_id(mut self, wallet_id: impl Into<String>) -> Self {
        self.signing_strategy = Some(SigningStrategy::Privy {
            wallet_id: wallet_id.into(),
        });
        self
    }

    /// Set the Solana RPC URL for blockhash fetching, transaction submission,
    /// and on-chain reads (when `solana-rpc` feature is enabled).
    pub fn rpc_url(mut self, url: &str) -> Self {
        self.rpc_url = Some(url.to_string());
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
            signing_strategy: Arc::new(RwLock::new(self.signing_strategy)),
            rpc_url: self.rpc_url.clone(),
            #[cfg(feature = "solana-rpc")]
            solana_rpc_client: self.rpc_url.map(|url| {
                SolanaRpcClient::new_with_commitment(url, CommitmentConfig::confirmed())
            }),
        })
    }
}
