//! WebSocket layer — messages, subscriptions, events.
//!
//! The actual WS transport is compile-time dispatched:
//! - `ws-native` feature → `tokio-tungstenite` (native.rs)
//! - `ws-wasm` feature → `web-sys::WebSocket` (wasm.rs)
//!
//! Both export the same `WsClient` type with identical methods.
//! This module defines the shared message/event types.

pub mod subscriptions;

#[cfg(feature = "ws-native")]
pub mod native;

#[cfg(feature = "ws-wasm")]
pub mod wasm;

use crate::domain::market::wire::MarketEvent;
use crate::domain::order::wire::{AuthUpdate, UserUpdate};
use crate::domain::orderbook::wire::{OrderBook, WsTickerData};
use crate::domain::price_history::wire::PriceHistory;
use crate::domain::trade::wire::WsTrade;
use crate::shared::{OrderBookId, PubkeyStr, Resolution};
use serde::{Deserialize, Serialize};

pub use subscriptions::{SubscribeParams, Subscription, UnsubscribeParams};

// ─── Outbound messages ───────────────────────────────────────────────────────

/// Messages sent from client to server.
///
/// Wire format: `{"method": "subscribe", "params": {...}}`
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "method", content = "params", rename_all = "lowercase")]
pub enum MessageOut {
    Subscribe(SubscribeParams),
    Unsubscribe(UnsubscribeParams),
    Ping,
}

impl From<SubscribeParams> for MessageOut {
    fn from(p: SubscribeParams) -> Self {
        MessageOut::Subscribe(p)
    }
}

impl From<UnsubscribeParams> for MessageOut {
    fn from(p: UnsubscribeParams) -> Self {
        MessageOut::Unsubscribe(p)
    }
}

impl std::fmt::Display for MessageOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, "<serialization error>"),
        }
    }
}

impl MessageOut {
    pub fn ping() -> MessageOut {
        MessageOut::Ping
    }

    pub fn subscribe_books(orderbook_ids: Vec<OrderBookId>) -> MessageOut {
        SubscribeParams::Books { orderbook_ids }.into()
    }

    pub fn unsubscribe_books(orderbook_ids: Vec<OrderBookId>) -> MessageOut {
        UnsubscribeParams::Books { orderbook_ids }.into()
    }

    pub fn subscribe_trades(orderbook_ids: Vec<OrderBookId>) -> MessageOut {
        SubscribeParams::Trades { orderbook_ids }.into()
    }

    pub fn unsubscribe_trades(orderbook_ids: Vec<OrderBookId>) -> MessageOut {
        UnsubscribeParams::Trades { orderbook_ids }.into()
    }

    pub fn subscribe_user(wallet_address: PubkeyStr) -> MessageOut {
        SubscribeParams::User { wallet_address }.into()
    }

    pub fn unsubscribe_user(wallet_address: PubkeyStr) -> MessageOut {
        UnsubscribeParams::User { wallet_address }.into()
    }

    pub fn subscribe_price_history(
        orderbook_id: OrderBookId,
        resolution: Resolution,
    ) -> MessageOut {
        SubscribeParams::PriceHistory {
            orderbook_id,
            resolution,
            include_ohlcv: false,
        }
        .into()
    }

    pub fn unsubscribe_price_history(
        orderbook_id: OrderBookId,
        resolution: Resolution,
    ) -> MessageOut {
        UnsubscribeParams::PriceHistory {
            orderbook_id,
            resolution,
        }
        .into()
    }

    pub fn subscribe_ticker(orderbook_ids: Vec<OrderBookId>) -> MessageOut {
        SubscribeParams::Ticker { orderbook_ids }.into()
    }

    pub fn unsubscribe_ticker(orderbook_ids: Vec<OrderBookId>) -> MessageOut {
        UnsubscribeParams::Ticker { orderbook_ids }.into()
    }

    pub fn subscribe_market(market_pubkey: PubkeyStr) -> MessageOut {
        SubscribeParams::Market { market_pubkey }.into()
    }

    pub fn unsubscribe_market(market_pubkey: PubkeyStr) -> MessageOut {
        UnsubscribeParams::Market { market_pubkey }.into()
    }
}

// ─── Inbound messages ────────────────────────────────────────────────────────

/// Raw inbound message from the server.
///
/// Backend sends: `{"type": "<channel>", "version": 0.1, "data": <payload>}`
#[derive(Debug, Clone, Deserialize)]
pub struct MessageIn {
    #[serde(flatten)]
    pub kind: Kind,
    pub version: f32,
}

/// Discriminated inbound message types.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Kind {
    #[serde(rename = "book_update")]
    BookUpdate(OrderBook),
    #[serde(rename = "pong")]
    Pong(Pong),
    #[serde(rename = "user")]
    User(UserUpdate),
    #[serde(rename = "error")]
    Error(WsError),
    #[serde(rename = "price_history")]
    PriceHistory(PriceHistory),
    #[serde(rename = "trades")]
    Trade(WsTrade),
    #[serde(rename = "auth")]
    Auth(AuthUpdate),
    #[serde(rename = "ticker")]
    Ticker(WsTickerData),
    #[serde(rename = "market")]
    Market(MarketEvent),
}

#[derive(Deserialize, Debug, Clone)]
pub struct Pong {}

/// Server-side WebSocket error with full diagnostic info from backend.
#[derive(Deserialize, Debug, Clone)]
pub struct WsError {
    pub error: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub orderbook_id: Option<String>,
    #[serde(default)]
    pub wallet_address: Option<String>,
    #[serde(default)]
    pub hint: Option<String>,
    #[serde(default)]
    pub details: Option<String>,
}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(code) = &self.code {
            write!(f, "[{}] {}", code, self.error)
        } else {
            write!(f, "{}", self.error)
        }
    }
}

// ─── WsEvent ─────────────────────────────────────────────────────────────────

/// High-level events emitted by the WS client to the consumer.
#[derive(Debug, Clone)]
pub enum WsEvent {
    /// A parsed message from the server.
    Message(Kind),
    /// Connection established.
    Connected,
    /// Connection lost (may trigger reconnect).
    Disconnected { code: Option<u16>, reason: String },
    /// A deserialization or protocol error.
    Error(String),
    /// All automatic reconnect attempts exhausted.
    MaxReconnectReached,
}

// ─── WsConfig ────────────────────────────────────────────────────────────────

/// Configuration for the WS client.
#[derive(Debug, Clone)]
pub struct WsConfig {
    pub url: String,
    pub reconnect: bool,
    pub max_reconnect_attempts: u32,
    pub base_reconnect_delay_ms: u32,
    pub ping_interval_ms: u32,
    pub pong_timeout_ms: u32,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            url: crate::network::DEFAULT_WS_URL.to_string(),
            reconnect: true,
            max_reconnect_attempts: 10,
            base_reconnect_delay_ms: 1_000,
            ping_interval_ms: 30_000,
            pong_timeout_ms: 1_000,
        }
    }
}

/// WebSocket connection states (W3C readyState values).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReadyState {
    Connecting = 0,
    Open = 1,
    Closing = 2,
    Closed = 3,
}

impl From<u16> for ReadyState {
    fn from(value: u16) -> Self {
        match value {
            0 => ReadyState::Connecting,
            1 => ReadyState::Open,
            2 => ReadyState::Closing,
            3 => ReadyState::Closed,
            _ => ReadyState::Closed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ready_state_from_u16() {
        assert!(matches!(ReadyState::from(0), ReadyState::Connecting));
        assert!(matches!(ReadyState::from(1), ReadyState::Open));
        assert!(matches!(ReadyState::from(2), ReadyState::Closing));
        assert!(matches!(ReadyState::from(3), ReadyState::Closed));
        assert!(matches!(ReadyState::from(99), ReadyState::Closed));
    }

    #[test]
    fn test_message_out_subscribe_serialization() {
        let msg = MessageOut::subscribe_books(vec![OrderBookId::new("abc")]);
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["method"], "subscribe");
        assert_eq!(parsed["params"]["type"], "book_update");
        assert_eq!(parsed["params"]["orderbook_ids"][0], "abc");
    }

    #[test]
    fn test_message_out_unsubscribe_serialization() {
        let msg = MessageOut::unsubscribe_books(vec![OrderBookId::new("abc")]);
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["method"], "unsubscribe");
        assert_eq!(parsed["params"]["type"], "book_update");
    }

    #[test]
    fn test_message_out_ping_serialization() {
        let msg = MessageOut::ping();
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["method"], "ping");
    }

    #[test]
    fn test_kind_book_update_deserialization() {
        let json = r#"{"type": "book_update", "data": {"orderbook_id": "abc", "bids": [], "asks": []}, "version": 0.1}"#;
        let msg: MessageIn = serde_json::from_str(json).unwrap();
        assert!(matches!(msg.kind, Kind::BookUpdate(_)));
    }

    #[test]
    fn test_kind_pong_deserialization() {
        let json = r#"{"type": "pong", "data": {}, "version": 0.1}"#;
        let msg: MessageIn = serde_json::from_str(json).unwrap();
        assert!(matches!(msg.kind, Kind::Pong(_)));
    }

    #[test]
    fn test_kind_error_deserialization() {
        let json = r#"{"type": "error", "data": {"error": "something broke", "code": "ENGINE_UNAVAILABLE"}, "version": 0.1}"#;
        let msg: MessageIn = serde_json::from_str(json).unwrap();
        match msg.kind {
            Kind::Error(e) => {
                assert_eq!(e.error, "something broke");
                assert_eq!(e.code.as_deref(), Some("ENGINE_UNAVAILABLE"));
                assert_eq!(e.to_string(), "[ENGINE_UNAVAILABLE] something broke");
            }
            _ => panic!("expected Kind::Error"),
        }
    }

    #[test]
    fn test_kind_error_minimal_deserialization() {
        let json = r#"{"type": "error", "data": {"error": "bad"}, "version": 0.1}"#;
        let msg: MessageIn = serde_json::from_str(json).unwrap();
        match msg.kind {
            Kind::Error(e) => {
                assert_eq!(e.error, "bad");
                assert!(e.code.is_none());
            }
            _ => panic!("expected Kind::Error"),
        }
    }

    #[test]
    fn test_kind_trades_deserialization() {
        let json = r#"{"type": "trades", "data": {"orderbook_id": "abc", "trade_id": "t1", "timestamp": "2025-01-01T00:00:00Z", "price": "1.5", "size": "100", "side": "bid"}, "version": 0.1}"#;
        let msg: MessageIn = serde_json::from_str(json).unwrap();
        assert!(matches!(msg.kind, Kind::Trade(_)));
    }

    #[test]
    fn test_ws_config_default() {
        let config = WsConfig::default();
        assert_eq!(config.max_reconnect_attempts, 10);
        assert_eq!(config.ping_interval_ms, 30_000);
        assert_eq!(config.pong_timeout_ms, 1_000);
        assert!(config.reconnect);
    }
}
