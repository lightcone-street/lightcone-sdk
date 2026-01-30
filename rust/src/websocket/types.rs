//! Message types for the Lightcone WebSocket protocol.
//!
//! This module contains all request and response types for the WebSocket API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// REQUEST TYPES (Client → Server)
// ============================================================================

/// Subscribe/Unsubscribe request wrapper
#[derive(Debug, Clone, Serialize)]
pub struct WsRequest {
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<SubscribeParams>,
}

impl WsRequest {
    /// Create a subscribe request
    pub fn subscribe(params: SubscribeParams) -> Self {
        Self {
            method: "subscribe".to_string(),
            params: Some(params),
        }
    }

    /// Create an unsubscribe request
    pub fn unsubscribe(params: SubscribeParams) -> Self {
        Self {
            method: "unsubscribe".to_string(),
            params: Some(params),
        }
    }

    /// Create a ping request
    pub fn ping() -> Self {
        Self {
            method: "ping".to_string(),
            params: None,
        }
    }
}

/// Subscription parameters (polymorphic)
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SubscribeParams {
    /// Subscribe to orderbook updates
    BookUpdate {
        #[serde(rename = "type")]
        type_: &'static str,
        orderbook_ids: Vec<String>,
    },
    /// Subscribe to trade executions
    Trades {
        #[serde(rename = "type")]
        type_: &'static str,
        orderbook_ids: Vec<String>,
    },
    /// Subscribe to user events
    User {
        #[serde(rename = "type")]
        type_: &'static str,
        user: String,
    },
    /// Subscribe to price history
    PriceHistory {
        #[serde(rename = "type")]
        type_: &'static str,
        orderbook_id: String,
        resolution: String,
        include_ohlcv: bool,
    },
    /// Subscribe to market events
    Market {
        #[serde(rename = "type")]
        type_: &'static str,
        market_pubkey: String,
    },
}

impl SubscribeParams {
    /// Create book update subscription params
    pub fn book_update(orderbook_ids: Vec<String>) -> Self {
        Self::BookUpdate {
            type_: "book_update",
            orderbook_ids,
        }
    }

    /// Create trades subscription params
    pub fn trades(orderbook_ids: Vec<String>) -> Self {
        Self::Trades {
            type_: "trades",
            orderbook_ids,
        }
    }

    /// Create user subscription params
    pub fn user(user: String) -> Self {
        Self::User {
            type_: "user",
            user,
        }
    }

    /// Create price history subscription params
    pub fn price_history(orderbook_id: String, resolution: String, include_ohlcv: bool) -> Self {
        Self::PriceHistory {
            type_: "price_history",
            orderbook_id,
            resolution,
            include_ohlcv,
        }
    }

    /// Create market subscription params
    pub fn market(market_pubkey: String) -> Self {
        Self::Market {
            type_: "market",
            market_pubkey,
        }
    }

    /// Get the subscription type as a string
    pub fn subscription_type(&self) -> &'static str {
        match self {
            Self::BookUpdate { .. } => "book_update",
            Self::Trades { .. } => "trades",
            Self::User { .. } => "user",
            Self::PriceHistory { .. } => "price_history",
            Self::Market { .. } => "market",
        }
    }
}

// ============================================================================
// RESPONSE TYPES (Server → Client)
// ============================================================================

/// Raw message wrapper for initial parsing
#[derive(Debug, Clone, Deserialize)]
pub struct RawWsMessage {
    #[serde(rename = "type")]
    pub type_: String,
    pub version: f32,
    pub data: serde_json::Value,
}

/// Generic WebSocket message wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct WsMessage<T> {
    #[serde(rename = "type")]
    pub type_: String,
    pub version: f32,
    pub data: T,
}

// ============================================================================
// BOOK UPDATE TYPES
// ============================================================================

/// Orderbook snapshot/delta data
#[derive(Debug, Clone, Deserialize)]
pub struct BookUpdateData {
    pub orderbook_id: String,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub sequence: u64,
    #[serde(default)]
    pub bids: Vec<PriceLevel>,
    #[serde(default)]
    pub asks: Vec<PriceLevel>,
    #[serde(default)]
    pub is_snapshot: bool,
    #[serde(default)]
    pub resync: bool,
    #[serde(default)]
    pub message: Option<String>,
}

/// Price level in the orderbook
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PriceLevel {
    pub side: String,
    /// Price as decimal string (e.g., "0.500000")
    pub price: String,
    /// Size as decimal string
    pub size: String,
}

// ============================================================================
// TRADE TYPES
// ============================================================================

/// Trade execution data
#[derive(Debug, Clone, Deserialize)]
pub struct TradeData {
    pub orderbook_id: String,
    /// Price as decimal string
    pub price: String,
    /// Size as decimal string
    pub size: String,
    pub side: String,
    pub timestamp: String,
    pub trade_id: String,
    pub sequence: u64,
}

// ============================================================================
// USER EVENT TYPES
// ============================================================================

/// User event data (snapshot, order_update, balance_update)
#[derive(Debug, Clone, Deserialize)]
pub struct UserEventData {
    pub event_type: String,
    #[serde(default)]
    pub orders: Vec<Order>,
    #[serde(default)]
    pub balances: HashMap<String, BalanceEntry>,
    #[serde(default)]
    pub order: Option<OrderUpdate>,
    #[serde(default)]
    pub balance: Option<Balance>,
    #[serde(default)]
    pub market_pubkey: Option<String>,
    #[serde(default)]
    pub orderbook_id: Option<String>,
    #[serde(default)]
    pub deposit_mint: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// User order from snapshot
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Order {
    pub order_hash: String,
    pub market_pubkey: String,
    pub orderbook_id: String,
    /// 0 = BUY, 1 = SELL
    pub side: i32,
    /// Maker amount as decimal string
    pub maker_amount: String,
    /// Taker amount as decimal string
    pub taker_amount: String,
    /// Remaining amount as decimal string
    pub remaining: String,
    /// Filled amount as decimal string
    pub filled: String,
    /// Price as decimal string
    pub price: String,
    pub created_at: i64,
    pub expiration: i64,
}

/// Order update from real-time event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderUpdate {
    pub order_hash: String,
    /// Price as decimal string
    pub price: String,
    /// Fill amount as decimal string
    pub fill_amount: String,
    /// Remaining amount as decimal string
    pub remaining: String,
    /// Filled amount as decimal string
    pub filled: String,
    /// 0 = BUY, 1 = SELL
    pub side: i32,
    pub is_maker: bool,
    pub created_at: i64,
    #[serde(default)]
    pub balance: Option<Balance>,
}

/// Balance containing outcome balances
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Balance {
    pub outcomes: Vec<OutcomeBalance>,
}

/// Individual outcome balance
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutcomeBalance {
    pub outcome_index: i32,
    pub mint: String,
    /// Idle balance as decimal string
    pub idle: String,
    /// On-book balance as decimal string
    pub on_book: String,
}

/// Balance entry from user snapshot
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceEntry {
    pub market_pubkey: String,
    pub deposit_mint: String,
    pub outcomes: Vec<OutcomeBalance>,
}

// ============================================================================
// PRICE HISTORY TYPES
// ============================================================================

/// Price history data (snapshot, update, heartbeat)
#[derive(Debug, Clone, Deserialize)]
pub struct PriceHistoryData {
    pub event_type: String,
    #[serde(default)]
    pub orderbook_id: Option<String>,
    #[serde(default)]
    pub resolution: Option<String>,
    #[serde(default)]
    pub include_ohlcv: Option<bool>,
    #[serde(default)]
    pub prices: Vec<Candle>,
    #[serde(default)]
    pub last_timestamp: Option<i64>,
    #[serde(default)]
    pub server_time: Option<i64>,
    #[serde(default)]
    pub last_processed: Option<i64>,
    // For updates (inline candle data)
    #[serde(default)]
    pub t: Option<i64>,
    #[serde(default)]
    pub o: Option<String>,
    #[serde(default)]
    pub h: Option<String>,
    #[serde(default)]
    pub l: Option<String>,
    #[serde(default)]
    pub c: Option<String>,
    #[serde(default)]
    pub v: Option<String>,
    #[serde(default)]
    pub m: Option<String>,
    #[serde(default)]
    pub bb: Option<String>,
    #[serde(default)]
    pub ba: Option<String>,
}

impl PriceHistoryData {
    /// Convert inline candle data to a Candle struct (for update events)
    pub fn to_candle(&self) -> Option<Candle> {
        self.t.map(|t| Candle {
            t,
            o: self.o.clone(),
            h: self.h.clone(),
            l: self.l.clone(),
            c: self.c.clone(),
            v: self.v.clone(),
            m: self.m.clone(),
            bb: self.bb.clone(),
            ba: self.ba.clone(),
        })
    }
}

/// OHLCV candle data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Candle {
    /// Timestamp (Unix ms)
    pub t: i64,
    /// Open price as decimal string (null if no trades)
    #[serde(default)]
    pub o: Option<String>,
    /// High price as decimal string (null if no trades)
    #[serde(default)]
    pub h: Option<String>,
    /// Low price as decimal string (null if no trades)
    #[serde(default)]
    pub l: Option<String>,
    /// Close price as decimal string (null if no trades)
    #[serde(default)]
    pub c: Option<String>,
    /// Volume as decimal string (null if no trades)
    #[serde(default)]
    pub v: Option<String>,
    /// Midpoint: (best_bid + best_ask) / 2 as decimal string
    #[serde(default)]
    pub m: Option<String>,
    /// Best bid price as decimal string
    #[serde(default)]
    pub bb: Option<String>,
    /// Best ask price as decimal string
    #[serde(default)]
    pub ba: Option<String>,
}

// ============================================================================
// MARKET EVENT TYPES
// ============================================================================

/// Market event data
#[derive(Debug, Clone, Deserialize)]
pub struct MarketEventData {
    /// Event type: "orderbook_created", "settled", "opened", "paused"
    pub event_type: String,
    pub market_pubkey: String,
    #[serde(default)]
    pub orderbook_id: Option<String>,
    pub timestamp: String,
}

/// Market event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketEventType {
    OrderbookCreated,
    Settled,
    Opened,
    Paused,
    Unknown,
}

impl From<&str> for MarketEventType {
    fn from(s: &str) -> Self {
        match s {
            "orderbook_created" => Self::OrderbookCreated,
            "settled" => Self::Settled,
            "opened" => Self::Opened,
            "paused" => Self::Paused,
            _ => Self::Unknown,
        }
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Error response from server
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorData {
    pub error: String,
    pub code: String,
    #[serde(default)]
    pub orderbook_id: Option<String>,
}

/// Server error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    EngineUnavailable,
    InvalidJson,
    InvalidMethod,
    RateLimited,
    Unknown,
}

impl From<&str> for ErrorCode {
    fn from(s: &str) -> Self {
        match s {
            "ENGINE_UNAVAILABLE" => Self::EngineUnavailable,
            "INVALID_JSON" => Self::InvalidJson,
            "INVALID_METHOD" => Self::InvalidMethod,
            "RATE_LIMITED" => Self::RateLimited,
            _ => Self::Unknown,
        }
    }
}

// ============================================================================
// PONG TYPE
// ============================================================================

/// Pong response data (empty)
#[derive(Debug, Clone, Deserialize)]
pub struct PongData {}

// ============================================================================
// CLIENT EVENTS
// ============================================================================

/// Events emitted by the WebSocket client
#[derive(Debug, Clone)]
pub enum WsEvent {
    /// Successfully connected to server
    Connected,

    /// Disconnected from server
    Disconnected { reason: String },

    /// Orderbook update received
    BookUpdate {
        orderbook_id: String,
        is_snapshot: bool,
    },

    /// Trade executed
    Trade {
        orderbook_id: String,
        trade: TradeData,
    },

    /// User event received
    UserUpdate {
        event_type: String,
        user: String,
    },

    /// Price history update
    PriceUpdate {
        orderbook_id: String,
        resolution: String,
    },

    /// Market event
    MarketEvent {
        event_type: String,
        market_pubkey: String,
    },

    /// Error occurred
    Error {
        error: super::error::WebSocketError,
    },

    /// Resync required for an orderbook
    ResyncRequired { orderbook_id: String },

    /// Pong received
    Pong,

    /// Reconnecting
    Reconnecting { attempt: u32 },
}

// ============================================================================
// MESSAGE TYPE ENUM
// ============================================================================

/// Enum for all possible server message types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    BookUpdate,
    Trades,
    User,
    PriceHistory,
    Market,
    Error,
    Pong,
    Unknown,
}

impl From<&str> for MessageType {
    fn from(s: &str) -> Self {
        match s {
            "book_update" => Self::BookUpdate,
            "trades" => Self::Trades,
            "user" => Self::User,
            "price_history" => Self::PriceHistory,
            "market" => Self::Market,
            "error" => Self::Error,
            "pong" => Self::Pong,
            _ => Self::Unknown,
        }
    }
}

// ============================================================================
// SIDE HELPERS
// ============================================================================

/// Order side enum for user events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

impl From<i32> for Side {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Buy,
            _ => Self::Sell,
        }
    }
}

impl Side {
    pub fn as_i32(&self) -> i32 {
        match self {
            Self::Buy => 0,
            Self::Sell => 1,
        }
    }
}

/// Price level side (from orderbook updates)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceLevelSide {
    Bid,
    Ask,
}

impl From<&str> for PriceLevelSide {
    fn from(s: &str) -> Self {
        match s {
            "bid" => Self::Bid,
            _ => Self::Ask,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_conversion() {
        assert_eq!(Side::from(0), Side::Buy);
        assert_eq!(Side::from(1), Side::Sell);
        assert_eq!(Side::Buy.as_i32(), 0);
        assert_eq!(Side::Sell.as_i32(), 1);
    }

    #[test]
    fn test_message_type_parsing() {
        assert_eq!(MessageType::from("book_update"), MessageType::BookUpdate);
        assert_eq!(MessageType::from("trades"), MessageType::Trades);
        assert_eq!(MessageType::from("user"), MessageType::User);
        assert_eq!(MessageType::from("unknown"), MessageType::Unknown);
    }

    #[test]
    fn test_subscribe_params_serialization() {
        let params = SubscribeParams::book_update(vec!["market1:ob1".to_string()]);
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("book_update"));
        assert!(json.contains("market1:ob1"));
    }

    #[test]
    fn test_ws_request_serialization() {
        let request = WsRequest::ping();
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(json, r#"{"method":"ping"}"#);
    }

    #[test]
    fn test_book_update_deserialization() {
        let json = r#"{
            "orderbook_id": "ob1",
            "timestamp": "2024-01-01T00:00:00.000Z",
            "sequence": 42,
            "bids": [{"side": "bid", "price": "0.500000", "size": "0.001000"}],
            "asks": [{"side": "ask", "price": "0.510000", "size": "0.000500"}],
            "is_snapshot": true
        }"#;
        let data: BookUpdateData = serde_json::from_str(json).unwrap();
        assert_eq!(data.orderbook_id, "ob1");
        assert_eq!(data.sequence, 42);
        assert!(data.is_snapshot);
        assert_eq!(data.bids.len(), 1);
        assert_eq!(data.bids[0].price, "0.500000");
        assert_eq!(data.bids[0].size, "0.001000");
        assert_eq!(data.asks.len(), 1);
        assert_eq!(data.asks[0].price, "0.510000");
        assert_eq!(data.asks[0].size, "0.000500");
    }

    #[test]
    fn test_trade_deserialization() {
        let json = r#"{
            "orderbook_id": "ob1",
            "price": "0.505000",
            "size": "0.000250",
            "side": "bid",
            "timestamp": "2024-01-01T00:00:00.000Z",
            "trade_id": "trade123",
            "sequence": 1
        }"#;
        let data: TradeData = serde_json::from_str(json).unwrap();
        assert_eq!(data.orderbook_id, "ob1");
        assert_eq!(data.price, "0.505000");
        assert_eq!(data.size, "0.000250");
        assert_eq!(data.sequence, 1);
    }
}
