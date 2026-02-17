//! High-level client — `LightconeClient` with nested sub-client accessors.
//!
//! Each domain has its own sub-client in `domain/<name>/client.rs`.
//! This module keeps the builder, shared cache state, and accessor methods.

use crate::auth::client::Auth;
use crate::auth::AuthCredentials;
use crate::domain::admin::client::Admin;
use crate::domain::market::client::Markets;
use crate::domain::market::Market;
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
use std::time::{Duration, Instant};

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
pub struct LightconeClient {
    pub(crate) http: LightconeHttp,
    pub(crate) ws_config: WsConfig,
    /// Internal auth state.
    pub(crate) auth_credentials: Arc<RwLock<Option<AuthCredentials>>>,
    /// Market cache: pubkey → (Market, fetched_at)
    pub(crate) market_cache: Arc<RwLock<HashMap<String, (Market, Instant)>>>,
    /// Market slug index: slug → pubkey
    pub(crate) slug_index: Arc<RwLock<HashMap<String, String>>>,
    /// Decimals cache: orderbook_id → DecimalsResponse (rarely changes)
    pub(crate) decimals_cache: Arc<RwLock<HashMap<String, DecimalsResponse>>>,
    /// Cache TTL for markets
    pub(crate) market_cache_ttl: Duration,
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

    /// Get a WS config for creating a WebSocket connection.
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

    /// Clear all HTTP caches.
    pub async fn clear_all_caches(&self) {
        self.market_cache.write().await.clear();
        self.slug_index.write().await.clear();
        self.decimals_cache.write().await.clear();
    }
}

impl Clone for LightconeClient {
    fn clone(&self) -> Self {
        Self {
            http: self.http.clone(),
            ws_config: self.ws_config.clone(),
            auth_credentials: self.auth_credentials.clone(),
            market_cache: self.market_cache.clone(),
            slug_index: self.slug_index.clone(),
            decimals_cache: self.decimals_cache.clone(),
            market_cache_ttl: self.market_cache_ttl,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Builder
// ═════════════════════════════════════════════════════════════════════════════

pub struct LightconeClientBuilder {
    base_url: String,
    ws_url: String,
    market_cache_ttl: Duration,
    auth_credentials: Option<AuthCredentials>,
}

impl Default for LightconeClientBuilder {
    fn default() -> Self {
        Self {
            base_url: crate::network::DEFAULT_API_URL.to_string(),
            ws_url: crate::network::DEFAULT_WS_URL.to_string(),
            market_cache_ttl: Duration::from_secs(60),
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

    pub fn market_cache_ttl(mut self, ttl: Duration) -> Self {
        self.market_cache_ttl = ttl;
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
            market_cache: Arc::new(RwLock::new(HashMap::new())),
            slug_index: Arc::new(RwLock::new(HashMap::new())),
            decimals_cache: Arc::new(RwLock::new(HashMap::new())),
            market_cache_ttl: self.market_cache_ttl,
        })
    }
}
