//! Types for platform / market / orderbook / category / deposit-token metrics.
//!
//! Mirrors the `dto::metrics` types on the backend. `Decimal` fields deserialize
//! from JSON strings via `rust_decimal`'s `serde-str` feature; `PubkeyStr` and
//! `OrderBookId` newtypes are serialization-transparent.

use crate::shared::{OrderBookId, PubkeyStr};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ─── Platform ────────────────────────────────────────────────────────────────

/// `GET /api/metrics/platform` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlatformMetrics {
    pub volume_24h_usd: Decimal,
    pub volume_7d_usd: Decimal,
    pub volume_30d_usd: Decimal,
    pub volume_total_usd: Decimal,
    pub taker_bid_volume_24h_usd: Decimal,
    pub taker_bid_volume_7d_usd: Decimal,
    pub taker_bid_volume_30d_usd: Decimal,
    pub taker_bid_volume_total_usd: Decimal,
    pub taker_ask_volume_24h_usd: Decimal,
    pub taker_ask_volume_7d_usd: Decimal,
    pub taker_ask_volume_30d_usd: Decimal,
    pub taker_ask_volume_total_usd: Decimal,
    pub taker_bid_ask_imbalance_24h_pct: Decimal,
    pub taker_bid_ask_imbalance_7d_pct: Decimal,
    pub taker_bid_ask_imbalance_30d_pct: Decimal,
    pub taker_bid_ask_imbalance_total_pct: Decimal,
    pub unique_traders_24h: i32,
    pub unique_traders_7d: i32,
    pub unique_traders_30d: i32,
    pub active_markets: i64,
    pub active_orderbooks: i64,
    pub deposit_token_volumes: Vec<DepositTokenVolumeMetrics>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

// ─── Market (listing + detail) ───────────────────────────────────────────────

/// Entry in `GET /api/metrics/markets`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketVolumeMetrics {
    pub market_pubkey: PubkeyStr,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub market_name: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    pub volume_24h_usd: Decimal,
    pub volume_7d_usd: Decimal,
    pub volume_30d_usd: Decimal,
    pub volume_total_usd: Decimal,
    pub taker_bid_volume_24h_usd: Decimal,
    pub taker_bid_volume_7d_usd: Decimal,
    pub taker_bid_volume_30d_usd: Decimal,
    pub taker_bid_volume_total_usd: Decimal,
    pub taker_ask_volume_24h_usd: Decimal,
    pub taker_ask_volume_7d_usd: Decimal,
    pub taker_ask_volume_30d_usd: Decimal,
    pub taker_ask_volume_total_usd: Decimal,
    pub taker_bid_ask_imbalance_24h_pct: Decimal,
    pub taker_bid_ask_imbalance_7d_pct: Decimal,
    pub taker_bid_ask_imbalance_30d_pct: Decimal,
    pub taker_bid_ask_imbalance_total_pct: Decimal,
    pub unique_traders_24h: i32,
    pub unique_traders_7d: i32,
    pub unique_traders_30d: i32,
    pub category_volume_share_24h_pct: Decimal,
    pub platform_volume_share_24h_pct: Decimal,
}

/// `GET /api/metrics/markets` envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketsMetrics {
    pub markets: Vec<MarketVolumeMetrics>,
    pub total: usize,
}

impl Default for MarketsMetrics {
    fn default() -> Self {
        Self {
            markets: Vec::new(),
            total: 0,
        }
    }
}

/// Per-outcome breakdown inside `MarketDetailMetrics`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutcomeVolumeMetrics {
    pub outcome_index: Option<i16>,
    #[serde(default)]
    pub outcome_name: Option<String>,
    pub volume_24h_usd: Decimal,
    pub volume_7d_usd: Decimal,
    pub volume_30d_usd: Decimal,
    pub volume_total_usd: Decimal,
    pub taker_bid_volume_24h_usd: Decimal,
    pub taker_bid_volume_7d_usd: Decimal,
    pub taker_bid_volume_30d_usd: Decimal,
    pub taker_bid_volume_total_usd: Decimal,
    pub taker_ask_volume_24h_usd: Decimal,
    pub taker_ask_volume_7d_usd: Decimal,
    pub taker_ask_volume_30d_usd: Decimal,
    pub taker_ask_volume_total_usd: Decimal,
    pub taker_bid_ask_imbalance_24h_pct: Decimal,
    pub taker_bid_ask_imbalance_7d_pct: Decimal,
    pub taker_bid_ask_imbalance_30d_pct: Decimal,
    pub taker_bid_ask_imbalance_total_pct: Decimal,
    pub unique_traders_24h: i32,
    pub unique_traders_7d: i32,
    pub unique_traders_30d: i32,
    pub volume_share_24h_pct: Decimal,
}

/// Per-orderbook breakdown inside `MarketDetailMetrics`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketOrderbookVolumeMetrics {
    pub orderbook_id: OrderBookId,
    pub outcome_index: Option<i16>,
    #[serde(default)]
    pub outcome_name: Option<String>,
    pub base_deposit_asset: PubkeyStr,
    #[serde(default)]
    pub base_deposit_symbol: Option<String>,
    pub quote_deposit_asset: PubkeyStr,
    #[serde(default)]
    pub quote_deposit_symbol: Option<String>,
    pub volume_24h_usd: Decimal,
    pub volume_7d_usd: Decimal,
    pub volume_30d_usd: Decimal,
    pub volume_total_usd: Decimal,
    pub volume_24h_base: Decimal,
    pub volume_7d_base: Decimal,
    pub volume_30d_base: Decimal,
    pub volume_total_base: Decimal,
    pub volume_24h_quote: Decimal,
    pub volume_7d_quote: Decimal,
    pub volume_30d_quote: Decimal,
    pub volume_total_quote: Decimal,
    pub taker_bid_volume_24h_usd: Decimal,
    pub taker_bid_volume_7d_usd: Decimal,
    pub taker_bid_volume_30d_usd: Decimal,
    pub taker_bid_volume_total_usd: Decimal,
    pub taker_bid_volume_24h_base: Decimal,
    pub taker_bid_volume_7d_base: Decimal,
    pub taker_bid_volume_30d_base: Decimal,
    pub taker_bid_volume_total_base: Decimal,
    pub taker_bid_volume_24h_quote: Decimal,
    pub taker_bid_volume_7d_quote: Decimal,
    pub taker_bid_volume_30d_quote: Decimal,
    pub taker_bid_volume_total_quote: Decimal,
    pub taker_ask_volume_24h_usd: Decimal,
    pub taker_ask_volume_7d_usd: Decimal,
    pub taker_ask_volume_30d_usd: Decimal,
    pub taker_ask_volume_total_usd: Decimal,
    pub taker_ask_volume_24h_base: Decimal,
    pub taker_ask_volume_7d_base: Decimal,
    pub taker_ask_volume_30d_base: Decimal,
    pub taker_ask_volume_total_base: Decimal,
    pub taker_ask_volume_24h_quote: Decimal,
    pub taker_ask_volume_7d_quote: Decimal,
    pub taker_ask_volume_30d_quote: Decimal,
    pub taker_ask_volume_total_quote: Decimal,
    pub taker_bid_ask_imbalance_24h_pct: Decimal,
    pub taker_bid_ask_imbalance_7d_pct: Decimal,
    pub taker_bid_ask_imbalance_30d_pct: Decimal,
    pub taker_bid_ask_imbalance_total_pct: Decimal,
    pub volume_share_24h_pct: Decimal,
}

/// `GET /api/metrics/markets/{market_pubkey}` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketDetailMetrics {
    pub market_pubkey: PubkeyStr,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub market_name: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    pub volume_24h_usd: Decimal,
    pub volume_7d_usd: Decimal,
    pub volume_30d_usd: Decimal,
    pub volume_total_usd: Decimal,
    pub taker_bid_volume_24h_usd: Decimal,
    pub taker_bid_volume_7d_usd: Decimal,
    pub taker_bid_volume_30d_usd: Decimal,
    pub taker_bid_volume_total_usd: Decimal,
    pub taker_ask_volume_24h_usd: Decimal,
    pub taker_ask_volume_7d_usd: Decimal,
    pub taker_ask_volume_30d_usd: Decimal,
    pub taker_ask_volume_total_usd: Decimal,
    pub taker_bid_ask_imbalance_24h_pct: Decimal,
    pub taker_bid_ask_imbalance_7d_pct: Decimal,
    pub taker_bid_ask_imbalance_30d_pct: Decimal,
    pub taker_bid_ask_imbalance_total_pct: Decimal,
    pub unique_traders_24h: i32,
    pub unique_traders_7d: i32,
    pub unique_traders_30d: i32,
    pub category_volume_share_24h_pct: Decimal,
    pub platform_volume_share_24h_pct: Decimal,
    pub outcome_volumes: Vec<OutcomeVolumeMetrics>,
    pub orderbook_volumes: Vec<MarketOrderbookVolumeMetrics>,
    pub deposit_token_volumes: Vec<DepositTokenVolumeMetrics>,
}

// ─── Orderbook ───────────────────────────────────────────────────────────────

/// `GET /api/metrics/orderbooks/{orderbook_id}` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderbookVolumeMetrics {
    pub orderbook_id: OrderBookId,
    pub market_pubkey: PubkeyStr,
    pub outcome_index: Option<i16>,
    #[serde(default)]
    pub outcome_name: Option<String>,
    pub base_deposit_asset: PubkeyStr,
    #[serde(default)]
    pub base_deposit_symbol: Option<String>,
    pub quote_deposit_asset: PubkeyStr,
    #[serde(default)]
    pub quote_deposit_symbol: Option<String>,
    pub volume_24h_usd: Decimal,
    pub volume_7d_usd: Decimal,
    pub volume_30d_usd: Decimal,
    pub volume_total_usd: Decimal,
    pub volume_24h_base: Decimal,
    pub volume_7d_base: Decimal,
    pub volume_30d_base: Decimal,
    pub volume_total_base: Decimal,
    pub volume_24h_quote: Decimal,
    pub volume_7d_quote: Decimal,
    pub volume_30d_quote: Decimal,
    pub volume_total_quote: Decimal,
    pub taker_bid_volume_24h_usd: Decimal,
    pub taker_bid_volume_7d_usd: Decimal,
    pub taker_bid_volume_30d_usd: Decimal,
    pub taker_bid_volume_total_usd: Decimal,
    pub taker_bid_volume_24h_base: Decimal,
    pub taker_bid_volume_7d_base: Decimal,
    pub taker_bid_volume_30d_base: Decimal,
    pub taker_bid_volume_total_base: Decimal,
    pub taker_bid_volume_24h_quote: Decimal,
    pub taker_bid_volume_7d_quote: Decimal,
    pub taker_bid_volume_30d_quote: Decimal,
    pub taker_bid_volume_total_quote: Decimal,
    pub taker_ask_volume_24h_usd: Decimal,
    pub taker_ask_volume_7d_usd: Decimal,
    pub taker_ask_volume_30d_usd: Decimal,
    pub taker_ask_volume_total_usd: Decimal,
    pub taker_ask_volume_24h_base: Decimal,
    pub taker_ask_volume_7d_base: Decimal,
    pub taker_ask_volume_30d_base: Decimal,
    pub taker_ask_volume_total_base: Decimal,
    pub taker_ask_volume_24h_quote: Decimal,
    pub taker_ask_volume_7d_quote: Decimal,
    pub taker_ask_volume_30d_quote: Decimal,
    pub taker_ask_volume_total_quote: Decimal,
    pub taker_bid_ask_imbalance_24h_pct: Decimal,
    pub taker_bid_ask_imbalance_7d_pct: Decimal,
    pub taker_bid_ask_imbalance_30d_pct: Decimal,
    pub taker_bid_ask_imbalance_total_pct: Decimal,
    pub unique_traders_24h: i32,
    pub unique_traders_7d: i32,
    pub unique_traders_30d: i32,
    pub market_volume_share_24h_pct: Decimal,
}

// ─── Category ────────────────────────────────────────────────────────────────

/// Entry in `GET /api/metrics/categories` and the single response from
/// `GET /api/metrics/categories/{category}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryVolumeMetrics {
    pub category: String,
    pub volume_24h_usd: Decimal,
    pub volume_7d_usd: Decimal,
    pub volume_30d_usd: Decimal,
    pub volume_total_usd: Decimal,
    pub taker_bid_volume_24h_usd: Decimal,
    pub taker_bid_volume_7d_usd: Decimal,
    pub taker_bid_volume_30d_usd: Decimal,
    pub taker_bid_volume_total_usd: Decimal,
    pub taker_ask_volume_24h_usd: Decimal,
    pub taker_ask_volume_7d_usd: Decimal,
    pub taker_ask_volume_30d_usd: Decimal,
    pub taker_ask_volume_total_usd: Decimal,
    pub taker_bid_ask_imbalance_24h_pct: Decimal,
    pub taker_bid_ask_imbalance_7d_pct: Decimal,
    pub taker_bid_ask_imbalance_30d_pct: Decimal,
    pub taker_bid_ask_imbalance_total_pct: Decimal,
    pub unique_traders_24h: i32,
    pub unique_traders_7d: i32,
    pub unique_traders_30d: i32,
    pub platform_volume_share_24h_pct: Decimal,
    pub deposit_token_volumes: Vec<DepositTokenVolumeMetrics>,
}

/// `GET /api/metrics/categories` envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoriesMetrics {
    pub categories: Vec<CategoryVolumeMetrics>,
}

// ─── Deposit tokens ──────────────────────────────────────────────────────────

/// Entry in `GET /api/metrics/deposit-tokens`, also nested in platform/market/category
/// responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepositTokenVolumeMetrics {
    pub deposit_asset: PubkeyStr,
    #[serde(default)]
    pub symbol: Option<String>,
    pub volume_24h_usd: Decimal,
    pub volume_7d_usd: Decimal,
    pub volume_30d_usd: Decimal,
    pub volume_total_usd: Decimal,
    pub taker_bid_volume_24h_usd: Decimal,
    pub taker_bid_volume_7d_usd: Decimal,
    pub taker_bid_volume_30d_usd: Decimal,
    pub taker_bid_volume_total_usd: Decimal,
    pub taker_ask_volume_24h_usd: Decimal,
    pub taker_ask_volume_7d_usd: Decimal,
    pub taker_ask_volume_30d_usd: Decimal,
    pub taker_ask_volume_total_usd: Decimal,
    pub taker_bid_ask_imbalance_24h_pct: Decimal,
    pub taker_bid_ask_imbalance_7d_pct: Decimal,
    pub taker_bid_ask_imbalance_30d_pct: Decimal,
    pub taker_bid_ask_imbalance_total_pct: Decimal,
    pub volume_share_24h_pct: Decimal,
}

/// `GET /api/metrics/deposit-tokens` envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepositTokensMetrics {
    pub deposit_tokens: Vec<DepositTokenVolumeMetrics>,
}

// ─── Leaderboard ─────────────────────────────────────────────────────────────

/// Entry in `GET /api/metrics/leaderboard/markets`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LeaderboardEntry {
    pub rank: i32,
    pub market_pubkey: PubkeyStr,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub market_name: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    pub volume_24h_usd: Decimal,
    pub category_volume_share_24h_pct: Decimal,
    pub platform_volume_share_24h_pct: Decimal,
}

/// `GET /api/metrics/leaderboard/markets` envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Leaderboard {
    pub entries: Vec<LeaderboardEntry>,
    pub period: String,
}

// ─── History ─────────────────────────────────────────────────────────────────

/// Bucket in `GET /api/metrics/history/{scope}/{scope_key}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HistoryPoint {
    /// Bucket start, Unix epoch milliseconds.
    pub bucket_start: i64,
    pub volume_usd: Decimal,
}

/// `GET /api/metrics/history/{scope}/{scope_key}` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricsHistory {
    pub scope: String,
    pub scope_key: String,
    pub resolution: String,
    pub points: Vec<HistoryPoint>,
}

// ─── Queries (SDK-side; not wire-returned from the backend) ──────────────────

/// Query parameters for `GET /api/metrics/markets` (reserved for future filters).
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct MarketsMetricsQuery {}

/// Query parameters for `GET /api/metrics/markets/{market_pubkey}` (reserved for future filters).
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct MarketMetricsQuery {}

/// Query parameters for `GET /api/metrics/orderbooks/{orderbook_id}` (reserved for future filters).
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct OrderbookMetricsQuery {}

/// Query parameters for `GET /api/metrics/categories/{category}` (reserved for future filters).
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct CategoryMetricsQuery {}

/// Query parameters for `GET /api/metrics/history/{scope}/{scope_key}`.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MetricsHistoryQuery {
    pub resolution: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

impl Default for MetricsHistoryQuery {
    fn default() -> Self {
        Self {
            resolution: "1h".to_string(),
            from: None,
            to: None,
            limit: None,
        }
    }
}
