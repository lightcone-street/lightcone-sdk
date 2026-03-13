//! Wire types for the `deposit_price` websocket channel.

use crate::shared::{PubkeyStr, Resolution};
use serde::{Deserialize, Serialize};

/// A single deposit-token candle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositTokenCandle {
    /// Unix timestamp in milliseconds (candle open time).
    pub t: i64,
    /// Unix timestamp in milliseconds (candle close time).
    pub tc: i64,
    /// Close price as a raw Binance decimal string.
    pub c: String,
}

/// Snapshot of deposit-token candles for one asset + resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositPriceSnapshot {
    pub deposit_asset: PubkeyStr,
    pub resolution: Resolution,
    pub prices: Vec<DepositTokenCandle>,
}

/// Incremental candle update for one deposit asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositPriceCandleUpdate {
    pub deposit_asset: PubkeyStr,
    pub resolution: Resolution,
    /// Unix timestamp in milliseconds (candle open time).
    pub t: i64,
    /// Unix timestamp in milliseconds (candle close time).
    pub tc: i64,
    /// Close price as a raw Binance decimal string.
    pub c: String,
}

/// Ongoing spot-price tick for a deposit asset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositPriceTick {
    pub deposit_asset: PubkeyStr,
    pub price: String,
    pub event_time: i64,
}

/// Tagged websocket payload for the `deposit_price` channel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum DepositPrice {
    #[serde(rename = "snapshot")]
    Snapshot(DepositPriceSnapshot),
    #[serde(rename = "candle")]
    Candle(DepositPriceCandleUpdate),
    #[serde(rename = "price")]
    Price(DepositPriceTick),
}
