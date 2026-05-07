//! Wire types for market responses (REST).

use std::collections::BTreeMap;

use crate::domain::market::Status;
use crate::domain::orderbook::wire::OrderbookResponse;
use crate::shared::{OrderBookId, PubkeyStr};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ─── Outcome wire types ─────────────────────────────────────────────────────

/// Raw outcome from the REST API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutcomeResponse {
    pub index: i16,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
}

// ─── Token wire types (REST) ────────────────────────────────────────────────

/// REST response for a deposit asset with its conditional mints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepositAssetResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    pub deposit_asset: String,
    pub id: i32,
    pub market_pubkey: String,
    pub vault: String,
    pub num_outcomes: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimals: Option<i16>,
    pub conditional_mints: Vec<ConditionalTokenResponse>,
    pub created_at: DateTime<Utc>,
}

/// REST response for a conditional mint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConditionalTokenResponse {
    pub id: i32,
    pub outcome_index: i16,
    pub token_address: String,
    pub symbol: Option<String>,
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deposit_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimals: Option<i16>,
    pub created_at: DateTime<Utc>,
}

/// REST response for deposit mints list.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DepositMintsResponse {
    pub market_pubkey: String,
    pub deposit_assets: Vec<DepositAssetResponse>,
    pub total: usize,
}

// ─── Market resolution wire types ───────────────────────────────────────────

/// Canonical market resolution kind returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MarketResolutionKind {
    SingleWinner,
    Scalar,
}

/// Payout numerator for a single outcome in a resolved market.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketResolutionPayout {
    pub outcome_index: i16,
    pub payout_numerator: i64,
}

/// Canonical payout-vector resolution returned by the REST API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketResolutionResponse {
    pub kind: MarketResolutionKind,
    pub payout_denominator: i64,
    pub payouts: Vec<MarketResolutionPayout>,
    pub single_winning_outcome: Option<i16>,
}

/// REST response for a single market.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    pub outcomes: Vec<OutcomeResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_image_url_low: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_image_url_medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_image_url_high: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub featured_rank: Option<i16>,
    pub market_pubkey: String,
    pub market_id: i64,
    pub oracle: String,
    pub question_id: String,
    pub condition_id: String,
    pub market_status: String,
    #[serde(default)]
    pub resolution: Option<MarketResolutionResponse>,
    pub created_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub settled_at: Option<DateTime<Utc>>,
    pub deposit_assets: Vec<DepositAssetResponse>,
    pub orderbooks: Vec<OrderbookResponse>,
}

impl MarketResponse {
    pub fn is_resolved(&self) -> bool {
        self.resolution.is_some()
    }

    pub fn single_winning_outcome(&self) -> Option<i16> {
        self.resolution
            .as_ref()
            .and_then(|resolution| resolution.single_winning_outcome)
    }

    pub fn has_single_winning_outcome(&self) -> bool {
        self.single_winning_outcome().is_some()
    }
}

/// REST response for paginated markets list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsResponse {
    pub markets: Vec<MarketResponse>,
    pub next_cursor: Option<i64>,
    pub has_more: bool,
}

/// REST response wrapping a single market (used by by-slug and by-pubkey endpoints).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SingleMarketResponse {
    pub market: MarketResponse,
}

/// Minimal search/featured result for a single orderbook.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchOrderbook {
    pub orderbook_id: OrderBookId,
    pub outcome_name: String,
    pub outcome_index: i16,
    pub deposit_base_asset: PubkeyStr,
    pub deposit_quote_asset: PubkeyStr,
    pub deposit_base_symbol: String,
    pub deposit_quote_symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_icon_url_low: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_icon_url_medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_icon_url_high: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_icon_url_low: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_icon_url_medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_icon_url_high: Option<String>,
    pub conditional_base_mint: PubkeyStr,
    pub conditional_quote_mint: PubkeyStr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome_icon_url_low: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome_icon_url_medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome_icon_url_high: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional_base_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional_quote_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_mid_price: Option<Decimal>,
}

/// Minimal market result for search and featured listings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketSearchResult {
    pub slug: String,
    pub market_name: String,
    pub market_status: Status,
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub featured_rank: i16,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
    pub orderbooks: Vec<SearchOrderbook>,
}

/// Orderbooks for a single outcome within a market search result.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchOutcomeGroup {
    pub outcome_index: i16,
    pub outcome_name: String,
    pub outcome_icon_url_low: Option<String>,
    pub outcome_icon_url_medium: Option<String>,
    pub outcome_icon_url_high: Option<String>,
    pub orderbooks: Vec<SearchOrderbook>,
    pub market_name: String,
    pub market_slug: String,
    pub market_icon_url_low: Option<String>,
    pub market_icon_url_medium: Option<String>,
    pub market_icon_url_high: Option<String>,
}

impl MarketSearchResult {
    pub fn orderbooks_by_outcome(&self) -> Vec<SearchOutcomeGroup> {
        let mut groups: BTreeMap<i16, SearchOutcomeGroup> = BTreeMap::new();
        for orderbook in &self.orderbooks {
            groups
                .entry(orderbook.outcome_index)
                .or_insert_with(|| SearchOutcomeGroup {
                    market_name: self.market_name.clone(),
                    market_slug: self.slug.clone(),
                    market_icon_url_low: self.icon_url_low.clone(),
                    market_icon_url_medium: self.icon_url_medium.clone(),
                    market_icon_url_high: self.icon_url_high.clone(),
                    outcome_index: orderbook.outcome_index,
                    outcome_name: orderbook.outcome_name.clone(),
                    outcome_icon_url_low: orderbook.outcome_icon_url_low.clone(),
                    outcome_icon_url_medium: orderbook.outcome_icon_url_medium.clone(),
                    outcome_icon_url_high: orderbook.outcome_icon_url_high.clone(),
                    orderbooks: Vec::new(),
                })
                .orderbooks
                .push(orderbook.clone());
        }
        groups.into_values().collect()
    }
}

// ─── Global deposit asset wire types ────────────────────────────────────────

/// REST response for a single globally whitelisted deposit asset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalDepositAssetResponse {
    pub id: i32,
    pub mint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
    pub decimals: Option<i16>,
    pub whitelist_index: i16,
    pub active: bool,
}

/// REST response envelope for the global deposit asset whitelist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalDepositAssetsListResponse {
    pub assets: Vec<GlobalDepositAssetResponse>,
    pub total: usize,
}

/// WS market lifecycle event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type")]
pub enum MarketEvent {
    #[serde(rename = "settled")]
    Settled { market_pubkey: String },
    #[serde(rename = "created")]
    Created { market_pubkey: String },
    #[serde(rename = "opened")]
    Opened { market_pubkey: String },
    #[serde(rename = "paused")]
    Paused { market_pubkey: String },
    #[serde(rename = "orderbook_created")]
    OrderbookCreated {
        market_pubkey: String,
        orderbook_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_resolution_deserializes_single_winner() {
        let resolution: MarketResolutionResponse = serde_json::from_str(
            r#"{
                "kind": "single_winner",
                "payout_denominator": 1,
                "payouts": [
                    { "outcome_index": 0, "payout_numerator": 0 },
                    { "outcome_index": 1, "payout_numerator": 1 }
                ],
                "single_winning_outcome": 1
            }"#,
        )
        .unwrap();

        assert_eq!(resolution.kind, MarketResolutionKind::SingleWinner);
        assert_eq!(resolution.payout_denominator, 1);
        assert_eq!(resolution.single_winning_outcome, Some(1));
        assert_eq!(resolution.payouts[1].payout_numerator, 1);
    }

    #[test]
    fn market_resolution_deserializes_scalar() {
        let resolution: MarketResolutionResponse = serde_json::from_str(
            r#"{
                "kind": "scalar",
                "payout_denominator": 10,
                "payouts": [
                    { "outcome_index": 0, "payout_numerator": 7 },
                    { "outcome_index": 1, "payout_numerator": 3 }
                ],
                "single_winning_outcome": null
            }"#,
        )
        .unwrap();

        assert_eq!(resolution.kind, MarketResolutionKind::Scalar);
        assert_eq!(resolution.payout_denominator, 10);
        assert_eq!(resolution.single_winning_outcome, None);
        assert_eq!(resolution.payouts[0].payout_numerator, 7);
        assert_eq!(resolution.payouts[1].payout_numerator, 3);
    }
}
