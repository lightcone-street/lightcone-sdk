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
use serde::{Deserialize, Serialize};

pub use subscriptions::{SubscribeParams, Subscription, UnsubscribeParams};

// ─── Outbound messages ───────────────────────────────────────────────────────

/// Messages sent from client to server.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum MessageOut {
    #[serde(rename = "subscribe")]
    Subscribe { params: SubscribeParams },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { params: UnsubscribeParams },
    #[serde(rename = "ping")]
    Ping,
}

// ─── Inbound messages ────────────────────────────────────────────────────────

/// Raw inbound message from the server.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageIn {
    #[serde(flatten)]
    pub kind: Kind,
}

/// The type of inbound WebSocket message.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum Kind {
    #[serde(rename = "book_update")]
    BookUpdate(BookUpdatePayload),
    #[serde(rename = "pong")]
    Pong,
    #[serde(rename = "user")]
    User(UserPayload),
    #[serde(rename = "error")]
    Error(WsErrorPayload),
    #[serde(rename = "price_history")]
    PriceHistory(PriceHistoryPayload),
    #[serde(rename = "trade")]
    Trade(TradePayload),
    #[serde(rename = "auth")]
    Auth(AuthPayload),
    #[serde(rename = "ticker")]
    Ticker(TickerPayload),
    #[serde(rename = "market")]
    Market(MarketPayload),
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookUpdatePayload {
    pub data: OrderBook,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserPayload {
    pub data: UserUpdate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WsErrorPayload {
    pub message: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceHistoryPayload {
    pub data: PriceHistory,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradePayload {
    pub data: WsTrade,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthPayload {
    pub data: AuthUpdate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TickerPayload {
    pub data: WsTickerData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarketPayload {
    pub data: MarketEvent,
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
}

/// Configuration for the WS client.
#[derive(Debug, Clone)]
pub struct WsConfig {
    pub url: String,
    pub reconnect: bool,
    pub reconnect_delay_ms: u64,
    pub ping_interval_ms: u64,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            url: crate::network::DEFAULT_WS_URL.to_string(),
            reconnect: true,
            reconnect_delay_ms: 2000,
            ping_interval_ms: 30_000,
        }
    }
}
