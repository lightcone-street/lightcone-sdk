//! Wire types for orderbook responses (REST + WS).

use crate::domain::orderbook::aggregation::BookAggregation;
use crate::shared::scaling::{OrderbookRules, TradingRules};
use crate::shared::{OrderBookId, Side};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ─── REST wire types ─────────────────────────────────────────────────────────

/// REST response for a single orderbook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderbookResponse {
    pub id: i32,
    pub market_pubkey: String,
    pub orderbook_id: String,
    pub base_token: String,
    pub quote_token: String,
    pub outcome_index: Option<i16>,
    pub tick_size: i64,
    pub total_bids: i32,
    pub total_asks: i32,
    pub last_trade_price: Option<Decimal>,
    pub last_trade_time: Option<DateTime<Utc>>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// REST response for multiple orderbooks.
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderbooksResponse {
    pub orderbooks: Vec<OrderbookResponse>,
    pub total: usize,
}

/// REST response for orderbook depth.
///
/// Depth is capped server-side at 20 levels per side.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderbookDepthResponse {
    pub orderbook_id: OrderBookId,
    #[serde(default)]
    pub market_pubkey: Option<String>,
    pub best_bid: Option<Decimal>,
    pub best_ask: Option<Decimal>,
    #[serde(default)]
    pub spread: Option<Decimal>,
    /// Deprecated backend alias retained only for wire visibility. Never use
    /// this field for order admission.
    #[serde(default)]
    pub tick_size: Option<String>,
    pub price_quantum: String,
    pub trading_rules: TradingRules,
    #[serde(default)]
    pub bids_truncated: bool,
    #[serde(default)]
    pub asks_truncated: bool,
    pub revision: u64,
    pub captured_at_ms: u64,
    pub bids: Vec<RestBookLevel>,
    pub asks: Vec<RestBookLevel>,
    /// Required display decimals for prices and sizes. `size` is the base
    /// token's on-chain decimal count, not the admission size precision.
    pub decimals: OrderbookDepthDecimals,
}

/// Price/size display decimals for an orderbook, as returned by the depth
/// endpoint. Distinct from [`DecimalsResponse`] (the `/decimals` endpoint).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderbookDepthDecimals {
    pub price: u8,
    pub size: u8,
}

/// A single price level from the REST depth endpoint.
///
/// Side is implicit from the `bids`/`asks` array — not included in the response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RestBookLevel {
    pub price: Decimal,
    pub size: Decimal,
    #[serde(default)]
    pub orders: Option<u32>,
}

/// Exact decimals and immutable trading rules for an orderbook.
pub type DecimalsResponse = OrderbookRules;

// ─── WS wire types ───────────────────────────────────────────────────────────

/// WS orderbook snapshot frame.
///
/// The stream is snapshot-only: every data frame carries the full top-20
/// levels per side and replaces the previous book wholesale. `seq` is the
/// engine depth revision and is monotonic only within a subscription generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderBook {
    #[serde(rename = "orderbook_id")]
    pub id: OrderBookId,
    #[serde(default)]
    pub is_snapshot: bool,
    pub seq: u64,
    #[serde(default)]
    pub resync: bool,
    #[serde(default = "Vec::new")]
    pub bids: Vec<WsBookLevel>,
    #[serde(default = "Vec::new")]
    pub asks: Vec<WsBookLevel>,
    #[serde(default)]
    pub bids_truncated: bool,
    #[serde(default)]
    pub asks_truncated: bool,
    #[serde(default)]
    pub timestamp: Option<DateTime<Utc>>,
    /// Aggregation tags echoed by the backend (omitted = full precision).
    /// Always normalized server-side ((5, none) arrives as (5, 1)).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_sig_figs: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mantissa: Option<u32>,
}

impl OrderBook {
    /// The aggregation view this frame belongs to (untagged = full precision).
    /// Use it to key per-`(orderbook_id, aggregation)` book state when one
    /// connection holds multiple aggregation views of the same orderbook.
    pub fn aggregation(&self) -> BookAggregation {
        BookAggregation::from_frame(self.n_sig_figs, self.mantissa)
    }
}

/// A single price level from the WS book update.
///
/// `side` is explicitly provided by the backend in WS messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WsBookLevel {
    pub side: Side,
    pub price: Decimal,
    pub size: Decimal,
}

/// WS ticker data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WsTickerData {
    pub orderbook_id: OrderBookId,
    pub best_bid: Option<Decimal>,
    pub best_ask: Option<Decimal>,
    #[serde(alias = "mid_price")]
    pub mid: Option<Decimal>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULES: &str = r#"{
        "base_size_decimals":5,
        "max_price_decimals":1,
        "max_price_significant_figures":5,
        "integer_prices_always_allowed":true,
        "price_quantum":"0.1000",
        "price_quantum_raw":"1000",
        "base_size_quantum":"0.00001000",
        "base_size_quantum_raw":"1000"
    }"#;

    #[test]
    fn decimals_raw_quantums_parse_from_strings() {
        let json = format!(
            r#"{{
            "orderbook_id":"ob","base_decimals":8,"quote_decimals":6,
            "price_decimals":4,"trading_rules":{RULES}
        }}"#
        );
        let response: DecimalsResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.trading_rules.price_quantum_raw.to_string(), "1000");
        assert!(serde_json::from_str::<DecimalsResponse>(&json.replace(
            r#""price_quantum_raw":"1000""#,
            r#""price_quantum_raw":1000"#,
        ))
        .is_err());
    }

    #[test]
    fn depth_defaults_only_truncation_flags() {
        let json = format!(
            r#"{{
            "orderbook_id":"ob","best_bid":null,"best_ask":null,
            "bids":[],"asks":[],"price_quantum":"0.1000",
            "trading_rules":{RULES},"revision":1842,"captured_at_ms":1785776400123,
            "decimals":{{"price":4,"size":8}}
        }}"#
        );
        let depth: OrderbookDepthResponse = serde_json::from_str(&json).unwrap();
        assert!(!depth.bids_truncated);
        assert!(!depth.asks_truncated);
        assert_eq!(depth.revision, 1842);
        assert_eq!(depth.captured_at_ms, 1_785_776_400_123);
    }

    #[test]
    fn websocket_truncation_flags_default_and_parse() {
        let omitted: OrderBook =
            serde_json::from_str(r#"{"orderbook_id":"ob","seq":1842,"bids":[],"asks":[]}"#)
                .unwrap();
        assert!(!omitted.bids_truncated && !omitted.asks_truncated);
        let present: OrderBook = serde_json::from_str(
            r#"{"orderbook_id":"ob","seq":1843,"bids":[],"asks":[],"bids_truncated":true}"#,
        )
        .unwrap();
        assert!(present.bids_truncated);
    }
}
