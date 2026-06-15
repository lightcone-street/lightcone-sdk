//! Wire types for orderbook responses (REST + WS).

use crate::domain::orderbook::aggregation::BookAggregation;
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
///
/// Depth is capped server-side at 20 levels per side.
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
    /// Display decimals for prices and sizes. Always sent by current backends;
    /// optional for tolerance of older payloads.
    #[serde(default)]
    pub decimals: Option<OrderbookDepthDecimals>,
}

/// Price/size display decimals for an orderbook, as returned by the depth
/// endpoint. Distinct from [`DecimalsResponse`] (the `/decimals` endpoint).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderbookDepthDecimals {
    pub price: u8,
    pub size: u8,
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

/// WS orderbook snapshot frame.
///
/// The stream is snapshot-only: every data frame carries the full top-20
/// levels per side and replaces the previous book wholesale (last-write-wins).
/// `seq` is strictly increasing but non-contiguous, and the initial snapshot
/// after every (re)subscribe is `seq: 0` — informational only, never a gate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderBook {
    #[serde(rename = "orderbook_id")]
    pub id: OrderBookId,
    #[serde(default)]
    pub is_snapshot: bool,
    #[serde(default)]
    pub seq: u64,
    #[serde(default)]
    pub resync: bool,
    #[serde(default = "Vec::new")]
    pub bids: Vec<WsBookLevel>,
    #[serde(default = "Vec::new")]
    pub asks: Vec<WsBookLevel>,
    /// Aggregation tags echoed by the backend (omitted = full precision).
    /// Always normalized server-side ((5, none) arrives as (5, 1)).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_sig_figs: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mantissa: Option<u32>,
}

impl OrderBook {
    /// The aggregation view this frame belongs to (untagged = full precision).
    /// Use it to key per-`(orderbook_id, aggregation)` book state when one
    /// connection holds multiple aggregation views of the same orderbook.
    pub fn aggregation(&self) -> BookAggregation {
        BookAggregation::from_frame(self.n_sig_figs, self.mantissa)
    }
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
