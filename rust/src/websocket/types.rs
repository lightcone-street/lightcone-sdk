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
    /// Subscribe to user events (requires authentication)
    User {
        #[serde(rename = "type")]
        type_: &'static str,
        wallet_address: String,
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
    /// Subscribe to best bid/ask ticker updates
    Ticker {
        #[serde(rename = "type")]
        type_: &'static str,
        orderbook_ids: Vec<String>,
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
    pub fn user(wallet_address: String) -> Self {
        Self::User {
            type_: "user",
            wallet_address,
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

    /// Create ticker subscription params
    pub fn ticker(orderbook_ids: Vec<String>) -> Self {
        Self::Ticker {
            type_: "ticker",
            orderbook_ids,
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
            Self::Ticker { .. } => "ticker",
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
    pub seq: u64,
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

/// Discriminated user event -- deserialized by event_type from raw JSON.
///
/// The backend sends 4 different event shapes on the "user" channel:
/// - `snapshot`: Full state of orders, balances, and nonce
/// - `order`: Real-time order placement, update, or cancellation
/// - `balance_update`: Real-time balance change
/// - `nonce`: Nonce update
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event_type")]
pub enum UserEventData {
    #[serde(rename = "snapshot")]
    Snapshot(UserSnapshotData),
    #[serde(rename = "order")]
    Order(UserOrderEvent),
    #[serde(rename = "balance_update")]
    BalanceUpdate(UserBalanceEvent),
    #[serde(rename = "nonce")]
    Nonce(UserNonceEvent),
}

/// User snapshot: full state of orders, balances, and nonce
#[derive(Debug, Clone, Deserialize)]
pub struct UserSnapshotData {
    pub orders: Vec<UserOrderSnapshot>,
    pub balances: HashMap<String, BalanceEntry>,
    #[serde(default)]
    pub nonce: u64,
}

/// Real-time order event (placement, update, cancellation)
#[derive(Debug, Clone, Deserialize)]
pub struct UserOrderEvent {
    /// "PLACEMENT", "UPDATE", or "CANCELLATION"
    #[serde(rename = "type")]
    pub update_type: String,
    pub market_pubkey: String,
    pub orderbook_id: String,
    pub timestamp: String,
    pub order: UserFillInfo,
}

/// Real-time balance change event
#[derive(Debug, Clone, Deserialize)]
pub struct UserBalanceEvent {
    pub market_pubkey: String,
    pub orderbook_id: String,
    pub balance: Balance,
    pub timestamp: String,
}

/// Nonce update event
#[derive(Debug, Clone, Deserialize)]
pub struct UserNonceEvent {
    pub user_pubkey: String,
    pub new_nonce: u64,
    pub timestamp: String,
}

/// User order from snapshot
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserOrderSnapshot {
    pub order_hash: String,
    pub market_pubkey: String,
    pub orderbook_id: String,
    /// "bid" or "ask"
    pub side: String,
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
    /// Base token mint address
    pub base_mint: String,
    /// Quote token mint address
    pub quote_mint: String,
    /// Outcome index for base token (-1 if not found)
    pub outcome_index: i32,
    /// Order status: "OPEN", "MATCHING", "FILLED", "CANCELLED"
    pub status: String,
}

/// Per-user fill/order information from real-time events
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserFillInfo {
    pub order_hash: String,
    /// Price as decimal string
    pub price: String,
    /// Amount filled in this event as decimal string
    pub fill_amount: String,
    /// Remaining amount as decimal string
    pub remaining: String,
    /// Total filled so far as decimal string
    pub filled: String,
    /// "bid" or "ask"
    pub side: String,
    pub is_maker: bool,
    pub created_at: i64,
    #[serde(default)]
    pub balance: Option<Balance>,
    /// Base token mint address
    pub base_mint: String,
    /// Quote token mint address
    pub quote_mint: String,
    /// Outcome index for base token (-1 if not found)
    pub outcome_index: i32,
    /// Order status: "OPEN", "MATCHING", "FILLED", "CANCELLED"
    #[serde(default)]
    pub status: String,
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

/// Balance entry from user snapshot (keyed by orderbook_id)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceEntry {
    pub market_pubkey: String,
    pub orderbook_id: String,
    pub outcomes: Vec<OutcomeBalance>,
}

// ============================================================================
// TICKER TYPES
// ============================================================================

/// Ticker (best bid/ask) data from server
#[derive(Debug, Clone, Deserialize)]
pub struct TickerData {
    pub orderbook_id: String,
    #[serde(default)]
    pub best_bid: Option<String>,
    #[serde(default)]
    pub best_ask: Option<String>,
    #[serde(default)]
    pub mid: Option<String>,
    pub timestamp: String,
}

// ============================================================================
// AUTH TYPES
// ============================================================================

/// Auth status data from server (sent on connection)
#[derive(Debug, Clone, Deserialize)]
pub struct AuthData {
    pub status: String,
    #[serde(default)]
    pub wallet: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
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
    ParseError,
    AuthRequired,
    WalletMismatch,
    ConfigError,
    VerificationError,
    DecimalsNotConfigured,
    Unknown,
}

impl From<&str> for ErrorCode {
    fn from(s: &str) -> Self {
        match s {
            "ENGINE_UNAVAILABLE" => Self::EngineUnavailable,
            "INVALID_JSON" => Self::InvalidJson,
            "INVALID_METHOD" => Self::InvalidMethod,
            "RATE_LIMITED" => Self::RateLimited,
            "PARSE_ERROR" => Self::ParseError,
            "AUTH_REQUIRED" => Self::AuthRequired,
            "WALLET_MISMATCH" => Self::WalletMismatch,
            "CONFIG_ERROR" => Self::ConfigError,
            "VERIFICATION_ERROR" => Self::VerificationError,
            "DECIMALS_NOT_CONFIGURED" => Self::DecimalsNotConfigured,
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

    /// Auth status received from server on connection
    Auth {
        status: String,
        wallet: Option<String>,
        message: Option<String>,
    },

    /// Ticker (best bid/ask) update
    Ticker {
        orderbook_id: String,
        best_bid: Option<String>,
        best_ask: Option<String>,
        mid: Option<String>,
    },

    /// Nonce update for a user
    NonceUpdate {
        user: String,
        new_nonce: u64,
    },
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
    Auth,
    Ticker,
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
            "auth" => Self::Auth,
            "ticker" => Self::Ticker,
            _ => Self::Unknown,
        }
    }
}

// ============================================================================
// SIDE HELPERS
// ============================================================================

/// Order side enum (matches backend's "bid"/"ask" serialization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Bid,
    Ask,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bid => "bid",
            Self::Ask => "ask",
        }
    }
}

impl From<&str> for Side {
    fn from(s: &str) -> Self {
        match s {
            "bid" => Self::Bid,
            _ => Self::Ask,
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
        assert_eq!(Side::from("bid"), Side::Bid);
        assert_eq!(Side::from("ask"), Side::Ask);
        assert_eq!(Side::Bid.as_str(), "bid");
        assert_eq!(Side::Ask.as_str(), "ask");
    }

    #[test]
    fn test_side_serde() {
        let bid: Side = serde_json::from_str(r#""bid""#).unwrap();
        assert_eq!(bid, Side::Bid);
        let ask: Side = serde_json::from_str(r#""ask""#).unwrap();
        assert_eq!(ask, Side::Ask);
        assert_eq!(serde_json::to_string(&Side::Bid).unwrap(), r#""bid""#);
        assert_eq!(serde_json::to_string(&Side::Ask).unwrap(), r#""ask""#);
    }

    #[test]
    fn test_message_type_parsing() {
        assert_eq!(MessageType::from("book_update"), MessageType::BookUpdate);
        assert_eq!(MessageType::from("trades"), MessageType::Trades);
        assert_eq!(MessageType::from("user"), MessageType::User);
        assert_eq!(MessageType::from("auth"), MessageType::Auth);
        assert_eq!(MessageType::from("ticker"), MessageType::Ticker);
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
    fn test_subscribe_user_has_wallet_address() {
        let params = SubscribeParams::user("wallet123".to_string());
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("wallet_address"));
        assert!(json.contains("wallet123"));
        assert!(!json.contains(r#""user""#)); // should not have a "user" field
    }

    #[test]
    fn test_subscribe_ticker() {
        let params = SubscribeParams::ticker(vec!["ob1".to_string(), "ob2".to_string()]);
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("ticker"));
        assert!(json.contains("ob1"));
        assert!(json.contains("ob2"));
        assert_eq!(params.subscription_type(), "ticker");
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
            "seq": 42,
            "bids": [{"side": "bid", "price": "0.500000", "size": "0.001000"}],
            "asks": [{"side": "ask", "price": "0.510000", "size": "0.000500"}],
            "is_snapshot": true
        }"#;
        let data: BookUpdateData = serde_json::from_str(json).unwrap();
        assert_eq!(data.orderbook_id, "ob1");
        assert_eq!(data.seq, 42);
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

    #[test]
    fn test_user_snapshot_deserialization() {
        let json = r#"{
            "event_type": "snapshot",
            "orders": [{
                "order_hash": "hash1",
                "market_pubkey": "market1",
                "orderbook_id": "ob1",
                "side": "bid",
                "maker_amount": "1.000000",
                "taker_amount": "0.500000",
                "remaining": "0.800000",
                "filled": "0.200000",
                "price": "0.500000",
                "created_at": 1704067200000,
                "expiration": 0,
                "base_mint": "base_mint1",
                "quote_mint": "quote_mint1",
                "outcome_index": 0,
                "status": "OPEN"
            }],
            "balances": {
                "ob1": {
                    "market_pubkey": "market1",
                    "orderbook_id": "ob1",
                    "outcomes": [{"outcome_index": 0, "mint": "mint1", "idle": "5.0", "on_book": "1.0"}]
                }
            },
            "nonce": 42
        }"#;
        let data: UserEventData = serde_json::from_str(json).unwrap();
        match data {
            UserEventData::Snapshot(s) => {
                assert_eq!(s.orders.len(), 1);
                assert_eq!(s.orders[0].side, "bid");
                assert_eq!(s.orders[0].base_mint, "base_mint1");
                assert_eq!(s.orders[0].status, "OPEN");
                assert_eq!(s.nonce, 42);
                assert!(s.balances.contains_key("ob1"));
            }
            _ => panic!("Expected Snapshot variant"),
        }
    }

    #[test]
    fn test_user_order_event_deserialization() {
        let json = r#"{
            "event_type": "order",
            "type": "UPDATE",
            "market_pubkey": "market1",
            "orderbook_id": "ob1",
            "timestamp": "2024-01-01T00:00:00.000Z",
            "order": {
                "order_hash": "hash1",
                "price": "0.500000",
                "fill_amount": "0.100000",
                "remaining": "0.700000",
                "filled": "0.300000",
                "side": "bid",
                "is_maker": true,
                "created_at": 1704067200000,
                "base_mint": "base1",
                "quote_mint": "quote1",
                "outcome_index": 0,
                "status": "OPEN"
            }
        }"#;
        let data: UserEventData = serde_json::from_str(json).unwrap();
        match data {
            UserEventData::Order(e) => {
                assert_eq!(e.update_type, "UPDATE");
                assert_eq!(e.order.fill_amount, "0.100000");
                assert_eq!(e.order.base_mint, "base1");
            }
            _ => panic!("Expected Order variant"),
        }
    }

    #[test]
    fn test_user_balance_event_deserialization() {
        let json = r#"{
            "event_type": "balance_update",
            "market_pubkey": "market1",
            "orderbook_id": "ob1",
            "balance": {
                "outcomes": [{"outcome_index": 0, "mint": "mint1", "idle": "10.0", "on_book": "2.0"}]
            },
            "timestamp": "2024-01-01T00:00:00.000Z"
        }"#;
        let data: UserEventData = serde_json::from_str(json).unwrap();
        match data {
            UserEventData::BalanceUpdate(e) => {
                assert_eq!(e.orderbook_id, "ob1");
                assert_eq!(e.balance.outcomes[0].idle, "10.0");
            }
            _ => panic!("Expected BalanceUpdate variant"),
        }
    }

    #[test]
    fn test_user_nonce_event_deserialization() {
        let json = r#"{
            "event_type": "nonce",
            "user_pubkey": "user1",
            "new_nonce": 99,
            "timestamp": "2024-01-01T00:00:00.000Z"
        }"#;
        let data: UserEventData = serde_json::from_str(json).unwrap();
        match data {
            UserEventData::Nonce(e) => {
                assert_eq!(e.user_pubkey, "user1");
                assert_eq!(e.new_nonce, 99);
            }
            _ => panic!("Expected Nonce variant"),
        }
    }

    #[test]
    fn test_ticker_data_deserialization() {
        let json = r#"{
            "orderbook_id": "ob1",
            "best_bid": "0.500000",
            "best_ask": "0.510000",
            "mid": "0.505000",
            "timestamp": "2024-01-01T00:00:00.000Z"
        }"#;
        let data: TickerData = serde_json::from_str(json).unwrap();
        assert_eq!(data.orderbook_id, "ob1");
        assert_eq!(data.best_bid.unwrap(), "0.500000");
        assert_eq!(data.best_ask.unwrap(), "0.510000");
        assert_eq!(data.mid.unwrap(), "0.505000");
    }

    #[test]
    fn test_auth_data_deserialization() {
        let json = r#"{"status": "authenticated", "wallet": "abc123"}"#;
        let data: AuthData = serde_json::from_str(json).unwrap();
        assert_eq!(data.status, "authenticated");
        assert_eq!(data.wallet.unwrap(), "abc123");

        let json = r#"{"status": "anonymous", "message": "No auth provided"}"#;
        let data: AuthData = serde_json::from_str(json).unwrap();
        assert_eq!(data.status, "anonymous");
        assert_eq!(data.message.unwrap(), "No auth provided");
    }

    #[test]
    fn test_error_code_parsing() {
        assert_eq!(ErrorCode::from("PARSE_ERROR"), ErrorCode::ParseError);
        assert_eq!(ErrorCode::from("AUTH_REQUIRED"), ErrorCode::AuthRequired);
        assert_eq!(ErrorCode::from("WALLET_MISMATCH"), ErrorCode::WalletMismatch);
        assert_eq!(ErrorCode::from("DECIMALS_NOT_CONFIGURED"), ErrorCode::DecimalsNotConfigured);
        assert_eq!(ErrorCode::from("UNKNOWN_CODE"), ErrorCode::Unknown);
    }
}
