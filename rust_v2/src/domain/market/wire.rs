//! Wire types for market responses (REST).

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
    pub icon_url: Option<String>,
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
    pub icon_url: Option<String>,
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
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deposit_symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
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
    pub banner_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
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
    pub winning_outcome: Option<i16>,
    pub has_winning_outcome: bool,
    pub created_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub settled_at: Option<DateTime<Utc>>,
    pub deposit_assets: Vec<DepositAssetResponse>,
    pub orderbooks: Vec<OrderbookResponse>,
}

/// REST response for paginated markets list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketsResponse {
    pub markets: Vec<MarketResponse>,
    pub total: usize,
    pub has_more: bool,
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
    pub base_icon_url: String,
    pub quote_icon_url: String,
    pub conditional_base_mint: PubkeyStr,
    pub conditional_quote_mint: PubkeyStr,
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
    pub icon_url: Option<String>,
    pub orderbooks: Vec<SearchOrderbook>,
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
