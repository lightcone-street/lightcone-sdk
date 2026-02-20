//! Wire types for trade responses (REST + WS).

use crate::shared::{OrderBookId, Side};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
// ─── REST wire types ─────────────────────────────────────────────────────────

/// A single trade from the REST API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeResponse {
    pub id: i64,
    pub orderbook_id: OrderBookId,
    pub taker_pubkey: String,
    pub maker_pubkey: String,
    pub side: Side,
    pub size: String,
    pub price: String,
    #[serde(default)]
    pub taker_fee: Option<String>,
    #[serde(default)]
    pub maker_fee: Option<String>,
    pub executed_at: i64,
}

/// REST decimals metadata for trades.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradesDecimals {
    #[serde(default)]
    pub price: Option<u8>,
    #[serde(default)]
    pub size: Option<u8>,
    #[serde(default)]
    pub fee: Option<u8>,
}

/// REST response for trades list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesResponse {
    pub orderbook_id: OrderBookId,
    pub trades: Vec<TradeResponse>,
    #[serde(default)]
    pub next_cursor: Option<i64>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub decimals: Option<TradesDecimals>,
}

// ─── WS wire types ───────────────────────────────────────────────────────────

/// WS trade event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WsTrade {
    pub orderbook_id: OrderBookId,
    pub trade_id: String,
    pub timestamp: DateTime<Utc>,
    pub price: Decimal,
    pub size: Decimal,
    pub side: Side,
}
