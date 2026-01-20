//! Lightcone REST API client implementation.
//!
//! The [`LightconeApiClient`] provides a type-safe interface for interacting with
//! the Lightcone REST API.
//!
//! # Example
//!
//! ```rust,ignore
//! use lightcone_pinocchio_sdk::api::LightconeApiClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = LightconeApiClient::new("https://api.lightcone.io");
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

use std::time::Duration;

use reqwest::{Client, StatusCode};

use crate::api::error::{ApiError, ApiResult, ErrorResponse};
use crate::api::types::*;

/// Default request timeout in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Builder for configuring [`LightconeApiClient`].
#[derive(Debug, Clone)]
pub struct LightconeApiClientBuilder {
    base_url: String,
    timeout: Duration,
    default_headers: Vec<(String, String)>,
}

impl LightconeApiClientBuilder {
    /// Create a new builder with the given base URL.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
            default_headers: Vec::new(),
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
            if let (Ok(header_name), Ok(header_value)) = (
                reqwest::header::HeaderName::try_from(name.as_str()),
                reqwest::header::HeaderValue::from_str(&value),
            ) {
                headers.insert(header_name, header_value);
            }
        }

        builder = builder.default_headers(headers);

        let http_client = builder.build()?;

        Ok(LightconeApiClient {
            http_client,
            base_url: self.base_url,
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
}

impl LightconeApiClient {
    /// Create a new client with the given base URL.
    ///
    /// Uses default settings (30s timeout, connection pooling).
    pub fn new(base_url: impl Into<String>) -> Self {
        LightconeApiClientBuilder::new(base_url)
            .build()
            .expect("Failed to build default HTTP client")
    }

    /// Create a new client builder for custom configuration.
    pub fn builder(base_url: impl Into<String>) -> LightconeApiClientBuilder {
        LightconeApiClientBuilder::new(base_url)
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    /// Handle HTTP response and map errors.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> ApiResult<T> {
        let status = response.status();

        if status.is_success() {
            response.json::<T>().await.map_err(|e| {
                ApiError::Deserialize(format!("Failed to deserialize response: {}", e))
            })
        } else {
            // Try to parse error response
            let error_text = response.text().await.unwrap_or_default();
            let error_msg = if let Ok(err) = serde_json::from_str::<ErrorResponse>(&error_text) {
                err.get_message()
            } else {
                error_text
            };

            Err(Self::map_status_error(status, error_msg))
        }
    }

    /// Map HTTP status code to ApiError.
    fn map_status_error(status: StatusCode, message: String) -> ApiError {
        match status {
            StatusCode::NOT_FOUND => ApiError::NotFound(message),
            StatusCode::BAD_REQUEST => ApiError::BadRequest(message),
            StatusCode::FORBIDDEN => ApiError::Forbidden(message),
            StatusCode::CONFLICT => ApiError::Conflict(message),
            _ if status.is_server_error() => ApiError::ServerError(message),
            _ => ApiError::UnexpectedStatus(status.as_u16(), message),
        }
    }

    // =========================================================================
    // Health endpoints
    // =========================================================================

    /// Check API health.
    ///
    /// Returns `Ok(())` if the API is healthy.
    pub async fn health_check(&self) -> ApiResult<()> {
        let url = format!("{}/health", self.base_url);
        let response = self.http_client.get(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ApiError::ServerError("Health check failed".to_string()))
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
        let response = self.http_client.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// Get market details by pubkey.
    ///
    /// Returns complete market information including deposit assets.
    pub async fn get_market(&self, market_pubkey: &str) -> ApiResult<MarketInfoResponse> {
        let url = format!("{}/api/markets/{}", self.base_url, market_pubkey);
        let response = self.http_client.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// Get market by URL-friendly slug.
    pub async fn get_market_by_slug(&self, slug: &str) -> ApiResult<MarketInfoResponse> {
        let url = format!("{}/api/markets/by-slug/{}", self.base_url, slug);
        let response = self.http_client.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// Get deposit assets for a market.
    pub async fn get_deposit_assets(&self, market_pubkey: &str) -> ApiResult<DepositAssetsResponse> {
        let url = format!(
            "{}/api/markets/{}/deposit-assets",
            self.base_url, market_pubkey
        );
        let response = self.http_client.get(&url).send().await?;
        self.handle_response(response).await
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
        let mut url = format!("{}/api/orderbook/{}", self.base_url, orderbook_id);

        if let Some(d) = depth {
            url.push_str(&format!("?depth={}", d));
        }

        let response = self.http_client.get(&url).send().await?;
        self.handle_response(response).await
    }

    // =========================================================================
    // Order endpoints
    // =========================================================================

    /// Submit a new order.
    ///
    /// The order must be pre-signed with the maker's Ed25519 key.
    pub async fn submit_order(&self, request: SubmitOrderRequest) -> ApiResult<OrderResponse> {
        let url = format!("{}/api/orders/submit", self.base_url);
        let response = self.http_client.post(&url).json(&request).send().await?;
        self.handle_response(response).await
    }

    /// Cancel a specific order.
    ///
    /// The maker must match the order creator.
    pub async fn cancel_order(&self, order_hash: &str, maker: &str) -> ApiResult<CancelResponse> {
        let url = format!("{}/api/orders/cancel", self.base_url);
        let request = CancelOrderRequest {
            order_hash: order_hash.to_string(),
            maker: maker.to_string(),
        };
        let response = self.http_client.post(&url).json(&request).send().await?;
        self.handle_response(response).await
    }

    /// Cancel all orders for a user.
    ///
    /// Optionally filter by market.
    pub async fn cancel_all_orders(
        &self,
        user_pubkey: &str,
        market_pubkey: Option<&str>,
    ) -> ApiResult<CancelAllResponse> {
        let url = format!("{}/api/orders/cancel-all", self.base_url);
        let request = CancelAllOrdersRequest {
            user_pubkey: user_pubkey.to_string(),
            market_pubkey: market_pubkey.map(|s| s.to_string()),
        };
        let response = self.http_client.post(&url).json(&request).send().await?;
        self.handle_response(response).await
    }

    // =========================================================================
    // User endpoints
    // =========================================================================

    /// Get all positions for a user.
    pub async fn get_user_positions(&self, user_pubkey: &str) -> ApiResult<PositionsResponse> {
        let url = format!("{}/api/users/{}/positions", self.base_url, user_pubkey);
        let response = self.http_client.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// Get user positions in a specific market.
    pub async fn get_user_market_positions(
        &self,
        user_pubkey: &str,
        market_pubkey: &str,
    ) -> ApiResult<MarketPositionsResponse> {
        let url = format!(
            "{}/api/users/{}/markets/{}/positions",
            self.base_url, user_pubkey, market_pubkey
        );
        let response = self.http_client.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// Get all open orders and balances for a user.
    pub async fn get_user_orders(&self, user_pubkey: &str) -> ApiResult<UserOrdersResponse> {
        let url = format!("{}/api/users/orders", self.base_url);
        let request = GetUserOrdersRequest {
            user_pubkey: user_pubkey.to_string(),
        };
        let response = self.http_client.post(&url).json(&request).send().await?;
        self.handle_response(response).await
    }

    // =========================================================================
    // Price history endpoints
    // =========================================================================

    /// Get historical price data (candlesticks).
    pub async fn get_price_history(
        &self,
        params: PriceHistoryParams,
    ) -> ApiResult<PriceHistoryResponse> {
        let mut url = format!(
            "{}/api/price-history?orderbook_id={}",
            self.base_url, params.orderbook_id
        );

        if let Some(resolution) = params.resolution {
            url.push_str(&format!("&resolution={}", resolution));
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

        let response = self.http_client.get(&url).send().await?;
        self.handle_response(response).await
    }

    // =========================================================================
    // Trade endpoints
    // =========================================================================

    /// Get executed trades.
    pub async fn get_trades(&self, params: TradesParams) -> ApiResult<TradesResponse> {
        let mut url = format!(
            "{}/api/trades?orderbook_id={}",
            self.base_url, params.orderbook_id
        );

        if let Some(user_pubkey) = params.user_pubkey {
            url.push_str(&format!("&user_pubkey={}", user_pubkey));
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

        let response = self.http_client.get(&url).send().await?;
        self.handle_response(response).await
    }

    // =========================================================================
    // Admin endpoints
    // =========================================================================

    /// Admin health check endpoint.
    pub async fn admin_health_check(&self) -> ApiResult<AdminResponse> {
        let url = format!("{}/api/admin/test", self.base_url);
        let response = self.http_client.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// Create a new orderbook for a market.
    pub async fn create_orderbook(
        &self,
        request: CreateOrderbookRequest,
    ) -> ApiResult<CreateOrderbookResponse> {
        let url = format!("{}/api/admin/create-orderbook", self.base_url);
        let response = self.http_client.post(&url).json(&request).send().await?;
        self.handle_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::Resolution;

    #[test]
    fn test_client_creation() {
        let client = LightconeApiClient::new("https://api.lightcone.io");
        assert_eq!(client.base_url(), "https://api.lightcone.io");
    }

    #[test]
    fn test_client_builder() {
        let client = LightconeApiClient::builder("https://api.lightcone.io/")
            .timeout_secs(60)
            .header("X-Custom", "test")
            .build()
            .unwrap();

        // Base URL should have trailing slash removed
        assert_eq!(client.base_url(), "https://api.lightcone.io");
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
}
