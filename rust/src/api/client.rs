//! Lightcone REST API client implementation.
//!
//! The [`LightconeApiClient`] provides a type-safe interface for interacting with
//! the Lightcone REST API.
//!
//! # Example
//!
//! ```rust,ignore
//! use lightcone_sdk::api::LightconeApiClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = LightconeApiClient::new("https://api.lightcone.xyz");
//!
//!     // Get all markets
//!     let markets = client.get_markets().await?;
//!     println!("Found {} markets", markets.total);
//!
//!     // Get orderbook
//!     let orderbook = client.get_orderbook("orderbook_id", None).await?;
//!     println!("Best bid: {:?}", orderbook.best_bid);
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tokio::sync::RwLock;

use crate::api::error::{ApiError, ApiResult, ErrorResponse};
use crate::api::types::*;
use crate::program::orders::SignedOrder;
use crate::shared::OrderbookDecimals;

/// Default request timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Maximum allowed limit for paginated API requests.
const MAX_PAGINATION_LIMIT: u32 = 500;

/// Retry configuration for the API client.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = disabled)
    pub max_retries: u32,
    /// Base delay before first retry (ms)
    pub base_delay_ms: u64,
    /// Maximum delay between retries (ms)
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 0,
            base_delay_ms: 100,
            max_delay_ms: 10_000,
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with the given max retries.
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// Set the base delay in milliseconds.
    pub fn with_base_delay_ms(mut self, ms: u64) -> Self {
        self.base_delay_ms = ms;
        self
    }

    /// Set the maximum delay in milliseconds.
    pub fn with_max_delay_ms(mut self, ms: u64) -> Self {
        self.max_delay_ms = ms;
        self
    }

    /// Calculate delay for a given attempt with exponential backoff and jitter.
    fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let exp_delay = self.base_delay_ms.saturating_mul(1 << attempt.min(10));
        let capped_delay = exp_delay.min(self.max_delay_ms);
        // Add jitter: 75-100% of calculated delay
        let jitter_range = capped_delay / 4;
        let jitter = rand::random::<u64>() % (jitter_range + 1);
        Duration::from_millis(capped_delay - jitter_range + jitter)
    }
}

/// Builder for configuring [`LightconeApiClient`].
#[derive(Debug, Clone)]
pub struct LightconeApiClientBuilder {
    base_url: String,
    timeout: Duration,
    default_headers: Vec<(String, String)>,
    retry_config: RetryConfig,
    auth_token: Option<String>,
}

impl LightconeApiClientBuilder {
    /// Create a new builder with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            default_headers: Vec::new(),
            retry_config: RetryConfig::default(),
            auth_token: None,
        }
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set the timeout in seconds.
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout = Duration::from_secs(secs);
        self
    }

    /// Add a default header to all requests.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.push((name.into(), value.into()));
        self
    }

    /// Enable retries with exponential backoff.
    ///
    /// # Arguments
    ///
    /// * `config` - Retry configuration (use `RetryConfig::new(3)` for 3 retries with defaults)
    pub fn with_retry(mut self, config: RetryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Set the authentication token for authenticated endpoints (e.g. `get_user_orders`).
    pub fn auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Build the client.
    pub fn build(self) -> ApiResult<LightconeApiClient> {
        let mut builder = Client::builder()
            .timeout(self.timeout)
            .pool_max_idle_per_host(10);

        // Build default headers
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        for (name, value) in self.default_headers {
            let header_name = reqwest::header::HeaderName::try_from(name.as_str())
                .map_err(|e| ApiError::InvalidParameter(format!("Invalid header name '{}': {}", name, e)))?;
            let header_value = reqwest::header::HeaderValue::from_str(&value)
                .map_err(|e| ApiError::InvalidParameter(format!("Invalid header value for '{}': {}", name, e)))?;
            headers.insert(header_name, header_value);
        }

        builder = builder.default_headers(headers);

        let http_client = builder.build()?;

        Ok(LightconeApiClient {
            http_client,
            base_url: self.base_url,
            retry_config: self.retry_config,
            decimals_cache: Arc::new(RwLock::new(HashMap::new())),
            auth_token: self.auth_token,
        })
    }
}

/// Lightcone REST API client.
///
/// Provides methods for all Lightcone API endpoints including markets, orderbooks,
/// orders, positions, and price history.
#[derive(Debug, Clone)]
pub struct LightconeApiClient {
    http_client: Client,
    base_url: String,
    retry_config: RetryConfig,
    decimals_cache: Arc<RwLock<HashMap<String, OrderbookDecimals>>>,
    auth_token: Option<String>,
}

impl LightconeApiClient {
    /// Create a new client with the given base URL.
    ///
    /// Uses default settings (30s timeout, connection pooling).
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be initialized.
    pub fn new(base_url: impl Into<String>) -> ApiResult<Self> {
        LightconeApiClientBuilder::new(base_url).build()
    }

    /// Create a new client builder for custom configuration.
    pub fn builder(base_url: impl Into<String>) -> LightconeApiClientBuilder {
        LightconeApiClientBuilder::new(base_url)
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Authenticate with Lightcone using a Solana keypair.
    ///
    /// Calls the `/auth/login_or_register_with_message` endpoint, stores the
    /// returned JWT so that authenticated endpoints like `get_user_orders` work.
    ///
    /// Returns the full [`AuthCredentials`](crate::auth::AuthCredentials) (token, user_id, expiry).
    #[cfg(feature = "auth")]
    pub async fn login(
        &mut self,
        keypair: &solana_keypair::Keypair,
    ) -> ApiResult<crate::auth::AuthCredentials> {
        let credentials = crate::auth::authenticate(keypair)
            .await
            .map_err(|e| ApiError::InvalidParameter(e.to_string()))?;
        self.auth_token = Some(credentials.auth_token.clone());
        Ok(credentials)
    }

    /// Set the authentication token manually.
    pub fn set_auth_token(&mut self, token: impl Into<String>) {
        self.auth_token = Some(token.into());
    }

    /// Clear the authentication token.
    pub fn clear_auth_token(&mut self) {
        self.auth_token = None;
    }

    /// Check whether an authentication token is set.
    pub fn has_auth_token(&self) -> bool {
        self.auth_token.is_some()
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    /// Execute a GET request with optional retry logic.
    async fn get<T: serde::de::DeserializeOwned>(&self, url: &str) -> ApiResult<T> {
        self.execute_with_retry(|| self.http_client.get(url).send()).await
    }

    /// Execute a POST request with JSON body and optional retry logic.
    async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize + Clone>(
        &self,
        url: &str,
        body: &B,
    ) -> ApiResult<T> {
        self.execute_with_retry(|| self.http_client.post(url).json(body).send()).await
    }

    /// Execute a request with retry logic.
    async fn execute_with_retry<T, F, Fut>(&self, request_fn: F) -> ApiResult<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
        T: serde::de::DeserializeOwned,
    {
        let mut attempt = 0;

        loop {
            let result = request_fn().await;

            match result {
                Ok(response) => {
                    let status = response.status();

                    if status.is_success() {
                        return response.json::<T>().await.map_err(|e| {
                            ApiError::Deserialize(format!("Failed to deserialize response: {}", e))
                        });
                    }

                    // Parse error response
                    let error = self.parse_error_response(response).await;

                    // Check if we should retry
                    if attempt < self.retry_config.max_retries && Self::is_retryable_status(status) {
                        let delay = self.retry_config.delay_for_attempt(attempt);
                        tracing::debug!(
                            attempt = attempt + 1,
                            max_retries = self.retry_config.max_retries,
                            delay_ms = delay.as_millis(),
                            status = %status,
                            "Retrying request after error"
                        );
                        tokio::time::sleep(delay).await;
                        attempt += 1;
                        continue;
                    }

                    return Err(error);
                }
                Err(e) => {
                    let is_retryable = e.is_connect() || e.is_timeout() || e.is_request();

                    if attempt < self.retry_config.max_retries && is_retryable {
                        let delay = self.retry_config.delay_for_attempt(attempt);
                        tracing::debug!(
                            attempt = attempt + 1,
                            max_retries = self.retry_config.max_retries,
                            delay_ms = delay.as_millis(),
                            error = %e,
                            "Retrying request after network error"
                        );
                        tokio::time::sleep(delay).await;
                        attempt += 1;
                        continue;
                    }

                    return Err(ApiError::Http(e));
                }
            }
        }
    }

    /// Parse an error response into an ApiError.
    async fn parse_error_response(&self, response: reqwest::Response) -> ApiError {
        let status = response.status();
        let error_text = match response.text().await {
            Ok(text) => text,
            Err(e) => {
                tracing::warn!("Failed to read error response body: {}", e);
                return Self::map_status_error(
                    status,
                    ErrorResponse::from_text(format!("HTTP {} (body unreadable: {})", status, e)),
                );
            }
        };

        let error_response = serde_json::from_str::<ErrorResponse>(&error_text)
            .unwrap_or_else(|_| ErrorResponse::from_text(error_text));

        Self::map_status_error(status, error_response)
    }

    /// Map HTTP status code to ApiError.
    fn map_status_error(status: StatusCode, response: ErrorResponse) -> ApiError {
        match status {
            StatusCode::UNAUTHORIZED => ApiError::Unauthorized(response),
            StatusCode::NOT_FOUND => ApiError::NotFound(response),
            StatusCode::BAD_REQUEST => ApiError::BadRequest(response),
            StatusCode::FORBIDDEN => ApiError::Forbidden(response),
            StatusCode::CONFLICT => ApiError::Conflict(response),
            StatusCode::TOO_MANY_REQUESTS => ApiError::RateLimited(response),
            _ if status.is_server_error() => ApiError::ServerError(response),
            _ => ApiError::UnexpectedStatus(status.as_u16(), response),
        }
    }

    /// Check if a status code is retryable.
    fn is_retryable_status(status: StatusCode) -> bool {
        status.is_server_error() || status == StatusCode::TOO_MANY_REQUESTS
    }

    // =========================================================================
    // Validation helpers
    // =========================================================================

    /// Validate that a string is valid Base58 (Solana pubkey format).
    fn validate_base58(value: &str, field_name: &str) -> ApiResult<()> {
        if value.is_empty() {
            return Err(ApiError::InvalidParameter(format!("{} cannot be empty", field_name)));
        }
        bs58::decode(value)
            .into_vec()
            .map_err(|_| ApiError::InvalidParameter(format!("{} is not valid Base58", field_name)))?;
        Ok(())
    }

    /// Validate that a signature is 128 hex characters (64 bytes).
    fn validate_signature(sig: &str) -> ApiResult<()> {
        if sig.len() != 128 {
            return Err(ApiError::InvalidParameter(
                format!("Signature must be 128 hex characters, got {}", sig.len())
            ));
        }
        // Validate hex by attempting to decode
        for chunk in sig.as_bytes().chunks(2) {
            let hex_str = std::str::from_utf8(chunk).unwrap_or("");
            u8::from_str_radix(hex_str, 16)
                .map_err(|_| ApiError::InvalidParameter("Signature must contain only hex characters".to_string()))?;
        }
        Ok(())
    }

    /// Validate that a limit is within bounds.
    fn validate_limit(limit: u32, max: u32) -> ApiResult<()> {
        if limit == 0 || limit > max {
            return Err(ApiError::InvalidParameter(format!("Limit must be 1-{}", max)));
        }
        Ok(())
    }

    // =========================================================================
    // Health endpoints
    // =========================================================================

    /// Check API health.
    ///
    /// Returns `Ok(())` if the API is healthy.
    pub async fn health_check(&self) -> ApiResult<()> {
        let url = format!("{}/health", self.base_url);
        // Health check is special - we just need success status, not JSON parsing
        let response = self.http_client.get(&url).send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(ApiError::ServerError(ErrorResponse::from_text("Health check failed".to_string())))
        }
    }

    // =========================================================================
    // Market endpoints
    // =========================================================================

    /// Get all markets.
    ///
    /// Returns a list of all markets with their metadata.
    pub async fn get_markets(&self) -> ApiResult<MarketsResponse> {
        let url = format!("{}/api/markets", self.base_url);
        self.get(&url).await
    }

    /// Get market details by pubkey.
    ///
    /// Returns complete market information including deposit assets.
    pub async fn get_market(&self, market_pubkey: &str) -> ApiResult<MarketInfoResponse> {
        Self::validate_base58(market_pubkey, "market_pubkey")?;
        let url = format!("{}/api/markets/{}", self.base_url, urlencoding::encode(market_pubkey));
        self.get(&url).await
    }

    /// Get market by URL-friendly slug.
    pub async fn get_market_by_slug(&self, slug: &str) -> ApiResult<MarketInfoResponse> {
        if slug.is_empty() {
            return Err(ApiError::InvalidParameter("slug cannot be empty".to_string()));
        }
        let url = format!("{}/api/markets/by-slug/{}", self.base_url, urlencoding::encode(slug));
        self.get(&url).await
    }

    /// Get deposit assets for a market.
    pub async fn get_deposit_assets(&self, market_pubkey: &str) -> ApiResult<DepositAssetsResponse> {
        Self::validate_base58(market_pubkey, "market_pubkey")?;
        let url = format!("{}/api/markets/{}/deposit-assets", self.base_url, urlencoding::encode(market_pubkey));
        self.get(&url).await
    }

    // =========================================================================
    // Orderbook endpoints
    // =========================================================================

    /// Get orderbook depth.
    ///
    /// Returns price levels for bids and asks.
    ///
    /// # Arguments
    ///
    /// * `orderbook_id` - Orderbook identifier (can be "orderbook_id" or "market_pubkey:orderbook_id")
    /// * `depth` - Optional max price levels per side (0 or None = all)
    pub async fn get_orderbook(
        &self,
        orderbook_id: &str,
        depth: Option<u32>,
    ) -> ApiResult<OrderbookResponse> {
        let mut url = format!("{}/api/orderbook/{}", self.base_url, urlencoding::encode(orderbook_id));
        if let Some(d) = depth {
            url.push_str(&format!("?depth={}", d));
        }
        self.get(&url).await
    }

    // =========================================================================
    // Order endpoints
    // =========================================================================

    /// Submit a new order.
    ///
    /// The order must be pre-signed with the maker's Ed25519 key.
    pub async fn submit_order(&self, request: SubmitOrderRequest) -> ApiResult<OrderResponse> {
        Self::validate_base58(&request.maker, "maker")?;
        Self::validate_base58(&request.market_pubkey, "market_pubkey")?;
        Self::validate_base58(&request.base_token, "base_token")?;
        Self::validate_base58(&request.quote_token, "quote_token")?;
        Self::validate_signature(&request.signature)?;

        let url = format!("{}/api/orders/submit", self.base_url);
        self.post(&url, &request).await
    }

    /// Submit a signed SignedOrder to the API.
    ///
    /// Convenience method that converts the order and submits it.
    /// This bridges on-chain order creation with REST API submission.
    ///
    /// # Arguments
    ///
    /// * `order` - A signed SignedOrder (must have called `order.sign(&keypair)`)
    /// * `orderbook_id` - Target orderbook (use `order.derive_orderbook_id()` or from market API)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut order = SignedOrder::new_bid(params);
    /// order.sign(&keypair);
    ///
    /// let response = api_client
    ///     .submit_full_order(&order, order.derive_orderbook_id())
    ///     .await?;
    /// ```
    pub async fn submit_full_order(
        &self,
        order: &SignedOrder,
        orderbook_id: impl Into<String>,
    ) -> ApiResult<OrderResponse> {
        let request = order.to_submit_request(orderbook_id);
        self.submit_order(request).await
    }

    /// Cancel a specific order.
    ///
    /// The maker must match the order creator.
    pub async fn cancel_order(&self, order_hash: &str, maker: &str) -> ApiResult<CancelResponse> {
        Self::validate_base58(maker, "maker")?;

        let url = format!("{}/api/orders/cancel", self.base_url);
        let request = CancelOrderRequest {
            order_hash: order_hash.to_string(),
            maker: maker.to_string(),
        };
        self.post(&url, &request).await
    }

    /// Cancel all orders for a user.
    ///
    /// Optionally filter by market.
    pub async fn cancel_all_orders(
        &self,
        user_pubkey: &str,
        market_pubkey: Option<&str>,
    ) -> ApiResult<CancelAllResponse> {
        Self::validate_base58(user_pubkey, "user_pubkey")?;
        if let Some(market) = market_pubkey {
            Self::validate_base58(market, "market_pubkey")?;
        }

        let url = format!("{}/api/orders/cancel-all", self.base_url);
        let request = CancelAllOrdersRequest {
            user_pubkey: user_pubkey.to_string(),
            market_pubkey: market_pubkey.map(|s| s.to_string()),
        };
        self.post(&url, &request).await
    }

    // =========================================================================
    // User endpoints
    // =========================================================================

    /// Get all positions for a user.
    pub async fn get_user_positions(&self, user_pubkey: &str) -> ApiResult<PositionsResponse> {
        Self::validate_base58(user_pubkey, "user_pubkey")?;
        let url = format!("{}/api/users/{}/positions", self.base_url, urlencoding::encode(user_pubkey));
        self.get(&url).await
    }

    /// Get user positions in a specific market.
    pub async fn get_user_market_positions(
        &self,
        user_pubkey: &str,
        market_pubkey: &str,
    ) -> ApiResult<MarketPositionsResponse> {
        Self::validate_base58(user_pubkey, "user_pubkey")?;
        Self::validate_base58(market_pubkey, "market_pubkey")?;

        let url = format!(
            "{}/api/users/{}/markets/{}/positions",
            self.base_url,
            urlencoding::encode(user_pubkey),
            urlencoding::encode(market_pubkey)
        );
        self.get(&url).await
    }

    /// Get all open orders and balances for a user.
    ///
    /// Requires an auth token to be set via [`set_auth_token`] or [`LightconeApiClientBuilder::auth_token`].
    ///
    /// # Arguments
    ///
    /// * `wallet_address` - The user's wallet address (Base58)
    /// * `cursor` - Optional pagination cursor from a previous response's `next_cursor`
    /// * `limit` - Optional page size limit
    pub async fn get_user_orders(
        &self,
        wallet_address: &str,
        cursor: Option<&str>,
        limit: Option<u32>,
    ) -> ApiResult<UserOrdersResponse> {
        let token = self
            .auth_token
            .as_deref()
            .ok_or(ApiError::AuthenticationRequired)?;

        Self::validate_base58(wallet_address, "wallet_address")?;

        let mut url = format!(
            "{}/api/users/orders?wallet_address={}",
            self.base_url,
            urlencoding::encode(wallet_address)
        );
        if let Some(c) = cursor {
            url.push_str(&format!("&cursor={}", urlencoding::encode(c)));
        }
        if let Some(l) = limit {
            url.push_str(&format!("&limit={}", l));
        }

        self.execute_with_retry(|| {
            self.http_client
                .post(&url)
                .header("Cookie", format!("auth_token={}", token))
                .send()
        })
        .await
    }

    // =========================================================================
    // Price history endpoints
    // =========================================================================

    /// Get historical price data (candlesticks).
    pub async fn get_price_history(
        &self,
        params: PriceHistoryParams,
    ) -> ApiResult<PriceHistoryResponse> {
        if let Some(limit) = params.limit {
            Self::validate_limit(limit, MAX_PAGINATION_LIMIT)?;
        }

        let mut url = format!(
            "{}/api/price-history?orderbook_id={}",
            self.base_url,
            urlencoding::encode(&params.orderbook_id)
        );

        if let Some(resolution) = params.resolution {
            url.push_str(&format!("&resolution={}", urlencoding::encode(&resolution.to_string())));
        }
        if let Some(from) = params.from {
            url.push_str(&format!("&from={}", from));
        }
        if let Some(to) = params.to {
            url.push_str(&format!("&to={}", to));
        }
        if let Some(cursor) = params.cursor {
            url.push_str(&format!("&cursor={}", cursor));
        }
        if let Some(limit) = params.limit {
            url.push_str(&format!("&limit={}", limit));
        }
        if let Some(include_ohlcv) = params.include_ohlcv {
            url.push_str(&format!("&include_ohlcv={}", include_ohlcv));
        }

        self.get(&url).await
    }

    // =========================================================================
    // Trade endpoints
    // =========================================================================

    /// Get executed trades.
    pub async fn get_trades(&self, params: TradesParams) -> ApiResult<TradesResponse> {
        if let Some(ref user_pubkey) = params.user_pubkey {
            Self::validate_base58(user_pubkey, "user_pubkey")?;
        }
        if let Some(limit) = params.limit {
            Self::validate_limit(limit, MAX_PAGINATION_LIMIT)?;
        }

        let mut url = format!(
            "{}/api/trades?orderbook_id={}",
            self.base_url,
            urlencoding::encode(&params.orderbook_id)
        );

        if let Some(user_pubkey) = params.user_pubkey {
            url.push_str(&format!("&user_pubkey={}", urlencoding::encode(&user_pubkey)));
        }
        if let Some(from) = params.from {
            url.push_str(&format!("&from={}", from));
        }
        if let Some(to) = params.to {
            url.push_str(&format!("&to={}", to));
        }
        if let Some(cursor) = params.cursor {
            url.push_str(&format!("&cursor={}", cursor));
        }
        if let Some(limit) = params.limit {
            url.push_str(&format!("&limit={}", limit));
        }

        self.get(&url).await
    }

    // =========================================================================
    // Admin endpoints
    // =========================================================================

    /// Admin health check endpoint.
    pub async fn admin_health_check(&self) -> ApiResult<AdminResponse> {
        let url = format!("{}/api/admin/test", self.base_url);
        self.get(&url).await
    }

    /// Create a new orderbook for a market.
    pub async fn create_orderbook(
        &self,
        request: CreateOrderbookRequest,
    ) -> ApiResult<CreateOrderbookResponse> {
        Self::validate_base58(&request.market_pubkey, "market_pubkey")?;
        Self::validate_base58(&request.base_token, "base_token")?;
        Self::validate_base58(&request.quote_token, "quote_token")?;

        let url = format!("{}/api/admin/create-orderbook", self.base_url);
        self.post(&url, &request).await
    }

    // =========================================================================
    // Decimals / scaling helpers
    // =========================================================================

    /// Fetch and cache orderbook decimals. Cached permanently (decimals never change).
    ///
    /// On cache hit returns immediately. On miss, fetches from
    /// `GET /api/orderbooks/{orderbook_id}/decimals` and caches the result.
    pub async fn get_orderbook_decimals(
        &self,
        orderbook_id: &str,
    ) -> ApiResult<OrderbookDecimals> {
        // Fast path: read lock
        {
            let cache = self.decimals_cache.read().await;
            if let Some(d) = cache.get(orderbook_id) {
                return Ok(d.clone());
            }
        }

        // Slow path: fetch + write lock
        let url = format!(
            "{}/api/orderbooks/{}/decimals",
            self.base_url,
            urlencoding::encode(orderbook_id)
        );
        let resp: DecimalsResponse = self.get(&url).await?;

        let decimals = OrderbookDecimals {
            orderbook_id: resp.orderbook_id.clone(),
            base_decimals: resp.base_decimals,
            quote_decimals: resp.quote_decimals,
            price_decimals: resp.price_decimals,
        };

        let mut cache = self.decimals_cache.write().await;
        cache.insert(orderbook_id.to_string(), decimals.clone());

        Ok(decimals)
    }

    /// Pre-warm the decimals cache for multiple orderbooks.
    ///
    /// Fetches decimals for each orderbook that is not already cached.
    /// Errors from individual fetches are propagated (fails on first error).
    pub async fn prefetch_decimals(&self, orderbook_ids: &[&str]) -> ApiResult<()> {
        for id in orderbook_ids {
            self.get_orderbook_decimals(id).await?;
        }
        Ok(())
    }

    /// Clear the entire decimals cache.
    pub async fn clear_decimals_cache(&self) {
        self.decimals_cache.write().await.clear();
    }

    /// Remove a single orderbook entry from the decimals cache.
    pub async fn invalidate_decimals(&self, orderbook_id: &str) {
        self.decimals_cache.write().await.remove(orderbook_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::Resolution;

    #[test]
    fn test_client_creation() {
        let client = LightconeApiClient::new("https://api.lightcone.xyz").unwrap();
        assert_eq!(client.base_url(), "https://api.lightcone.xyz");
    }

    #[test]
    fn test_client_builder() {
        let client = LightconeApiClient::builder("https://api.lightcone.xyz/")
            .timeout_secs(60)
            .header("X-Custom", "test")
            .build()
            .unwrap();

        // Base URL should have trailing slash removed
        assert_eq!(client.base_url(), "https://api.lightcone.xyz");
    }

    #[test]
    fn test_price_history_params() {
        let params = PriceHistoryParams::new("orderbook1")
            .with_resolution(Resolution::OneHour)
            .with_time_range(1000, 2000)
            .with_limit(100)
            .with_ohlcv();

        assert_eq!(params.orderbook_id, "orderbook1");
        assert_eq!(params.resolution, Some(Resolution::OneHour));
        assert_eq!(params.from, Some(1000));
        assert_eq!(params.to, Some(2000));
        assert_eq!(params.limit, Some(100));
        assert_eq!(params.include_ohlcv, Some(true));
    }

    #[test]
    fn test_trades_params() {
        let params = TradesParams::new("orderbook1")
            .with_user("user123")
            .with_time_range(1000, 2000)
            .with_cursor(50)
            .with_limit(100);

        assert_eq!(params.orderbook_id, "orderbook1");
        assert_eq!(params.user_pubkey, Some("user123".to_string()));
        assert_eq!(params.from, Some(1000));
        assert_eq!(params.to, Some(2000));
        assert_eq!(params.cursor, Some(50));
        assert_eq!(params.limit, Some(100));
    }

    #[test]
    fn test_create_orderbook_request() {
        let request = CreateOrderbookRequest::new("market1", "base1", "quote1").with_tick_size(500);

        assert_eq!(request.market_pubkey, "market1");
        assert_eq!(request.base_token, "base1");
        assert_eq!(request.quote_token, "quote1");
        assert_eq!(request.tick_size, Some(500));
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::new(3)
            .with_base_delay_ms(200)
            .with_max_delay_ms(5000);

        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 200);
        assert_eq!(config.max_delay_ms, 5000);
    }

    #[test]
    fn test_client_with_retry() {
        let client = LightconeApiClient::builder("https://api.lightcone.xyz")
            .with_retry(RetryConfig::new(3))
            .build()
            .unwrap();

        assert_eq!(client.retry_config.max_retries, 3);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = RetryConfig {
            max_retries: 5,
            base_delay_ms: 100,
            max_delay_ms: 1000,
        };

        // First attempt: ~100ms (75-100ms with jitter)
        let delay0 = config.delay_for_attempt(0);
        assert!(delay0.as_millis() >= 75 && delay0.as_millis() <= 100);

        // Second attempt: ~200ms (150-200ms with jitter)
        let delay1 = config.delay_for_attempt(1);
        assert!(delay1.as_millis() >= 150 && delay1.as_millis() <= 200);

        // Fourth attempt would be 800ms, but capped at 1000ms max
        let delay3 = config.delay_for_attempt(3);
        assert!(delay3.as_millis() >= 600 && delay3.as_millis() <= 800);

        // Large attempt: should be capped at max_delay
        let delay10 = config.delay_for_attempt(10);
        assert!(delay10.as_millis() >= 750 && delay10.as_millis() <= 1000);
    }
}
