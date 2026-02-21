//! Ticker data — best bid/ask/mid for an orderbook.

use crate::shared::OrderBookId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Best bid/ask/mid for a ticker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TickerData {
    pub orderbook_id: OrderBookId,
    pub best_bid: Option<Decimal>,
    pub best_ask: Option<Decimal>,
    pub mid_price: Option<Decimal>,
}
