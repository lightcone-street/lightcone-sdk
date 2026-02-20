//! Price history domain — chart data, candle resolution.

pub mod client;
pub mod state;
pub mod wire;

use serde::{Deserialize, Serialize};

pub use state::PriceHistoryState;

/// A single data point on a price chart (simplified from the full candle).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineData {
    /// Unix timestamp in milliseconds.
    pub time: i64,
    /// Midpoint value as a decimal string.
    pub value: String,
}

impl From<wire::PriceCandle> for LineData {
    fn from(c: wire::PriceCandle) -> Self {
        Self {
            time: c.t,
            value: c.m.unwrap_or_default(),
        }
    }
}
