//! Integration tests for the Lightcone REST API client.
//!
//! These tests verify serialization/deserialization of API types and client functionality.
//! For live API tests, set the `LIGHTCONE_API_URL` environment variable.

use lightcone_sdk::api::*;
use lightcone_sdk::shared::Resolution;

// =============================================================================
// Type Serialization/Deserialization Tests
// =============================================================================

mod market_types {
    use super::*;

    #[test]
    fn test_market_status_deserialize() {
        let json = r#""Pending""#;
        let status: ApiMarketStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, ApiMarketStatus::Pending);

        let json = r#""Active""#;
        let status: ApiMarketStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, ApiMarketStatus::Active);

        let json = r#""Settled""#;
        let status: ApiMarketStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, ApiMarketStatus::Settled);
    }

    #[test]
    fn test_market_status_serialize() {
        let status = ApiMarketStatus::Active;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""Active""#);
    }

    #[test]
    fn test_market_status_default() {
        let status = ApiMarketStatus::default();
        assert_eq!(status, ApiMarketStatus::Pending);
    }

    #[test]
    fn test_outcome_deserialize() {
        let json = r#"{"index": 0, "name": "Yes", "thumbnail_url": null}"#;
        let outcome: Outcome = serde_json::from_str(json).unwrap();
        assert_eq!(outcome.index, 0);
        assert_eq!(outcome.name, "Yes");
        assert!(outcome.thumbnail_url.is_none());
    }

    #[test]
    fn test_markets_response_deserialize() {
        let json = r#"{
            "markets": [],
            "total": 0
        }"#;
        let response: MarketsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total, 0);
        assert!(response.markets.is_empty());
    }

    #[test]
    fn test_deposit_asset_deserialize() {
        let json = r#"{
            "display_name": "USDC",
            "token_symbol": "USDC",
            "symbol": "USDC",
            "deposit_asset": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "id": 1,
            "market_pubkey": "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs",
            "vault": "VaultAddress123",
            "num_outcomes": 2,
            "description": null,
            "icon_url": null,
            "metadata_uri": null,
            "decimals": 6,
            "conditional_tokens": [],
            "created_at": "2024-01-15T10:30:00Z"
        }"#;
        let asset: DepositAsset = serde_json::from_str(json).unwrap();
        assert_eq!(asset.display_name.as_deref(), Some("USDC"));
        assert_eq!(asset.decimals, 6);
        assert_eq!(asset.num_outcomes, 2);
    }
}

mod orderbook_types {
    use super::*;

    #[test]
    fn test_price_level_deserialize() {
        let json = r#"{"price": "0.500000", "size": "1.000000", "orders": 5}"#;
        let level: PriceLevel = serde_json::from_str(json).unwrap();
        assert_eq!(level.price, "0.500000");
        assert_eq!(level.size, "1.000000");
        assert_eq!(level.orders, 5);
    }

    #[test]
    fn test_orderbook_response_deserialize() {
        let json = r#"{
            "market_pubkey": "7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs",
            "orderbook_id": "ob_123",
            "bids": [{"price": "0.490000", "size": "0.000100", "orders": 1}],
            "asks": [{"price": "0.510000", "size": "0.000200", "orders": 2}],
            "best_bid": "0.490000",
            "best_ask": "0.510000",
            "spread": "0.020000",
            "tick_size": "0.001000"
        }"#;
        let response: OrderbookResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.orderbook_id, "ob_123");
        assert_eq!(response.best_bid, Some("0.490000".to_string()));
        assert_eq!(response.best_ask, Some("0.510000".to_string()));
        assert_eq!(response.spread, Some("0.020000".to_string()));
        assert_eq!(response.bids.len(), 1);
        assert_eq!(response.asks.len(), 1);
    }

    #[test]
    fn test_orderbook_response_empty() {
        let json = r#"{
            "market_pubkey": "market123",
            "orderbook_id": "ob_123",
            "bids": [],
            "asks": [],
            "best_bid": null,
            "best_ask": null,
            "spread": null,
            "tick_size": "0.001000"
        }"#;
        let response: OrderbookResponse = serde_json::from_str(json).unwrap();
        assert!(response.best_bid.is_none());
        assert!(response.best_ask.is_none());
        assert!(response.bids.is_empty());
    }
}

mod order_types {
    use super::*;

    #[test]
    fn test_order_side_conversion() {
        assert_eq!(ApiOrderSide::try_from(0).unwrap(), ApiOrderSide::Bid);
        assert_eq!(ApiOrderSide::try_from(1).unwrap(), ApiOrderSide::Ask);
        assert!(ApiOrderSide::try_from(2).is_err());
        assert_eq!(u32::from(ApiOrderSide::Bid), 0);
        assert_eq!(u32::from(ApiOrderSide::Ask), 1);
    }

    #[test]
    fn test_submit_order_request_serialize() {
        let request = SubmitOrderRequest {
            maker: "MakerPubkey123".to_string(),
            nonce: 1,
            market_pubkey: "MarketPubkey456".to_string(),
            base_token: "BaseToken789".to_string(),
            quote_token: "QuoteTokenABC".to_string(),
            side: 0,
            maker_amount: 1000000,
            taker_amount: 500000,
            expiration: 0,
            signature: "a".repeat(128),
            orderbook_id: "ob_123".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("MakerPubkey123"));
        assert!(json.contains("1000000"));
    }

    #[test]
    fn test_order_response_deserialize() {
        let json = r#"{
            "order_hash": "abc123def456",
            "status": "accepted",
            "remaining": "1.000000",
            "filled": "0.000000",
            "fills": []
        }"#;
        let response: OrderResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.order_hash, "abc123def456");
        assert_eq!(response.status, OrderStatus::Accepted);
        assert_eq!(response.remaining, "1.000000");
        assert_eq!(response.filled, "0.000000");
    }

    #[test]
    fn test_order_response_with_fills() {
        let json = r#"{
            "order_hash": "abc123",
            "status": "partial_fill",
            "remaining": "0.500000",
            "filled": "0.500000",
            "fills": [
                {
                    "counterparty": "CounterpartyPubkey",
                    "counterparty_order_hash": "xyz789",
                    "fill_amount": "0.500000",
                    "price": "0.500000",
                    "is_maker": false
                }
            ]
        }"#;
        let response: OrderResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.fills.len(), 1);
        assert_eq!(response.fills[0].fill_amount, "0.500000");
        assert!(!response.fills[0].is_maker);
    }

    #[test]
    fn test_cancel_response_deserialize() {
        let json = r#"{
            "status": "cancelled",
            "order_hash": "abc123",
            "remaining": "1.000000"
        }"#;
        let response: CancelResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "cancelled");
        assert_eq!(response.remaining, "1.000000");
    }

    #[test]
    fn test_cancel_all_response_deserialize() {
        let json = r#"{
            "status": "success",
            "user_pubkey": "UserPubkey123",
            "market_pubkey": null,
            "cancelled_order_hashes": ["hash1", "hash2"],
            "count": 2,
            "message": "Cancelled 2 orders"
        }"#;
        let response: CancelAllResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.count, 2);
        assert_eq!(response.cancelled_order_hashes.len(), 2);
    }

    #[test]
    fn test_user_order_deserialize() {
        let json = r#"{
            "order_hash": "abc123",
            "market_pubkey": "market456",
            "orderbook_id": "ob_789",
            "side": 0,
            "maker_amount": "1.000000",
            "taker_amount": "0.500000",
            "remaining": "1.000000",
            "filled": "0.000000",
            "price": "0.500000",
            "created_at": "2024-01-15T10:30:00Z",
            "expiration": 0
        }"#;
        let order: UserOrder = serde_json::from_str(json).unwrap();
        assert_eq!(order.side, ApiOrderSide::Bid);
        assert_eq!(order.price, "0.500000");
    }
}

mod position_types {
    use super::*;

    #[test]
    fn test_outcome_balance_deserialize() {
        let json = r#"{
            "outcome_index": 0,
            "conditional_token": "TokenAddress123",
            "balance": "1.000000",
            "balance_idle": "0.500000",
            "balance_on_book": "0.500000"
        }"#;
        let balance: OutcomeBalance = serde_json::from_str(json).unwrap();
        assert_eq!(balance.outcome_index, 0);
        assert_eq!(balance.balance, "1.000000");
        assert_eq!(balance.balance_idle, "0.500000");
        assert_eq!(balance.balance_on_book, "0.500000");
    }

    #[test]
    fn test_position_deserialize() {
        let json = r#"{
            "id": 1,
            "position_pubkey": "PositionPDA123",
            "owner": "OwnerPubkey456",
            "market_pubkey": "MarketPubkey789",
            "outcomes": [],
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z"
        }"#;
        let position: Position = serde_json::from_str(json).unwrap();
        assert_eq!(position.id, 1);
        assert_eq!(position.owner, "OwnerPubkey456");
    }

    #[test]
    fn test_positions_response_deserialize() {
        let json = r#"{
            "owner": "OwnerPubkey123",
            "total_markets": 2,
            "positions": []
        }"#;
        let response: PositionsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total_markets, 2);
    }
}

mod trade_types {
    use super::*;

    #[test]
    fn test_trade_deserialize() {
        let json = r#"{
            "id": 123,
            "orderbook_id": "ob_456",
            "taker_pubkey": "TakerPubkey",
            "maker_pubkey": "MakerPubkey",
            "side": "BID",
            "size": "1.000000",
            "price": "0.500000",
            "taker_fee": "0.001000",
            "maker_fee": "0.000500",
            "executed_at": 1705315800000
        }"#;
        let trade: Trade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.id, 123);
        assert_eq!(trade.side, ApiTradeSide::Bid);
        assert_eq!(trade.size, "1.000000");
    }

    #[test]
    fn test_trades_params_builder() {
        let params = TradesParams::new("ob_123")
            .with_user("user456")
            .with_time_range(1000, 2000)
            .with_cursor(50)
            .with_limit(100);

        assert_eq!(params.orderbook_id, "ob_123");
        assert_eq!(params.user_pubkey, Some("user456".to_string()));
        assert_eq!(params.from, Some(1000));
        assert_eq!(params.to, Some(2000));
        assert_eq!(params.cursor, Some(50));
        assert_eq!(params.limit, Some(100));
    }

    #[test]
    fn test_trades_response_deserialize() {
        let json = r#"{
            "orderbook_id": "ob_123",
            "trades": [],
            "next_cursor": 100,
            "has_more": true
        }"#;
        let response: TradesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.next_cursor, Some(100));
        assert!(response.has_more);
    }
}

mod price_history_types {
    use super::*;

    #[test]
    fn test_resolution_serialize() {
        assert_eq!(serde_json::to_string(&Resolution::OneMinute).unwrap(), r#""1m""#);
        assert_eq!(serde_json::to_string(&Resolution::FiveMinutes).unwrap(), r#""5m""#);
        assert_eq!(serde_json::to_string(&Resolution::FifteenMinutes).unwrap(), r#""15m""#);
        assert_eq!(serde_json::to_string(&Resolution::OneHour).unwrap(), r#""1h""#);
        assert_eq!(serde_json::to_string(&Resolution::FourHours).unwrap(), r#""4h""#);
        assert_eq!(serde_json::to_string(&Resolution::OneDay).unwrap(), r#""1d""#);
    }

    #[test]
    fn test_resolution_deserialize() {
        assert_eq!(serde_json::from_str::<Resolution>(r#""1m""#).unwrap(), Resolution::OneMinute);
        assert_eq!(serde_json::from_str::<Resolution>(r#""1h""#).unwrap(), Resolution::OneHour);
    }

    #[test]
    fn test_resolution_as_str() {
        assert_eq!(Resolution::OneMinute.as_str(), "1m");
        assert_eq!(Resolution::OneHour.as_str(), "1h");
        assert_eq!(Resolution::OneDay.as_str(), "1d");
    }

    #[test]
    fn test_resolution_default() {
        assert_eq!(Resolution::default(), Resolution::OneMinute);
    }

    #[test]
    fn test_price_point_midpoint_only() {
        let json = r#"{"t": 1705315800000, "m": "0.500000"}"#;
        let point: PricePoint = serde_json::from_str(json).unwrap();
        assert_eq!(point.timestamp, 1705315800000);
        assert_eq!(point.midpoint, "0.500000");
        assert!(point.open.is_none());
    }

    #[test]
    fn test_price_point_with_ohlcv() {
        let json = r#"{
            "t": 1705315800000,
            "m": "0.500000",
            "o": "0.490000",
            "h": "0.520000",
            "l": "0.480000",
            "c": "0.510000",
            "v": "1.000000",
            "bb": "0.495000",
            "ba": "0.505000"
        }"#;
        let point: PricePoint = serde_json::from_str(json).unwrap();
        assert_eq!(point.open, Some("0.490000".to_string()));
        assert_eq!(point.high, Some("0.520000".to_string()));
        assert_eq!(point.low, Some("0.480000".to_string()));
        assert_eq!(point.close, Some("0.510000".to_string()));
        assert_eq!(point.volume, Some("1.000000".to_string()));
        assert_eq!(point.best_bid, Some("0.495000".to_string()));
        assert_eq!(point.best_ask, Some("0.505000".to_string()));
    }

    #[test]
    fn test_price_history_params_builder() {
        let params = PriceHistoryParams::new("ob_123")
            .with_resolution(Resolution::OneHour)
            .with_time_range(1000, 2000)
            .with_cursor(50)
            .with_limit(500)
            .with_ohlcv();

        assert_eq!(params.orderbook_id, "ob_123");
        assert_eq!(params.resolution, Some(Resolution::OneHour));
        assert_eq!(params.from, Some(1000));
        assert_eq!(params.to, Some(2000));
        assert_eq!(params.cursor, Some(50));
        assert_eq!(params.limit, Some(500));
        assert_eq!(params.include_ohlcv, Some(true));
    }

    #[test]
    fn test_price_history_response_deserialize() {
        let json = r#"{
            "orderbook_id": "ob_123",
            "resolution": "1h",
            "include_ohlcv": false,
            "prices": [{"t": 1705315800000, "m": "0.500000"}],
            "next_cursor": null,
            "has_more": false
        }"#;
        let response: PriceHistoryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.resolution, "1h");
        assert!(!response.include_ohlcv);
        assert_eq!(response.prices.len(), 1);
        assert!(!response.has_more);
    }
}

mod admin_types {
    use super::*;

    #[test]
    fn test_admin_response_deserialize() {
        let json = r#"{
            "status": "success",
            "message": "Admin endpoint is working"
        }"#;
        let response: AdminResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.status, "success");
    }

    #[test]
    fn test_create_orderbook_request_builder() {
        let request = CreateOrderbookRequest::new("market123", "base456", "quote789")
            .with_tick_size(500);

        assert_eq!(request.market_pubkey, "market123");
        assert_eq!(request.base_token, "base456");
        assert_eq!(request.quote_token, "quote789");
        assert_eq!(request.tick_size, Some(500));
    }

    #[test]
    fn test_create_orderbook_request_serialize() {
        let request = CreateOrderbookRequest::new("market123", "base456", "quote789");
        let json = serde_json::to_string(&request).unwrap();

        // tick_size should be omitted when None
        assert!(!json.contains("tick_size"));
        assert!(json.contains("market123"));
    }

    #[test]
    fn test_create_orderbook_response_deserialize() {
        let json = r#"{
            "status": "success",
            "orderbook_id": "ob_new_123",
            "market_pubkey": "market123",
            "base_token": "base456",
            "quote_token": "quote789",
            "tick_size": 1000,
            "message": "Orderbook created successfully"
        }"#;
        let response: CreateOrderbookResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.orderbook_id, "ob_new_123");
        assert_eq!(response.tick_size, 1000);
    }
}

mod error_types {
    use super::*;

    #[test]
    fn test_error_response_standard_format() {
        let json = r#"{
            "status": "error",
            "message": "Market not found",
            "details": "No market with pubkey xyz123"
        }"#;
        let response: ErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.get_message(), "Market not found");
    }

    #[test]
    fn test_error_response_alternative_format() {
        let json = r#"{
            "error": "Invalid signature format"
        }"#;
        let response: ErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.get_message(), "Invalid signature format");
    }

    #[test]
    fn test_error_response_fallback() {
        let json = r#"{}"#;
        let response: ErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.get_message(), "Unknown error");
    }
}

mod client_tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LightconeApiClient::new("https://api.lightcone.io").unwrap();
        assert_eq!(client.base_url(), "https://api.lightcone.io");
    }

    #[test]
    fn test_client_strips_trailing_slash() {
        let client = LightconeApiClient::new("https://api.lightcone.io/").unwrap();
        assert_eq!(client.base_url(), "https://api.lightcone.io");
    }

    #[test]
    fn test_client_builder() {
        let client = LightconeApiClient::builder("https://api.lightcone.io")
            .timeout_secs(60)
            .header("X-Custom-Header", "test-value")
            .build()
            .unwrap();

        assert_eq!(client.base_url(), "https://api.lightcone.io");
    }

    #[test]
    fn test_api_error_display() {
        let err = ApiError::NotFound(ErrorResponse::from_text("Market xyz not found".to_string()));
        assert_eq!(format!("{}", err), "Not found: Market xyz not found");

        let err = ApiError::BadRequest(ErrorResponse {
            status: None,
            message: Some("Failed to submit order".to_string()),
            details: Some("insufficient balance".to_string()),
        });
        assert_eq!(
            format!("{}", err),
            "Bad request: Failed to submit order: insufficient balance"
        );

        let err = ApiError::UnexpectedStatus(
            418,
            ErrorResponse::from_text("I'm a teapot".to_string()),
        );
        assert_eq!(format!("{}", err), "Unexpected status 418: I'm a teapot");
    }

    #[test]
    fn test_api_error_response_accessor() {
        let err = ApiError::NotFound(ErrorResponse {
            status: Some("error".to_string()),
            message: Some("Market not found".to_string()),
            details: Some("No market with pubkey xyz".to_string()),
        });
        let resp = err.error_response().unwrap();
        assert_eq!(resp.message.as_deref(), Some("Market not found"));
        assert_eq!(resp.details.as_deref(), Some("No market with pubkey xyz"));
        assert_eq!(err.status_code(), Some(404));
    }

    #[test]
    fn test_api_error_status_code() {
        assert_eq!(
            ApiError::NotFound(ErrorResponse::from_text("x".into())).status_code(),
            Some(404)
        );
        assert_eq!(
            ApiError::BadRequest(ErrorResponse::from_text("x".into())).status_code(),
            Some(400)
        );
        assert_eq!(
            ApiError::Forbidden(ErrorResponse::from_text("x".into())).status_code(),
            Some(403)
        );
        assert_eq!(
            ApiError::RateLimited(ErrorResponse::from_text("x".into())).status_code(),
            Some(429)
        );
        assert_eq!(
            ApiError::Unauthorized(ErrorResponse::from_text("x".into())).status_code(),
            Some(401)
        );
        assert_eq!(
            ApiError::Conflict(ErrorResponse::from_text("x".into())).status_code(),
            Some(409)
        );
        assert_eq!(
            ApiError::ServerError(ErrorResponse::from_text("x".into())).status_code(),
            Some(500)
        );
        assert_eq!(
            ApiError::UnexpectedStatus(418, ErrorResponse::from_text("x".into())).status_code(),
            Some(418)
        );
        assert_eq!(ApiError::Deserialize("x".into()).status_code(), None);
        assert_eq!(ApiError::InvalidParameter("x".into()).status_code(), None);
    }
}

// =============================================================================
// Live API Tests (require LIGHTCONE_API_URL environment variable)
// =============================================================================

#[cfg(feature = "live_tests")]
mod live_tests {
    use super::*;

    fn get_client() -> Option<LightconeApiClient> {
        std::env::var("LIGHTCONE_API_URL")
            .ok()
            .map(|url| LightconeApiClient::new(&url))
    }

    #[tokio::test]
    async fn test_live_health_check() {
        let Some(client) = get_client() else {
            println!("Skipping live test: LIGHTCONE_API_URL not set");
            return;
        };

        let result = client.health_check().await;
        assert!(result.is_ok(), "Health check failed: {:?}", result);
    }

    #[tokio::test]
    async fn test_live_get_markets() {
        let Some(client) = get_client() else {
            println!("Skipping live test: LIGHTCONE_API_URL not set");
            return;
        };

        let result = client.get_markets().await;
        assert!(result.is_ok(), "Get markets failed: {:?}", result);
    }
}
