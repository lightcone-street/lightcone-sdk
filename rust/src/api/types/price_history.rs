//! Price history types for the Lightcone REST API.

use crate::shared::Resolution;
use serde::{Deserialize, Serialize};

/// Price point data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricePoint {
    /// Timestamp (milliseconds)
    #[serde(rename = "t")]
    pub timestamp: i64,
    /// Midpoint price as decimal string (null when unavailable)
    #[serde(rename = "m", default)]
    pub midpoint: Option<String>,
    /// Open price (only with include_ohlcv) as decimal string
    #[serde(rename = "o", default, skip_serializing_if = "Option::is_none")]
    pub open: Option<String>,
    /// High price (only with include_ohlcv) as decimal string
    #[serde(rename = "h", default, skip_serializing_if = "Option::is_none")]
    pub high: Option<String>,
    /// Low price (only with include_ohlcv) as decimal string
    #[serde(rename = "l", default, skip_serializing_if = "Option::is_none")]
    pub low: Option<String>,
    /// Close price (only with include_ohlcv) as decimal string
    #[serde(rename = "c", default, skip_serializing_if = "Option::is_none")]
    pub close: Option<String>,
    /// Volume (only with include_ohlcv) as decimal string
    #[serde(rename = "v", default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    /// Best bid (only with include_ohlcv) as decimal string
    #[serde(rename = "bb", default, skip_serializing_if = "Option::is_none")]
    pub best_bid: Option<String>,
    /// Best ask (only with include_ohlcv) as decimal string
    #[serde(rename = "ba", default, skip_serializing_if = "Option::is_none")]
    pub best_ask: Option<String>,
}

/// Query parameters for GET /api/price-history.
#[derive(Debug, Clone, Default)]
pub struct PriceHistoryParams {
    /// Orderbook identifier (required)
    pub orderbook_id: String,
    /// Candle resolution
    pub resolution: Option<Resolution>,
    /// Start timestamp (milliseconds)
    pub from: Option<i64>,
    /// End timestamp (milliseconds)
    pub to: Option<i64>,
    /// Pagination cursor
    pub cursor: Option<i64>,
    /// Max results (1-1000)
    pub limit: Option<u32>,
    /// Include full OHLCV data
    pub include_ohlcv: Option<bool>,
}

impl PriceHistoryParams {
    /// Create new params with required orderbook_id.
    pub fn new(orderbook_id: impl Into<String>) -> Self {
        Self {
            orderbook_id: orderbook_id.into(),
            ..Default::default()
        }
    }

    /// Set resolution.
    pub fn with_resolution(mut self, resolution: Resolution) -> Self {
        self.resolution = Some(resolution);
        self
    }

    /// Set time range.
    pub fn with_time_range(mut self, from: i64, to: i64) -> Self {
        self.from = Some(from);
        self.to = Some(to);
        self
    }

    /// Set pagination cursor.
    pub fn with_cursor(mut self, cursor: i64) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Set result limit.
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Include OHLCV data.
    pub fn with_ohlcv(mut self) -> Self {
        self.include_ohlcv = Some(true);
        self
    }
}

/// Decimal precision info for price history data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryDecimals {
    /// Decimals for price fields (o, h, l, c, m, bb, ba)
    pub price: u8,
    /// Decimals for volume field (v)
    pub volume: u8,
}

/// Response for GET /api/price-history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistoryResponse {
    /// Orderbook ID
    pub orderbook_id: String,
    /// Resolution used
    pub resolution: String,
    /// Whether OHLCV data is included
    pub include_ohlcv: bool,
    /// Price points
    pub prices: Vec<PricePoint>,
    /// Next pagination cursor
    pub next_cursor: Option<i64>,
    /// Whether more results exist
    pub has_more: bool,
    /// Decimal precision for price and volume fields
    pub decimals: PriceHistoryDecimals,
}
