//! Wire types for price history (WS).

use crate::shared::{OrderBookId, Resolution};
use serde::{Deserialize, Serialize};

/// A single price data point from the backend.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WsLineData {
    pub t: u64,
    pub value: String,
}

/// WS price history snapshot.
#[derive(Deserialize, Debug, Clone)]
pub struct PriceHistorySnapshot {
    pub orderbook_id: OrderBookId,
    pub resolution: Resolution,
    pub prices: Vec<WsLineData>,
}

/// WS price history update (single candle).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PriceHistoryUpdate {
    pub orderbook_id: OrderBookId,
    pub resolution: Resolution,
    pub t: u64,
    pub value: String,
}

/// WS price history tagged enum.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "event_type")]
pub enum PriceHistory {
    #[serde(rename = "snapshot")]
    Snapshot(PriceHistorySnapshot),
    #[serde(rename = "update")]
    Update(PriceHistoryUpdate),
}
