#![doc = include_str!("README.md")]

pub mod client;
pub mod wire;

pub use client::Metrics;
pub use wire::{
    CategoriesMetrics, CategoryMetricsQuery, CategoryVolumeMetrics, DepositTokenVolumeHistory,
    DepositTokenVolumeHistoryPoint, DepositTokenVolumeHistoryPointToken,
    DepositTokenVolumeHistoryQuery, DepositTokenVolumeHistoryToken, DepositTokenVolumeMetrics,
    DepositTokensMetrics, HistoryPoint, Leaderboard, LeaderboardEntry, MarketDetailMetrics,
    MarketMetricsQuery, MarketOrderbookVolumeMetrics, MarketVolumeMetrics, MarketsMetrics,
    MarketsMetricsQuery, MetricsHistory, MetricsHistoryQuery, OpenInterestHistory,
    OpenInterestHistoryDepositAsset, OpenInterestHistoryPoint,
    OpenInterestHistoryPointDepositAsset, OpenInterestHistoryQuery, OrderbookMetricsQuery,
    OrderbookVolumeMetrics, OutcomeVolumeMetrics, PlatformMetrics, UniqueTradersHistory,
    UniqueTradersHistoryPoint, UniqueTradersHistoryQuery, UniqueTradersHistoryScope, UserMetrics,
};
