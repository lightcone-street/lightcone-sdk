//! Low-level HTTP client — `LightconeHttp`.
//!
//! One method per API endpoint. Returns wire types (conversion to domain types
//! happens at the Layer 5 boundary). Internal to the SDK — Layer 5 wraps this.

use crate::domain::admin::{AdminEnvelope, UnifiedMetadataRequest, UnifiedMetadataResponse};
use crate::domain::market::wire::{MarketResponse, MarketSearchResult, MarketsResponse};
use crate::domain::orderbook::wire::{DecimalsResponse, OrderbookDepthResponse};
use crate::domain::position::wire::PositionsResponse;
use crate::domain::trade::wire::TradesResponse;
use crate::error::HttpError;
use crate::http::retry::{RetryConfig, RetryPolicy};
use crate::shared::Resolution;

use async_lock::RwLock;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;
use tracing;

/// Low-level HTTP client for the Lightcone REST API.
pub struct LightconeHttp {
    base_url: String,
    client: Client,
    /// Auth token for native clients. NEVER exposed publicly.
    auth_token: Arc<RwLock<Option<String>>>,
}

impl LightconeHttp {
    pub fn new(base_url: &str) -> Self {
        let mut builder = Client::builder();
        #[cfg(not(target_arch = "wasm32"))]
        {
            builder = builder
                .timeout(Duration::from_secs(30))
                .pool_max_idle_per_host(10);
        }

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: builder.build().expect("Failed to build HTTP client"),
            auth_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the auth token (native only — on WASM, cookies handle auth).
    pub(crate) async fn set_auth_token(&self, token: Option<String>) {
        *self.auth_token.write().await = token;
    }

    /// Clear auth token.
    pub(crate) async fn clear_auth_token(&self) {
        *self.auth_token.write().await = None;
    }

    /// Check if an auth token is set (native only).
    #[allow(dead_code)]
    pub(crate) async fn has_auth_token(&self) -> bool {
        self.auth_token.read().await.is_some()
    }

    // ── Markets ──────────────────────────────────────────────────────────

    pub async fn get_markets(
        &self,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> Result<MarketsResponse, HttpError> {
        let mut url = format!("{}/api/markets", self.base_url);
        let mut params = Vec::new();
        if let Some(p) = page {
            params.push(format!("page={}", p));
        }
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }
        self.get(&url, RetryPolicy::Idempotent).await
    }

    pub async fn get_market_by_slug(&self, slug: &str) -> Result<MarketResponse, HttpError> {
        let url = format!("{}/api/markets/by-slug/{}", self.base_url, slug);
        self.get(&url, RetryPolicy::Idempotent).await
    }

    pub async fn search_markets(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> Result<Vec<MarketSearchResult>, HttpError> {
        let mut url = format!(
            "{}/api/markets/search?q={}",
            self.base_url,
            urlencoding::encode(query)
        );
        if let Some(l) = limit {
            url = format!("{}&limit={}", url, l);
        }
        self.get(&url, RetryPolicy::Idempotent).await
    }

    pub async fn get_featured_markets(&self) -> Result<Vec<MarketSearchResult>, HttpError> {
        let url = format!("{}/api/markets/featured", self.base_url);
        self.get(&url, RetryPolicy::Idempotent).await
    }

    // ── Orderbooks ───────────────────────────────────────────────────────

    pub async fn get_orderbook(
        &self,
        orderbook_id: &str,
        depth: Option<u32>,
    ) -> Result<OrderbookDepthResponse, HttpError> {
        let mut url = format!("{}/api/orderbook/{}", self.base_url, orderbook_id);
        if let Some(d) = depth {
            url = format!("{}?depth={}", url, d);
        }
        self.get(&url, RetryPolicy::Idempotent).await
    }

    pub async fn get_orderbook_decimals(
        &self,
        orderbook_id: &str,
    ) -> Result<DecimalsResponse, HttpError> {
        let url = format!(
            "{}/api/orderbooks/{}/decimals",
            self.base_url, orderbook_id
        );
        self.get(&url, RetryPolicy::Idempotent).await
    }

    // ── Orders ───────────────────────────────────────────────────────────

    pub async fn submit_order<T: Serialize>(
        &self,
        request: &T,
    ) -> Result<serde_json::Value, HttpError> {
        let url = format!("{}/api/orders/submit", self.base_url);
        self.post(&url, request, RetryPolicy::None).await
    }

    pub async fn cancel_order<T: Serialize>(
        &self,
        request: &T,
    ) -> Result<serde_json::Value, HttpError> {
        let url = format!("{}/api/orders/cancel", self.base_url);
        self.post(&url, request, RetryPolicy::None).await
    }

    pub async fn cancel_all_orders<T: Serialize>(
        &self,
        request: &T,
    ) -> Result<serde_json::Value, HttpError> {
        let url = format!("{}/api/orders/cancel-all", self.base_url);
        self.post(&url, request, RetryPolicy::None).await
    }

    pub async fn get_user_orders<T: Serialize>(
        &self,
        request: &T,
    ) -> Result<serde_json::Value, HttpError> {
        let url = format!("{}/api/users/orders", self.base_url);
        self.post(&url, request, RetryPolicy::Idempotent).await
    }

    // ── Positions ────────────────────────────────────────────────────────

    pub async fn get_user_positions(
        &self,
        user_pubkey: &str,
    ) -> Result<PositionsResponse, HttpError> {
        let url = format!("{}/api/users/{}/positions", self.base_url, user_pubkey);
        self.get(&url, RetryPolicy::Idempotent).await
    }

    pub async fn get_user_market_positions(
        &self,
        user_pubkey: &str,
        market_pubkey: &str,
    ) -> Result<PositionsResponse, HttpError> {
        let url = format!(
            "{}/api/users/{}/positions?market={}",
            self.base_url, user_pubkey, market_pubkey
        );
        self.get(&url, RetryPolicy::Idempotent).await
    }

    // ── Trades ───────────────────────────────────────────────────────────

    pub async fn get_trades(
        &self,
        orderbook_id: &str,
        limit: Option<u32>,
        before: Option<&str>,
    ) -> Result<TradesResponse, HttpError> {
        let mut url = format!(
            "{}/api/trades?orderbook_id={}",
            self.base_url, orderbook_id
        );
        if let Some(l) = limit {
            url = format!("{}&limit={}", url, l);
        }
        if let Some(b) = before {
            url = format!("{}&before={}", url, b);
        }
        self.get(&url, RetryPolicy::Idempotent).await
    }

    // ── Price History ────────────────────────────────────────────────────

    pub async fn get_price_history(
        &self,
        orderbook_id: &str,
        resolution: Resolution,
        from: Option<u64>,
        to: Option<u64>,
    ) -> Result<serde_json::Value, HttpError> {
        let mut url = format!(
            "{}/api/price-history?orderbook_id={}&resolution={}",
            self.base_url,
            orderbook_id,
            resolution.as_str()
        );
        if let Some(f) = from {
            url = format!("{}&from={}", url, f);
        }
        if let Some(t) = to {
            url = format!("{}&to={}", url, t);
        }
        self.get(&url, RetryPolicy::Idempotent).await
    }

    // ── Admin ────────────────────────────────────────────────────────────

    pub async fn admin_upsert_metadata(
        &self,
        envelope: &AdminEnvelope<UnifiedMetadataRequest>,
    ) -> Result<UnifiedMetadataResponse, HttpError> {
        let url = format!("{}/api/admin/metadata", self.base_url);
        self.post(&url, envelope, RetryPolicy::None).await
    }

    // ── Auth ─────────────────────────────────────────────────────────────

    pub async fn login(
        &self,
        body: &impl Serialize,
    ) -> Result<serde_json::Value, HttpError> {
        let url = format!(
            "{}/api/auth/login_or_register_with_message",
            self.base_url
        );
        self.post(&url, body, RetryPolicy::None).await
    }

    pub async fn logout(&self) -> Result<serde_json::Value, HttpError> {
        let url = format!("{}/api/auth/logout", self.base_url);
        self.post(&url, &serde_json::json!({}), RetryPolicy::None)
            .await
    }

    // ── Internal HTTP methods ────────────────────────────────────────────

    async fn get<T: DeserializeOwned>(
        &self,
        url: &str,
        retry: RetryPolicy,
    ) -> Result<T, HttpError> {
        self.request_with_retry(reqwest::Method::GET, url, None::<&()>, retry)
            .await
    }

    async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        url: &str,
        body: &B,
        retry: RetryPolicy,
    ) -> Result<T, HttpError> {
        self.request_with_retry(reqwest::Method::POST, url, Some(body), retry)
            .await
    }

    async fn request_with_retry<T: DeserializeOwned, B: Serialize>(
        &self,
        method: reqwest::Method,
        url: &str,
        body: Option<&B>,
        retry: RetryPolicy,
    ) -> Result<T, HttpError> {
        let config = match &retry {
            RetryPolicy::None => {
                return self.do_request(&method, url, body).await;
            }
            RetryPolicy::Idempotent => RetryConfig::idempotent(),
            RetryPolicy::Custom(c) => c.clone(),
        };

        let mut last_error = None;

        for attempt in 0..=config.max_retries {
            match self.do_request::<T, B>(&method, url, body).await {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    let should_retry = match &e {
                        HttpError::ServerError { status, .. } => {
                            config.retryable_statuses.contains(status)
                        }
                        HttpError::RateLimited { retry_after_ms } => {
                            if let Some(ms) = retry_after_ms {
                                let delay = Duration::from_millis(*ms);
                                futures_timer::Delay::new(delay).await;
                            }
                            true
                        }
                        HttpError::Timeout => true,
                        #[cfg(feature = "http")]
                        HttpError::Reqwest(re) => {
                            #[cfg(not(target_arch = "wasm32"))]
                            let retryable = re.is_connect() || re.is_timeout() || re.is_request();
                            #[cfg(target_arch = "wasm32")]
                            let retryable = re.is_timeout() || re.is_request();
                            retryable
                        }
                        _ => false,
                    };

                    if should_retry && attempt < config.max_retries {
                        let delay = config.delay_for_attempt(attempt);
                        tracing::debug!(
                            attempt = attempt + 1,
                            max = config.max_retries,
                            delay_ms = delay.as_millis() as u64,
                            "Retrying request to {}",
                            url
                        );
                        futures_timer::Delay::new(delay).await;
                        last_error = Some(e);
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        Err(HttpError::MaxRetriesExceeded {
            attempts: config.max_retries + 1,
            last_error: last_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
        })
    }

    async fn do_request<T: DeserializeOwned, B: Serialize>(
        &self,
        method: &reqwest::Method,
        url: &str,
        body: Option<&B>,
    ) -> Result<T, HttpError> {
        let mut req = self.client.request(method.clone(), url);

        // Inject auth token on native
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(token) = self.auth_token.read().await.as_ref() {
                req = req.header("Authorization", format!("Bearer {}", token));
            }
        }

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send().await?;
        let status = resp.status();

        if status.is_success() {
            let parsed = resp.json::<T>().await?;
            return Ok(parsed);
        }

        let status_code = status.as_u16();
        let body_text = resp.text().await.unwrap_or_default();

        match status_code {
            401 => Err(HttpError::Unauthorized),
            404 => Err(HttpError::NotFound(body_text)),
            429 => Err(HttpError::RateLimited {
                retry_after_ms: None,
            }),
            400..=499 => Err(HttpError::BadRequest(body_text)),
            _ => Err(HttpError::ServerError {
                status: status_code,
                body: body_text,
            }),
        }
    }
}

impl Clone for LightconeHttp {
    fn clone(&self) -> Self {
        Self {
            base_url: self.base_url.clone(),
            client: self.client.clone(),
            auth_token: self.auth_token.clone(),
        }
    }
}
