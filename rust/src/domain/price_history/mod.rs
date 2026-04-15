#![doc = include_str!("README.md")]

pub mod client;
pub mod state;
pub mod wire;

use crate::shared::{PubkeyStr, Resolution};
use serde::{Deserialize, Serialize};

pub use state::{DepositPriceState, LatestDepositPrice, PriceHistoryState};
pub use wire::{
    DepositPrice, DepositPriceCandle, DepositPriceCandleUpdate, DepositPriceHistoryResponse,
    DepositPriceSnapshot, DepositPriceTick, DepositTokenCandle, OrderbookPriceCandle,
    OrderbookPriceHistoryResponse, PriceHistoryDecimals,
};

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

impl From<wire::OrderbookPriceCandle> for LineData {
    fn from(c: wire::OrderbookPriceCandle) -> Self {
        Self {
            time: c.t,
            value: c.m.or(c.c).unwrap_or_default(),
        }
    }
}

impl From<wire::DepositTokenCandle> for LineData {
    fn from(c: wire::DepositTokenCandle) -> Self {
        Self {
            time: c.t,
            value: c.c,
        }
    }
}

/// Key for deposit-price lookups.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DepositPriceKey {
    pub deposit_asset: PubkeyStr,
    pub resolution: Resolution,
}

/// Query options for orderbook price history REST requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderbookPriceHistoryQuery {
    pub resolution: Resolution,
    pub from: Option<u64>,
    pub to: Option<u64>,
    pub cursor: Option<u64>,
    pub limit: Option<usize>,
    pub include_ohlcv: bool,
}

impl Default for OrderbookPriceHistoryQuery {
    fn default() -> Self {
        Self {
            resolution: Resolution::Minute1,
            from: None,
            to: None,
            cursor: None,
            limit: None,
            include_ohlcv: false,
        }
    }
}

impl OrderbookPriceHistoryQuery {
    pub fn new(resolution: Resolution) -> Self {
        Self {
            resolution,
            ..Self::default()
        }
    }
}

/// Query options for deposit-token price history REST requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositPriceHistoryQuery {
    pub resolution: Resolution,
    pub from: Option<u64>,
    pub to: Option<u64>,
    pub cursor: Option<u64>,
    pub limit: Option<usize>,
}

impl Default for DepositPriceHistoryQuery {
    fn default() -> Self {
        Self {
            resolution: Resolution::Minute1,
            from: None,
            to: None,
            cursor: None,
            limit: None,
        }
    }
}

impl DepositPriceHistoryQuery {
    pub fn new(resolution: Resolution) -> Self {
        Self {
            resolution,
            ..Self::default()
        }
    }
}
