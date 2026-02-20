//! Wire types for price history (WS).

use crate::shared::{OrderBookId, Resolution};
use serde::{Deserialize, Serialize};

/// A single candle from the backend.
///
/// When `include_ohlcv` is false (the default), candles with no trades
/// will only have `t` and `m`. All OHLCV fields are optional.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceCandle {
    /// Unix timestamp in milliseconds (candle start).
    pub t: i64,
    /// Midpoint: (best_bid + best_ask) / 2.
    #[serde(default)]
    pub m: Option<String>,
    /// Open price.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub o: Option<String>,
    /// High price.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h: Option<String>,
    /// Low price.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l: Option<String>,
    /// Close price.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c: Option<String>,
    /// Volume.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v: Option<String>,
}

/// WS price history snapshot.
#[derive(Deserialize, Debug, Clone)]
pub struct PriceHistorySnapshot {
    pub orderbook_id: OrderBookId,
    pub resolution: Resolution,
    pub prices: Vec<PriceCandle>,
    #[serde(default)]
    pub last_timestamp: Option<i64>,
    #[serde(default)]
    pub server_time: Option<u64>,
}

/// WS price history update (single candle).
///
/// Includes all candle fields plus `bb` (best bid) and `ba` (best ask).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PriceHistoryUpdate {
    pub orderbook_id: OrderBookId,
    pub resolution: Resolution,
    /// Unix timestamp in milliseconds.
    pub t: i64,
    /// Midpoint.
    #[serde(default)]
    pub m: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub o: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub c: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v: Option<String>,
    /// Best bid at this candle's time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bb: Option<String>,
    /// Best ask at this candle's time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ba: Option<String>,
}

/// WS price history heartbeat.
#[derive(Deserialize, Debug, Clone)]
pub struct PriceHistoryHeartbeat {
    pub server_time: u64,
    #[serde(default)]
    pub last_processed: Option<u64>,
}

/// WS price history tagged enum.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "event_type")]
pub enum PriceHistory {
    #[serde(rename = "snapshot")]
    Snapshot(PriceHistorySnapshot),
    #[serde(rename = "update")]
    Update(PriceHistoryUpdate),
    #[serde(rename = "heartbeat")]
    Heartbeat(PriceHistoryHeartbeat),
}
