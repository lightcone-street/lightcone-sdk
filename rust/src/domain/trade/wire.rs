//! Wire types for trade responses (REST + WS).

use crate::shared::price::quote_per_base;
use crate::shared::{serde_util, CommitInfo, OrderBookId, PubkeyStr, Side};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
// ─── REST wire types ─────────────────────────────────────────────────────────

/// A single trade from the REST API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradeResponse {
    /// Fill-fact row id; pass as `cursor` to page backwards.
    pub id: i64,
    /// `"<execution_id>:<leg_index>:<projection_generation>"`; see
    /// [`WsTrade::trade_id`] for the matching live id.
    pub trade_id: String,
    pub orderbook_id: OrderBookId,
    pub taker_pubkey: String,
    pub maker_pubkey: String,
    /// Taker side.
    pub side: Side,
    /// Size in base-token units.
    pub size: String,
    /// Maker price in quote units per base unit (truncated to price decimals).
    pub price: String,
    /// Signed taker fee estimate (negative = rebate), scaled with the quote
    /// token's decimals.
    #[serde(default)]
    pub taker_fee_estimate: Option<Decimal>,
    /// Signed maker fee estimate (negative = rebate), scaled with the quote
    /// token's decimals.
    #[serde(default)]
    pub maker_fee_estimate: Option<Decimal>,
    /// Unix milliseconds.
    pub executed_at: i64,
}

/// REST decimals metadata for trades.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TradesDecimals {
    #[serde(default)]
    pub price: Option<u8>,
    #[serde(default)]
    pub size: Option<u8>,
    #[serde(default)]
    pub fee: Option<u8>,
}

/// REST response for trades list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesResponse {
    pub orderbook_id: OrderBookId,
    pub trades: Vec<TradeResponse>,
    #[serde(default)]
    pub next_cursor: Option<i64>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub decimals: Option<TradesDecimals>,
}

/// REST response for market-level trades (all orderbooks in a market).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketTradesResponse {
    pub market_pubkey: PubkeyStr,
    pub trades: Vec<TradeResponse>,
    #[serde(default)]
    pub next_cursor: Option<i64>,
    #[serde(default)]
    pub has_more: bool,
}

// ─── WS wire types ───────────────────────────────────────────────────────────

/// Committed fill fact on the WS `trades` channel, also delivered to the maker
/// and taker as a `user` channel `fill` event.
///
/// The payload carries exact base and quote amounts rather than a price; use
/// [`Self::price`]. Amount fields are decimal strings in the named token's
/// units.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WsTrade {
    #[serde(flatten)]
    pub commit: CommitInfo,
    /// Relay dedup id: `"<evidence_id>:<event_index>:<leg_index>"`. Not the
    /// REST `trade_id`; see [`Self::trade_id`].
    pub fill_id: String,
    pub execution_id: String,
    pub evidence_id: String,
    /// Index of the settlement event within the confirming evidence.
    pub event_index: i32,
    /// Index of this leg within the execution.
    pub leg_index: i32,
    pub orderbook_id: OrderBookId,
    pub market_pubkey: PubkeyStr,
    pub base_mint: PubkeyStr,
    pub quote_mint: PubkeyStr,
    /// Filled size in base-token units.
    pub base_amount: Decimal,
    /// Filled notional in quote-token units.
    pub quote_amount: Decimal,
    pub maker_order_hash: String,
    pub taker_order_hash: String,
    pub maker_wallet: PubkeyStr,
    pub taker_wallet: PubkeyStr,
    /// Taker side; numeric (`0` bid, `1` ask) on the wire.
    #[serde(with = "serde_util::side_text_or_number")]
    pub taker_side: Side,
    /// Signed maker fee estimate in `fee_mint` units (negative = rebate).
    pub maker_fee_estimate: Decimal,
    /// Signed taker fee estimate in `fee_mint` units.
    pub taker_fee_estimate: Decimal,
    pub fee_mint: PubkeyStr,
    /// Maker fee rate in basis points (negative = rebate).
    pub maker_fee_bps: i16,
    /// Taker fee rate in basis points.
    pub taker_fee_bps: i16,
    pub executed_at: DateTime<Utc>,
}

impl WsTrade {
    /// Execution price in quote units per base unit (`quote_amount / base_amount`).
    pub fn price(&self) -> Option<Decimal> {
        quote_per_base(self.quote_amount, self.base_amount)
    }

    /// The REST `trade_id` of this fill
    /// (`"<execution_id>:<leg_index>:<projection_generation>"`), for merging
    /// live trades with REST history.
    pub fn trade_id(&self) -> String {
        format!(
            "{}:{}:{}",
            self.execution_id, self.leg_index, self.commit.projection_generation
        )
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Fill fact as `redis_listener::handle_committed` relays it on `trades`.
    pub(crate) fn fill_fact() -> serde_json::Value {
        serde_json::json!({
            "effect_id": "812:1",
            "committed_revision": 812,
            "projection_generation": 3,
            "actionable": true,
            "event_type": "fill",
            "fill_id": "0d6a6f3e-5b9b-4b43-9f0f-6c7c1f3f9a10:2:0",
            "execution_id": "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11",
            "evidence_id": "0d6a6f3e-5b9b-4b43-9f0f-6c7c1f3f9a10",
            "event_index": 2,
            "leg_index": 0,
            "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
            "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
            "base_mint": "BaseMint111111111111111111111111111111111111",
            "quote_mint": "QuoteMint11111111111111111111111111111111111",
            "base_amount": "4.00000000",
            "quote_amount": "2.200000",
            "maker_order_hash": "aa".repeat(32),
            "taker_order_hash": "bb".repeat(32),
            "maker_wallet": "Maker111111111111111111111111111111111111111",
            "taker_wallet": "Taker111111111111111111111111111111111111111",
            "taker_side": 1,
            "maker_fee_estimate": "-0.000110",
            "taker_fee_estimate": "0.006600",
            "fee_mint": "QuoteMint11111111111111111111111111111111111",
            "maker_fee_bps": -5,
            "taker_fee_bps": 30,
            "executed_at": "2026-09-29T12:00:00+00:00"
        })
    }

    #[test]
    fn fill_fact_decodes_and_derives_price_and_rest_id() {
        let trade: WsTrade = serde_json::from_value(fill_fact()).unwrap();
        assert_eq!(trade.commit.committed_revision, 812);
        assert_eq!(trade.taker_side, Side::Ask);
        assert_eq!(trade.base_amount, Decimal::from(4));
        assert_eq!(trade.price(), Some(Decimal::new(55, 2)));
        assert_eq!(trade.maker_fee_estimate, Decimal::new(-110, 6));
        assert_eq!(trade.trade_id(), "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3");
    }

    #[test]
    fn rest_trade_uses_fee_estimate_names() {
        let trade: TradeResponse = serde_json::from_value(serde_json::json!({
            "id": 9_007_199_254_740_993_i64,
            "trade_id": "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3",
            "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
            "taker_pubkey": "Taker111111111111111111111111111111111111111",
            "maker_pubkey": "Maker111111111111111111111111111111111111111",
            "side": "ask",
            "size": "4.00000000",
            "price": "0.5500",
            "taker_fee_estimate": "0.006600",
            "maker_fee_estimate": "-0.000110",
            "executed_at": 1790685600000_i64
        }))
        .unwrap();
        assert_eq!(trade.id, 9_007_199_254_740_993);
        assert_eq!(trade.taker_fee_estimate, Some(Decimal::new(6600, 6)));
        assert_eq!(trade.maker_fee_estimate, Some(Decimal::new(-110, 6)));
    }
}
