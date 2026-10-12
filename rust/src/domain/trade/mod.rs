#![doc = include_str!("README.md")]

pub mod client;
mod convert;
pub mod state;
pub mod wire;

use crate::shared::{OrderBookId, Side};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use state::TradeHistory;

/// A trade execution record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Trade {
    pub orderbook_id: OrderBookId,
    /// `"<execution_id>:<leg_index>:<projection_generation>"`, identical for
    /// REST and WS trades, so it deduplicates merged history.
    pub trade_id: String,
    /// Numeric REST row id used for cursor pagination. Absent on WS trades.
    #[serde(default)]
    pub cursor_id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    /// Price in quote units per base unit. REST reports the maker price
    /// truncated to the book's price decimals; WS trades derive the exact
    /// `quote_amount / base_amount`.
    pub price: Decimal,
    /// Size in base-token units.
    pub size: Decimal,
    /// Taker side.
    pub side: Side,
}

/// A page of trades with cursor-based pagination metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradesPage {
    pub trades: Vec<Trade>,
    pub next_cursor: Option<i64>,
    pub has_more: bool,
}
