//! Types for platform / market / orderbook / category / deposit-token metrics.
//!
//! Mirrors the `dto::metrics` types on the backend. `Decimal` fields deserialize
//! from JSON strings via `rust_decimal`'s `serde-str` feature; `PubkeyStr` and
//! `OrderBookId` newtypes are serialization-transparent.

use crate::shared::{OrderBookId, PubkeyStr, Resolution};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ─── Orderbook tickers (batch) ───────────────────────────────────────────────

/// One entry in `GET /api/metrics/orderbooks/tickers`. Same shape (BBO +
/// midpoint) as the WS `Ticker` stream, delivered in batch over REST.
/// Price fields are `None` when the orderbook has no liquidity yet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderbookTickerEntry {
    pub orderbook_id: OrderBookId,
    pub market_pubkey: PubkeyStr,
    #[serde(default)]
    pub outcome_index: Option<i16>,
    #[serde(default)]
    pub outcome_name: Option<String>,
    #[serde(default)]
    pub outcome_name_long: Option<String>,
    pub base_deposit_asset: PubkeyStr,
    pub quote_deposit_asset: PubkeyStr,
    #[serde(default)]
    pub best_bid: Option<Decimal>,
    #[serde(default)]
    pub best_ask: Option<Decimal>,
    #[serde(default)]
    pub midpoint: Option<Decimal>,
    #[serde(default)]
    pub computed_at: Option<DateTime<Utc>>,
}

/// `GET /api/metrics/orderbooks/tickers` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderbookTickersResponse {
    pub tickers: Vec<OrderbookTickerEntry>,
}

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
    pub open_interest_usd: Decimal,
    pub fees_24h_usd: Decimal,
    pub fees_7d_usd: Decimal,
    pub fees_30d_usd: Decimal,
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
    #[serde(default)]
    pub outcome_name_long: Option<String>,
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
    #[serde(default)]
    pub outcome_name_long: Option<String>,
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
    #[serde(default)]
    pub outcome_name_long: Option<String>,
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

// ─── Deposit-token volume history ────────────────────────────────────────────

/// Token summary entry in `GET /api/metrics/deposit-tokens/volume-history`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepositTokenVolumeHistoryToken {
    pub rank: i32,
    pub deposit_asset: PubkeyStr,
    #[serde(default)]
    pub symbol: Option<String>,
    pub volume_total_usd: Decimal,
}

/// Per-token stacked-bar entry for one daily volume history point.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepositTokenVolumeHistoryPointToken {
    pub deposit_asset: PubkeyStr,
    #[serde(default)]
    pub symbol: Option<String>,
    pub volume_usd: Decimal,
}

/// Daily point in `GET /api/metrics/deposit-tokens/volume-history`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepositTokenVolumeHistoryPoint {
    /// Bucket start, deserialized from Unix epoch milliseconds.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub bucket_start: DateTime<Utc>,
    /// Calendar day label in `YYYY-MM-DD` format.
    pub bucket_start_date: NaiveDate,
    pub total_volume_usd: Decimal,
    pub cumulative_volume_usd: Decimal,
    pub deposit_token_volumes: Vec<DepositTokenVolumeHistoryPointToken>,
}

/// `GET /api/metrics/deposit-tokens/volume-history` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DepositTokenVolumeHistory {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub resolution: Resolution,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub from: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub to: DateTime<Utc>,
    pub volume_total_usd: Decimal,
    pub total_days: u32,
    pub deposit_tokens: Vec<DepositTokenVolumeHistoryToken>,
    pub points: Vec<DepositTokenVolumeHistoryPoint>,
}

// ─── Open-interest history ───────────────────────────────────────────────────

/// Deposit-asset summary entry in `GET /api/metrics/open-interest/history`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenInterestHistoryDepositAsset {
    pub rank: i32,
    pub deposit_asset: PubkeyStr,
    #[serde(default)]
    pub symbol: Option<String>,
    pub latest_open_interest_usd: Decimal,
    pub max_open_interest_usd: Decimal,
}

/// Per-deposit-asset entry for one daily open-interest history point.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenInterestHistoryPointDepositAsset {
    pub deposit_asset: PubkeyStr,
    #[serde(default)]
    pub symbol: Option<String>,
    pub open_interest_usd: Decimal,
}

/// Daily point in `GET /api/metrics/open-interest/history`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenInterestHistoryPoint {
    /// Bucket start, deserialized from Unix epoch milliseconds for the UTC day start.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub bucket_start: DateTime<Utc>,
    /// UTC calendar day label in `YYYY-MM-DD` format.
    pub bucket_start_date: NaiveDate,
    pub total_open_interest_usd: Decimal,
    pub deposit_asset_open_interest: Vec<OpenInterestHistoryPointDepositAsset>,
}

/// `GET /api/metrics/open-interest/history` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenInterestHistory {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub resolution: Resolution,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub from: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub to: DateTime<Utc>,
    pub latest_open_interest_usd: Decimal,
    pub total_days: u32,
    pub deposit_assets: Vec<OpenInterestHistoryDepositAsset>,
    pub points: Vec<OpenInterestHistoryPoint>,
}

// ─── Unique-traders history ──────────────────────────────────────────────────

/// Scope vocabulary for `GET /api/metrics/unique-traders/history`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UniqueTradersHistoryScope {
    Platform,
    Market,
    Orderbook,
    Category,
    Outcome,
}

/// Daily point in `GET /api/metrics/unique-traders/history`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UniqueTradersHistoryPoint {
    /// Bucket start, deserialized from Unix epoch milliseconds for the UTC day start.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub bucket_start: DateTime<Utc>,
    /// UTC calendar day label in `YYYY-MM-DD` format.
    pub bucket_start_date: NaiveDate,
    pub unique_traders: u64,
}

/// `GET /api/metrics/unique-traders/history` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UniqueTradersHistory {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub resolution: Resolution,
    pub scope: UniqueTradersHistoryScope,
    pub scope_key: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub from: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub to: DateTime<Utc>,
    pub latest_unique_traders: u64,
    pub total_days: u32,
    pub points: Vec<UniqueTradersHistoryPoint>,
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
    /// Bucket start, deserialized from Unix epoch milliseconds.
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub bucket_start: DateTime<Utc>,
    pub volume_usd: Decimal,
}

/// `GET /api/metrics/history/{scope}/{scope_key}` response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricsHistory {
    pub scope: String,
    pub scope_key: String,
    pub resolution: Resolution,
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

/// Query parameters for `GET /api/metrics/deposit-tokens/volume-history`.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct DepositTokenVolumeHistoryQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Query parameters for `GET /api/metrics/open-interest/history`.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct OpenInterestHistoryQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Query parameters for `GET /api/metrics/unique-traders/history`.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct UniqueTradersHistoryQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<UniqueTradersHistoryScope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

/// Query parameters for `GET /api/metrics/history/{scope}/{scope_key}`.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MetricsHistoryQuery {
    pub resolution: Resolution,
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
            resolution: Resolution::Hour1,
            from: None,
            to: None,
            limit: None,
        }
    }
}

/// Per-wallet trading + referral aggregates. Response shape of
/// `metrics().user`, `metrics().user_with_cookies`, and `metrics().user_by_wallet`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UserMetrics {
    pub wallet_address: PubkeyStr,
    pub total_outcomes_traded: i64,
    pub total_volume_usd: Decimal,
    pub total_referrals_used: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use serde_json::json;
    use std::str::FromStr;

    fn dt(ms: i64) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_millis(ms).unwrap()
    }

    fn date(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn platform_metrics_deserializes_open_interest_and_fee_fields() {
        let metrics: PlatformMetrics = serde_json::from_value(json!({
            "volume_24h_usd": "1",
            "volume_7d_usd": "2",
            "volume_30d_usd": "3",
            "volume_total_usd": "4",
            "taker_bid_volume_24h_usd": "5",
            "taker_bid_volume_7d_usd": "6",
            "taker_bid_volume_30d_usd": "7",
            "taker_bid_volume_total_usd": "8",
            "taker_ask_volume_24h_usd": "9",
            "taker_ask_volume_7d_usd": "10",
            "taker_ask_volume_30d_usd": "11",
            "taker_ask_volume_total_usd": "12",
            "taker_bid_ask_imbalance_24h_pct": "13",
            "taker_bid_ask_imbalance_7d_pct": "14",
            "taker_bid_ask_imbalance_30d_pct": "15",
            "taker_bid_ask_imbalance_total_pct": "16",
            "open_interest_usd": "12345.67",
            "fees_24h_usd": "0",
            "fees_7d_usd": "0",
            "fees_30d_usd": "0",
            "unique_traders_24h": 17,
            "unique_traders_7d": 18,
            "unique_traders_30d": 19,
            "active_markets": 20,
            "active_orderbooks": 21,
            "deposit_token_volumes": [],
            "updated_at": null
        }))
        .unwrap();

        assert_eq!(
            metrics.open_interest_usd,
            Decimal::from_str("12345.67").unwrap()
        );
        assert_eq!(metrics.fees_24h_usd, Decimal::ZERO);
        assert_eq!(metrics.fees_7d_usd, Decimal::ZERO);
        assert_eq!(metrics.fees_30d_usd, Decimal::ZERO);
    }

    #[test]
    fn deposit_token_volume_history_query_serializes_bounds_and_limit() {
        let query = DepositTokenVolumeHistoryQuery {
            from: Some(1_704_067_200_000),
            to: Some(1_760_000_000_000),
            limit: Some(365),
        };

        let query_string = serde_urlencoded::to_string(query).unwrap();
        assert_eq!(
            query_string,
            "from=1704067200000&to=1760000000000&limit=365"
        );
    }

    #[test]
    fn deposit_token_volume_history_deserializes_daily_points() {
        let history: DepositTokenVolumeHistory = serde_json::from_value(json!({
            "timestamp": 1_760_000_000_000i64,
            "resolution": "1d",
            "from": 1_704_067_200_000i64,
            "to": 1_760_000_000_000i64,
            "volume_total_usd": "123456.78",
            "total_days": 365,
            "deposit_tokens": [{
                "rank": 1,
                "deposit_asset": "DepositAsset",
                "symbol": "BTC",
                "volume_total_usd": "90000.00"
            }],
            "points": [{
                "bucket_start": 1_704_067_200_000i64,
                "bucket_start_date": "2024-01-01",
                "total_volume_usd": "1000.00",
                "cumulative_volume_usd": "1000.00",
                "deposit_token_volumes": [{
                    "deposit_asset": "DepositAsset",
                    "symbol": "BTC",
                    "volume_usd": "700.00"
                }, {
                    "deposit_asset": "OtherDepositAsset",
                    "symbol": "ETH",
                    "volume_usd": "300.00"
                }]
            }]
        }))
        .unwrap();

        assert_eq!(history.resolution, Resolution::Day1);
        assert_eq!(
            history.volume_total_usd,
            Decimal::from_str("123456.78").unwrap()
        );
        assert_eq!(history.total_days, 365);
        assert_eq!(history.deposit_tokens.len(), 1);
        assert_eq!(history.deposit_tokens[0].rank, 1);
        assert_eq!(
            history.deposit_tokens[0].deposit_asset.as_str(),
            "DepositAsset"
        );
        assert_eq!(history.deposit_tokens[0].symbol.as_deref(), Some("BTC"));
        assert_eq!(
            history.deposit_tokens[0].volume_total_usd,
            Decimal::from_str("90000.00").unwrap()
        );

        let point = &history.points[0];
        assert_eq!(point.bucket_start, dt(1_704_067_200_000));
        assert_eq!(point.bucket_start_date, date(2024, 1, 1));
        assert_eq!(
            point.total_volume_usd,
            Decimal::from_str("1000.00").unwrap()
        );
        assert_eq!(
            point.cumulative_volume_usd,
            Decimal::from_str("1000.00").unwrap()
        );
        assert_eq!(point.deposit_token_volumes.len(), 2);
        assert_eq!(
            point.deposit_token_volumes[0].volume_usd,
            Decimal::from_str("700.00").unwrap()
        );
    }

    #[test]
    fn open_interest_history_query_serializes_bounds_and_limit() {
        let query = OpenInterestHistoryQuery {
            from: Some(1_704_067_200_000),
            to: Some(1_760_000_000_000),
            limit: Some(30),
        };

        let query_string = serde_urlencoded::to_string(query).unwrap();
        assert_eq!(query_string, "from=1704067200000&to=1760000000000&limit=30");
    }

    #[test]
    fn open_interest_history_deserializes_daily_snapshots() {
        let history: OpenInterestHistory = serde_json::from_value(json!({
            "timestamp": 1_760_000_000_000i64,
            "resolution": "1d",
            "from": 1_704_067_200_000i64,
            "to": 1_760_000_000_000i64,
            "latest_open_interest_usd": "123456.78",
            "total_days": 30,
            "deposit_assets": [{
                "rank": 1,
                "deposit_asset": "DepositAsset",
                "symbol": "BTC",
                "latest_open_interest_usd": "90000.00",
                "max_open_interest_usd": "100000.00"
            }],
            "points": [{
                "bucket_start": 1_704_067_200_000i64,
                "bucket_start_date": "2024-01-01",
                "total_open_interest_usd": "123456.78",
                "deposit_asset_open_interest": [{
                    "deposit_asset": "DepositAsset",
                    "symbol": "BTC",
                    "open_interest_usd": "90000.00"
                }, {
                    "deposit_asset": "OtherDepositAsset",
                    "symbol": "ETH",
                    "open_interest_usd": "0"
                }]
            }]
        }))
        .unwrap();

        assert_eq!(history.resolution, Resolution::Day1);
        assert_eq!(
            history.latest_open_interest_usd,
            Decimal::from_str("123456.78").unwrap()
        );
        assert_eq!(history.total_days, 30);
        assert_eq!(history.deposit_assets.len(), 1);
        assert_eq!(history.deposit_assets[0].rank, 1);
        assert_eq!(
            history.deposit_assets[0].deposit_asset.as_str(),
            "DepositAsset"
        );
        assert_eq!(history.deposit_assets[0].symbol.as_deref(), Some("BTC"));
        assert_eq!(
            history.deposit_assets[0].latest_open_interest_usd,
            Decimal::from_str("90000.00").unwrap()
        );
        assert_eq!(
            history.deposit_assets[0].max_open_interest_usd,
            Decimal::from_str("100000.00").unwrap()
        );

        let point = &history.points[0];
        assert_eq!(point.bucket_start, dt(1_704_067_200_000));
        assert_eq!(point.bucket_start_date, date(2024, 1, 1));
        assert_eq!(
            point.total_open_interest_usd,
            Decimal::from_str("123456.78").unwrap()
        );
        assert_eq!(point.deposit_asset_open_interest.len(), 2);
        assert_eq!(
            point.deposit_asset_open_interest[0].open_interest_usd,
            Decimal::from_str("90000.00").unwrap()
        );
        assert_eq!(
            point.deposit_asset_open_interest[1].open_interest_usd,
            Decimal::ZERO
        );
    }

    #[test]
    fn unique_traders_history_default_query_uses_backend_defaults() {
        let query_string =
            serde_urlencoded::to_string(UniqueTradersHistoryQuery::default()).unwrap();
        assert_eq!(query_string, "");
    }

    #[test]
    fn unique_traders_history_query_serializes_scope_bounds_and_limit() {
        let query = UniqueTradersHistoryQuery {
            scope: Some(UniqueTradersHistoryScope::Market),
            scope_key: Some("MarketPubkey".to_string()),
            from: Some(1_710_000_000_000),
            to: Some(1_720_000_000_000),
            limit: Some(30),
        };

        let query_string = serde_urlencoded::to_string(query).unwrap();
        assert_eq!(
            query_string,
            "scope=market&scope_key=MarketPubkey&from=1710000000000&to=1720000000000&limit=30"
        );
    }

    #[test]
    fn unique_traders_history_deserializes_daily_counts_and_preserves_zero_days() {
        let history: UniqueTradersHistory = serde_json::from_value(json!({
            "timestamp": 1_760_000_000_000i64,
            "resolution": "1d",
            "scope": "platform",
            "scope_key": "platform",
            "from": 1_710_000_000_000i64,
            "to": 1_720_000_000_000i64,
            "latest_unique_traders": 42,
            "total_days": 30,
            "points": [{
                "bucket_start": 1_710_000_000_000i64,
                "bucket_start_date": "2024-03-09",
                "unique_traders": 42
            }, {
                "bucket_start": 1_710_086_400_000i64,
                "bucket_start_date": "2024-03-10",
                "unique_traders": 0
            }]
        }))
        .unwrap();

        assert_eq!(history.resolution, Resolution::Day1);
        assert_eq!(history.scope, UniqueTradersHistoryScope::Platform);
        assert_eq!(history.scope_key, "platform");
        assert_eq!(history.from, dt(1_710_000_000_000));
        assert_eq!(history.to, dt(1_720_000_000_000));
        assert_eq!(history.latest_unique_traders, 42);
        assert_eq!(history.total_days, 30);
        assert_eq!(history.points.len(), 2);
        assert_eq!(history.points[0].bucket_start_date, date(2024, 3, 9));
        assert_eq!(history.points[0].unique_traders, 42);
        assert_eq!(history.points[1].bucket_start_date, date(2024, 3, 10));
        assert_eq!(history.points[1].unique_traders, 0);
    }
}
