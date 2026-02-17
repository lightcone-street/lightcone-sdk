//! Trade domain — trade execution records and history.

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
    pub trade_id: String,
    pub timestamp: DateTime<Utc>,
    pub price: Decimal,
    pub size: Decimal,
    pub side: Side,
}
