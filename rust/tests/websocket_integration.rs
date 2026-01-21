//! Integration tests for the WebSocket client.
//!
//! These tests require a running WebSocket server and are marked as ignored by default.
//! Run with: `cargo test --test websocket_integration -- --ignored`

use futures_util::StreamExt;
use lightcone_sdk::websocket::*;

const TEST_WS_URL: &str = "ws://localhost:8081/ws";

/// Test basic connection
#[tokio::test]
#[ignore = "requires running WebSocket server"]
async fn test_connect() {
    let client = LightconeWebSocketClient::connect(TEST_WS_URL).await;
    assert!(client.is_ok(), "Failed to connect: {:?}", client.err());

    let client = client.unwrap();
    assert!(client.is_connected());
}

/// Test connection with custom config
#[tokio::test]
#[ignore = "requires running WebSocket server"]
async fn test_connect_with_config() {
    let config = WebSocketConfig {
        reconnect_attempts: 3,
        base_delay_ms: 500,
        max_delay_ms: 5000,
        ping_interval_secs: 15,
        pong_timeout_secs: 60,
        auto_reconnect: false,
        auto_resubscribe: false,
        auth_token: None,
    };

    let client = LightconeWebSocketClient::connect_with_config(TEST_WS_URL, config).await;
    assert!(client.is_ok());
}

/// Test subscribe to book updates
#[tokio::test]
#[ignore = "requires running WebSocket server"]
async fn test_subscribe_book_updates() {
    let mut client = LightconeWebSocketClient::connect(TEST_WS_URL)
        .await
        .expect("Failed to connect");

    // Subscribe to an orderbook
    let result = client
        .subscribe_book_updates(vec!["test_market:test_ob".to_string()])
        .await;

    assert!(result.is_ok(), "Failed to subscribe: {:?}", result.err());
}

/// Test subscribe to user events
#[tokio::test]
#[ignore = "requires running WebSocket server"]
async fn test_subscribe_user() {
    let mut client = LightconeWebSocketClient::connect(TEST_WS_URL)
        .await
        .expect("Failed to connect");

    let result = client.subscribe_user("test_user_pubkey".to_string()).await;
    assert!(result.is_ok(), "Failed to subscribe: {:?}", result.err());
}

/// Test subscribe to trades
#[tokio::test]
#[ignore = "requires running WebSocket server"]
async fn test_subscribe_trades() {
    let mut client = LightconeWebSocketClient::connect(TEST_WS_URL)
        .await
        .expect("Failed to connect");

    let result = client
        .subscribe_trades(vec!["test_market:test_ob".to_string()])
        .await;

    assert!(result.is_ok(), "Failed to subscribe: {:?}", result.err());
}

/// Test subscribe to price history
#[tokio::test]
#[ignore = "requires running WebSocket server"]
async fn test_subscribe_price_history() {
    let mut client = LightconeWebSocketClient::connect(TEST_WS_URL)
        .await
        .expect("Failed to connect");

    let result = client
        .subscribe_price_history("test_ob".to_string(), "1m".to_string(), true)
        .await;

    assert!(result.is_ok(), "Failed to subscribe: {:?}", result.err());
}

/// Test subscribe to market events
#[tokio::test]
#[ignore = "requires running WebSocket server"]
async fn test_subscribe_market() {
    let mut client = LightconeWebSocketClient::connect(TEST_WS_URL)
        .await
        .expect("Failed to connect");

    let result = client.subscribe_market("test_market".to_string()).await;
    assert!(result.is_ok(), "Failed to subscribe: {:?}", result.err());
}

/// Test ping
#[tokio::test]
#[ignore = "requires running WebSocket server"]
async fn test_ping() {
    let mut client = LightconeWebSocketClient::connect(TEST_WS_URL)
        .await
        .expect("Failed to connect");

    let result = client.ping().await;
    assert!(result.is_ok(), "Failed to ping: {:?}", result.err());

    // Wait for pong response
    while let Some(event) = client.next().await {
        if matches!(event, WsEvent::Pong) {
            return;
        }
    }

    panic!("Did not receive pong response");
}

/// Test receive book update event
#[tokio::test]
#[ignore = "requires running WebSocket server"]
async fn test_receive_book_update() {
    let mut client = LightconeWebSocketClient::connect(TEST_WS_URL)
        .await
        .expect("Failed to connect");

    client
        .subscribe_book_updates(vec!["test_market:test_ob".to_string()])
        .await
        .expect("Failed to subscribe");

    // Wait for initial snapshot
    let timeout = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while let Some(event) = client.next().await {
            match event {
                WsEvent::BookUpdate {
                    orderbook_id,
                    is_snapshot,
                } => {
                    println!("Received book update for {}, snapshot: {}", orderbook_id, is_snapshot);
                    return true;
                }
                WsEvent::Error { error } => {
                    println!("Error: {:?}", error);
                }
                _ => {}
            }
        }
        false
    });

    assert!(
        timeout.await.unwrap_or(false),
        "Did not receive book update within timeout"
    );
}

/// Test orderbook state management
#[tokio::test]
#[ignore = "requires running WebSocket server"]
async fn test_orderbook_state() {
    let mut client = LightconeWebSocketClient::connect(TEST_WS_URL)
        .await
        .expect("Failed to connect");

    let orderbook_id = "test_market:test_ob";
    client
        .subscribe_book_updates(vec![orderbook_id.to_string()])
        .await
        .expect("Failed to subscribe");

    // Wait for snapshot
    let timeout = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        while let Some(event) = client.next().await {
            if let WsEvent::BookUpdate { is_snapshot, .. } = event {
                if is_snapshot {
                    // Check that state was populated
                    if let Some(book) = client.get_orderbook(orderbook_id).await {
                        println!(
                            "Best bid: {:?}, Best ask: {:?}",
                            book.best_bid(),
                            book.best_ask()
                        );
                        return true;
                    }
                }
            }
        }
        false
    });

    assert!(
        timeout.await.unwrap_or(false),
        "Did not receive orderbook snapshot"
    );
}

/// Test disconnect
#[tokio::test]
#[ignore = "requires running WebSocket server"]
async fn test_disconnect() {
    let mut client = LightconeWebSocketClient::connect(TEST_WS_URL)
        .await
        .expect("Failed to connect");

    assert!(client.is_connected());

    client.disconnect().await.expect("Failed to disconnect");

    assert!(!client.is_connected());
}

/// Test unsubscribe
#[tokio::test]
#[ignore = "requires running WebSocket server"]
async fn test_unsubscribe() {
    let mut client = LightconeWebSocketClient::connect(TEST_WS_URL)
        .await
        .expect("Failed to connect");

    // Subscribe
    client
        .subscribe_book_updates(vec!["test_ob".to_string()])
        .await
        .expect("Failed to subscribe");

    // Unsubscribe
    let result = client
        .unsubscribe_book_updates(vec!["test_ob".to_string()])
        .await;

    assert!(result.is_ok(), "Failed to unsubscribe: {:?}", result.err());
}

// ============================================================================
// Unit tests that don't require a server
// ============================================================================

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(config.reconnect_attempts, 10);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.ping_interval_secs, 30);
        assert!(config.auto_reconnect);
        assert!(config.auto_resubscribe);
    }

    #[test]
    fn test_price_level_deserialization() {
        let json = r#"{"side": "bid", "price": "0.500000", "size": "0.001000"}"#;
        let level: PriceLevel = serde_json::from_str(json).unwrap();
        assert_eq!(level.side, "bid");
        assert_eq!(level.price, "0.500000");
        assert_eq!(level.size, "0.001000");
    }

    #[test]
    fn test_book_update_deserialization() {
        let json = r#"{
            "orderbook_id": "test_ob",
            "timestamp": "2024-01-01T00:00:00.000Z",
            "seq": 42,
            "bids": [{"side": "bid", "price": "0.500000", "size": "0.001000"}],
            "asks": [{"side": "ask", "price": "0.510000", "size": "0.000500"}],
            "is_snapshot": true
        }"#;

        let data: BookUpdateData = serde_json::from_str(json).unwrap();
        assert_eq!(data.orderbook_id, "test_ob");
        assert_eq!(data.seq, 42);
        assert!(data.is_snapshot);
        assert_eq!(data.bids.len(), 1);
        assert_eq!(data.asks.len(), 1);
    }

    #[test]
    fn test_trade_deserialization() {
        let json = r#"{
            "orderbook_id": "test_ob",
            "price": "0.505000",
            "size": "0.000250",
            "side": "bid",
            "timestamp": "2024-01-01T00:00:00.000Z",
            "trade_id": "trade123"
        }"#;

        let data: TradeData = serde_json::from_str(json).unwrap();
        assert_eq!(data.orderbook_id, "test_ob");
        assert_eq!(data.price, "0.505000");
        assert_eq!(data.size, "0.000250");
        assert_eq!(data.trade_id, "trade123");
    }

    #[test]
    fn test_parse_decimal() {
        use lightcone_sdk::shared::parse_decimal;

        assert_eq!(parse_decimal("0.500000").unwrap(), 0.5);
        assert_eq!(parse_decimal("1.000000").unwrap(), 1.0);
    }

    #[test]
    fn test_local_orderbook_basic() {
        let mut book = LocalOrderbook::new("test".to_string());
        assert!(!book.has_snapshot());

        // Apply snapshot
        let snapshot = BookUpdateData {
            orderbook_id: "test".to_string(),
            timestamp: "2024-01-01T00:00:00.000Z".to_string(),
            seq: 0,
            bids: vec![PriceLevel {
                side: "bid".to_string(),
                price: "0.500000".to_string(),
                size: "0.001000".to_string(),
            }],
            asks: vec![PriceLevel {
                side: "ask".to_string(),
                price: "0.510000".to_string(),
                size: "0.000500".to_string(),
            }],
            is_snapshot: true,
            resync: false,
            message: None,
        };

        book.apply_snapshot(&snapshot);

        assert!(book.has_snapshot());
        assert_eq!(book.best_bid(), Some(("0.500000".to_string(), "0.001000".to_string())));
        assert_eq!(book.best_ask(), Some(("0.510000".to_string(), "0.000500".to_string())));
        assert_eq!(book.spread(), Some("0.010000".to_string()));
        assert_eq!(book.midpoint(), Some("0.505000".to_string()));
    }

    #[test]
    fn test_user_state_basic() {
        let state = UserState::new("test_user".to_string());
        assert!(!state.has_snapshot());
        assert_eq!(state.order_count(), 0);
    }

    #[test]
    fn test_price_history_basic() {
        let history = PriceHistory::new("test_ob".to_string(), "1m".to_string(), true);
        assert!(!history.has_snapshot());
        assert_eq!(history.candle_count(), 0);
    }
}
