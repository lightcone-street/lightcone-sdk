//! High-level client — `LightconeClient` with nested sub-client accessors.
//!
//! Each domain has its own sub-client in `domain/<name>/client.rs`.
//! This module keeps the builder, auth state, and accessor methods.
//!
//! **Caching philosophy**: The SDK is stateless for HTTP data. Caching is the
//! consumer's responsibility (e.g. Dioxus server functions, CLI memoization).
//! The only exception is `decimals_cache` — orderbook decimals are effectively
//! immutable and are a safe SDK-level optimization.

use crate::auth::client::Auth;
use crate::auth::AuthCredentials;
use crate::domain::admin::client::Admin;
use crate::privy::client::Privy;
use crate::domain::market::client::Markets;
use crate::domain::order::client::Orders;
use crate::domain::orderbook::client::Orderbooks;
use crate::domain::orderbook::wire::DecimalsResponse;
use crate::domain::position::client::Positions;
use crate::domain::price_history::client::PriceHistoryClient;
use crate::domain::trade::client::Trades;
use crate::error::SdkError;
use crate::http::LightconeHttp;
use crate::ws::WsConfig;

use async_lock::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

// Re-export sub-client types for convenience.
pub use crate::auth::client::Auth as AuthClient;
pub use crate::domain::admin::client::Admin as AdminClient;
pub use crate::domain::market::client::Markets as MarketsClient;
pub use crate::domain::order::client::Orders as OrdersClient;
pub use crate::domain::orderbook::client::Orderbooks as OrderbooksClient;
pub use crate::domain::position::client::Positions as PositionsClient;
pub use crate::domain::price_history::client::PriceHistoryClient as PriceHistorySubClient;
pub use crate::domain::trade::client::Trades as TradesClient;

/// The primary entry point for the Lightcone SDK.
///
/// Provides nested sub-client accessors for each domain:
/// `client.markets()`, `client.orders()`, etc.
///
/// The client is intentionally stateless for HTTP data — no market cache,
/// no slug index. The consumer manages caching at the application layer.
/// The only internal cache is `decimals_cache` for orderbook decimals,
/// which are effectively immutable.
pub struct LightconeClient {
    pub(crate) http: LightconeHttp,
    pub(crate) ws_config: WsConfig,
    pub(crate) auth_credentials: Arc<RwLock<Option<AuthCredentials>>>,
    /// Decimals cache: orderbook_id → DecimalsResponse (rarely changes)
    pub(crate) decimals_cache: Arc<RwLock<HashMap<String, DecimalsResponse>>>,
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
        crate::ws::native::WsClient::new(self.ws_config.clone())
    }

    /// Create a new WASM WS client from the current config.
    #[cfg(feature = "ws-wasm")]
    pub fn ws_wasm(&self) -> crate::ws::wasm::WsClient {
        crate::ws::wasm::WsClient::new(self.ws_config.clone())
    }

    /// Clear the decimals cache (the only SDK-internal cache).
    pub async fn clear_decimals_cache(&self) {
        self.decimals_cache.write().await.clear();
    }
}

impl Clone for LightconeClient {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            ws_config: self.ws_config.clone(),
            auth_credentials: self.auth_credentials.clone(),
            decimals_cache: self.decimals_cache.clone(),
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
}

impl Default for LightconeClientBuilder {
    fn default() -> Self {
        Self {
            base_url: crate::network::DEFAULT_API_URL.to_string(),
            ws_url: crate::network::DEFAULT_WS_URL.to_string(),
            auth_credentials: None,
        }
    }
}

impl LightconeClientBuilder {
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

    pub fn build(self) -> Result<LightconeClient, SdkError> {
        Ok(LightconeClient {
            http: LightconeHttp::new(&self.base_url),
            ws_config: WsConfig {
                url: self.ws_url,
                ..WsConfig::default()
            },
            auth_credentials: Arc::new(RwLock::new(self.auth_credentials)),
            decimals_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}
