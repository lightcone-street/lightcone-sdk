//! Wire types for price history (WS and REST).

use crate::shared::{OrderBookId, PubkeyStr, Resolution};
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

/// A single deposit-token websocket candle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositTokenCandle {
    /// Unix timestamp in milliseconds (candle open time).
    pub t: i64,
    /// Unix timestamp in milliseconds (candle close time).
    pub tc: i64,
    /// Close price as a raw Binance decimal string.
    pub c: String,
}

/// Websocket snapshot of deposit-token candles for one asset + resolution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositPriceSnapshot {
    pub deposit_asset: PubkeyStr,
    pub resolution: Resolution,
    pub prices: Vec<DepositTokenCandle>,
}

/// Websocket candle update for one deposit asset.
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

/// Websocket live spot-price tick for a deposit asset.
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

/// Orderbook price candle from the REST API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderbookPriceCandle {
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
    /// Best bid at this candle's time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bb: Option<String>,
    /// Best ask at this candle's time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ba: Option<String>,
}

/// Orderbook price-history decimals metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceHistoryDecimals {
    pub price: u8,
    pub volume: u8,
}

/// REST response for `/api/price-history?orderbook_id=...`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderbookPriceHistoryResponse {
    pub orderbook_id: OrderBookId,
    pub resolution: Resolution,
    pub include_ohlcv: bool,
    pub prices: Vec<OrderbookPriceCandle>,
    pub next_cursor: Option<u64>,
    pub has_more: bool,
    pub decimals: PriceHistoryDecimals,
}

/// Deposit-token price candle from the REST API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositPriceCandle {
    /// Unix timestamp in milliseconds (candle open time).
    pub t: i64,
    /// Unix timestamp in milliseconds (candle close time).
    pub tc: i64,
    /// Close price as a raw Binance decimal string.
    pub c: String,
}

/// REST response for `/api/price-history?deposit_asset=...`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositPriceHistoryResponse {
    pub deposit_asset: PubkeyStr,
    pub binance_symbol: String,
    pub resolution: Resolution,
    pub prices: Vec<DepositPriceCandle>,
    pub next_cursor: Option<u64>,
    pub has_more: bool,
}
