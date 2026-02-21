//! Wire types for orderbook responses (REST + WS).

use crate::shared::{OrderBookId, Side};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ─── REST wire types ─────────────────────────────────────────────────────────

/// REST response for a single orderbook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderbookResponse {
    pub id: i32,
    pub market_pubkey: String,
    pub orderbook_id: String,
    pub base_token: String,
    pub quote_token: String,
    pub outcome_index: Option<i16>,
    pub tick_size: i64,
    pub total_bids: i32,
    pub total_asks: i32,
    pub last_trade_price: Option<Decimal>,
    pub last_trade_time: Option<DateTime<Utc>>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// REST response for multiple orderbooks.
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderbooksResponse {
    pub orderbooks: Vec<OrderbookResponse>,
    pub total: usize,
}

/// REST response for orderbook depth.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderbookDepthResponse {
    pub orderbook_id: OrderBookId,
    #[serde(default)]
    pub market_pubkey: Option<String>,
    pub best_bid: Option<Decimal>,
    pub best_ask: Option<Decimal>,
    #[serde(default)]
    pub spread: Option<Decimal>,
    #[serde(default)]
    pub tick_size: Option<String>,
    pub bids: Vec<RestBookLevel>,
    pub asks: Vec<RestBookLevel>,
}

/// A single price level from the REST depth endpoint.
///
/// Side is implicit from the `bids`/`asks` array — not included in the response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RestBookLevel {
    pub price: Decimal,
    pub size: Decimal,
    #[serde(default)]
    pub orders: Option<u32>,
}

/// Decimals metadata for an orderbook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecimalsResponse {
    pub orderbook_id: String,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub price_decimals: u8,
}

// ─── WS wire types ───────────────────────────────────────────────────────────

/// WS orderbook snapshot or delta.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderBook {
    #[serde(rename = "orderbook_id")]
    pub id: OrderBookId,
    #[serde(default)]
    pub is_snapshot: bool,
    #[serde(default)]
    pub seq: u32,
    #[serde(default = "Vec::new")]
    pub bids: Vec<WsBookLevel>,
    #[serde(default = "Vec::new")]
    pub asks: Vec<WsBookLevel>,
}

/// A single price level from the WS book update.
///
/// `side` is explicitly provided by the backend in WS messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WsBookLevel {
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
}

/// WS ticker data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WsTickerData {
    pub orderbook_id: OrderBookId,
    pub best_bid: Option<Decimal>,
    pub best_ask: Option<Decimal>,
    #[serde(alias = "mid_price")]
    pub mid: Option<Decimal>,
}
