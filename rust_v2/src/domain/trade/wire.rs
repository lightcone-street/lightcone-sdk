//! Wire types for trade responses (REST + WS).

use crate::shared::{OrderBookId, Side};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// REST response for a single trade.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeResponse {
    pub orderbook_id: OrderBookId,
    pub trade_id: String,
    pub timestamp: DateTime<Utc>,
    pub price: Decimal,
    pub size: Decimal,
    pub side: Side,
}

/// REST response for trades list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesResponse {
    pub trades: Vec<TradeResponse>,
    pub total: usize,
}

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

