//! Price history domain — chart data, candle resolution.

pub mod client;
pub mod state;
pub mod wire;

use serde::{Deserialize, Serialize};

pub use state::PriceHistoryState;

/// A single data point on a price chart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineData {
    /// UTC timestamp in seconds.
    pub time: u64,
    pub value: String,
}

impl From<wire::WsLineData> for LineData {
    fn from(w: wire::WsLineData) -> Self {
        Self {
            time: w.t,
            value: w.value,
        }
    }
}
