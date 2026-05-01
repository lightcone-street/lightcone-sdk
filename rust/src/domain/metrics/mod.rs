#![doc = include_str!("README.md")]

pub mod client;
pub mod wire;

pub use client::Metrics;
pub use wire::{
    CategoriesMetrics, CategoryMetricsQuery, CategoryVolumeMetrics, DepositTokenVolumeMetrics,
    DepositTokensMetrics, HistoryPoint, Leaderboard, LeaderboardEntry, MarketDetailMetrics,
    MarketMetricsQuery, MarketOrderbookVolumeMetrics, MarketVolumeMetrics, MarketsMetrics,
    MarketsMetricsQuery, MetricsHistory, MetricsHistoryQuery, OrderbookMetricsQuery,
    OrderbookVolumeMetrics, OutcomeVolumeMetrics, PlatformMetrics, UserMetrics,
};
