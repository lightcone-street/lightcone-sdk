#![doc = include_str!("../README.md")]

// ── Layer 1: Core ────────────────────────────────────────────────────────────

/// Shared newtypes used across all domains.
pub mod shared;

/// Domain modules (vertical slices): types, wire types, conversions, state.
pub mod domain;

/// On-chain program interaction: instructions, orders, PDAs, accounts.
pub mod program;

/// Unified SDK error types.
pub mod error;

/// Environment configuration: deployment targets, URLs, and program IDs.
pub mod env;

/// Network configuration (re-exports from [`env`]).
pub mod network;

/// RPC sub-client: PDA helpers, account fetchers, blockhash access.
#[cfg(feature = "http")]
pub mod rpc;

/// RPC failover: automatic switch to a backup Solana RPC on infrastructure errors.
#[cfg(feature = "http")]
pub mod rpc_failover;

// ── Layer 2: Auth ────────────────────────────────────────────────────────────

/// Authentication: message generation, credentials, login/logout.
pub mod auth;

// ── Layer 3: HTTP API ────────────────────────────────────────────────────────

/// HTTP client with retry policies.
#[cfg(feature = "http")]
pub mod http;

// ── Layer 4: WebSocket ───────────────────────────────────────────────────────

/// WebSocket client: messages, subscriptions, events.
pub mod ws;

// ── Layer 5: High-Level Client ───────────────────────────────────────────────

/// `LightconeClient` — the primary entry point.
#[cfg(feature = "http")]
pub mod client;

// ── Prelude ──────────────────────────────────────────────────────────────────

pub mod prelude {
    // Shared newtypes
    pub use crate::shared::{
        Denominator, DepositSource, OrderBookId, PubkeyStr, Resolution, Side, TimeInForce,
        TriggerType,
    };

    // Domain types — market (includes outcome + tokens)
    pub use crate::domain::market::outcome::Outcome;
    pub use crate::domain::market::tokens::{
        sort_by_display_priority, ConditionalToken, DepositAsset, GlobalDepositAsset,
        HasDisplayToken, Token, TokenMetadata, ValidatedTokens,
    };
    pub use crate::domain::market::{
        Market, MarketResolutionKind, MarketResolutionPayout, MarketResolutionResponse, Status,
    };

    // Domain types — orderbook
    pub use crate::domain::orderbook::{
        BookAggregation, OrderBookPair, OrderBookValidationError, OutcomeImpact,
    };

    // Domain types — order
    pub use crate::domain::order::{
        AnyOrder, CancelAllBody, CancelAllSuccess, CancelBody, CancelSuccess, ConditionalBalance,
        FillInfo, GlobalDepositBalance, GlobalDepositUpdate, LimitOrder, Order, OrderEvent,
        OrderStatus, OrderType, SubmitOrderResponse, SubmitOrderStatus, TriggerOrderUpdate,
        UserBalanceUpdate, UserDepositAssetBalance, UserMarketBalance, UserOpenLimitOrders,
        UserOrdersResponse, UserOutcomeBalance, UserSnapshotOrder, UserSnapshotOrderCommon,
        UserUpdate,
    };
    #[cfg(feature = "trigger_orders")]
    pub use crate::domain::order::{
        CancelTriggerBody, CancelTriggerSuccess, TriggerOrder, TriggerOrderResponse,
        UserTriggerOrders,
    };

    // Domain types — position (includes portfolio + token balances)
    pub use crate::domain::position::{
        DepositAssetMetadata, DepositTokenBalance, Portfolio, Position, PositionOutcome,
        TokenBalance, TokenBalanceComputedBase, TokenBalanceTokenType, WalletHolding,
    };

    // Domain types — trade, price history
    pub use crate::domain::price_history::{
        DepositAssetPriceEvent, DepositAssetPriceSnapshot, DepositAssetPriceTick,
        DepositAssetPricesSnapshotResponse, DepositPrice, DepositPriceCandle,
        DepositPriceCandleUpdate, DepositPriceHistoryQuery, DepositPriceHistoryResponse,
        DepositPriceKey, DepositPriceSnapshot, DepositPriceState, DepositPriceTick,
        DepositTokenCandle, LatestDepositPrice, LineData, OrderbookPriceCandle,
        OrderbookPriceHistoryQuery, OrderbookPriceHistoryResponse, PriceHistoryDecimals,
        PriceHistoryState,
    };
    pub use crate::domain::trade::Trade;

    // Domain types — metrics
    pub use crate::domain::metrics::{
        CategoriesMetrics, CategoryMetricsQuery, CategoryVolumeMetrics, DepositTokenVolumeHistory,
        DepositTokenVolumeHistoryPoint, DepositTokenVolumeHistoryPointToken,
        DepositTokenVolumeHistoryQuery, DepositTokenVolumeHistoryToken, DepositTokenVolumeMetrics,
        DepositTokensMetrics, HistoryPoint, Leaderboard, LeaderboardEntry, MarketDetailMetrics,
        MarketMetricsQuery, MarketOrderbookVolumeMetrics, MarketVolumeMetrics, MarketsMetrics,
        MarketsMetricsQuery, MetricsHistory, MetricsHistoryQuery, OpenInterestHistory,
        OpenInterestHistoryDepositAsset, OpenInterestHistoryPoint,
        OpenInterestHistoryPointDepositAsset, OpenInterestHistoryQuery, OrderbookMetricsQuery,
        OrderbookVolumeMetrics, OutcomeVolumeMetrics, PlatformMetrics, UniqueTradersHistory,
        UniqueTradersHistoryPoint, UniqueTradersHistoryQuery, UniqueTradersHistoryScope,
    };

    // Domain types — faucet
    pub use crate::domain::faucet::{FaucetRequest, FaucetResponse, FaucetToken};

    // Domain types — market wire (raw market and deposit-asset responses)
    pub use crate::domain::market::wire::{
        ConditionalTokenResponse, DepositAssetResponse, DepositMintsResponse, MarketResponse,
    };

    // Errors
    pub use crate::error::SdkError;

    // Environment
    pub use crate::env::LightconeEnv;

    // Auth + User types
    pub use crate::auth::{
        AuthCredentials, AuthMethod, ChainType, GoogleAccountData, PrivyEmbeddedWallet,
        SessionResponse, User, UserIdentity, UserPrivyData, XAccountData,
    };

    // Program — order envelopes, trait, payload
    #[cfg(feature = "trigger_orders")]
    pub use crate::program::TriggerOrderEnvelope;
    pub use crate::program::{
        generate_cancel_all_salt, LimitOrderEnvelope, OrderEnvelope, OrderPayload,
    };

    // Position builders
    pub use crate::domain::position::{
        DepositBuilder, DepositToGlobalBuilder, ExtendPositionTokensBuilder,
        GlobalToMarketDepositBuilder, InitPositionTokensBuilder, MergeBuilder,
        RedeemWinningsBuilder, WithdrawBuilder, WithdrawFromGlobalBuilder,
        WithdrawFromPositionBuilder,
    };

    // Signing strategy
    pub use crate::shared::signing::{ExternalSigner, SigningStrategy};

    // Domain types — referral
    pub use crate::domain::referral::{RedeemResult, ReferralCodeInfo, ReferralStatus};

    // Domain types — notification
    pub use crate::domain::notification::{
        MarketData, MarketResolvedData, Notification, NotificationKind, OrderFilledData,
    };

    // HTTP client + sub-clients
    #[cfg(feature = "http")]
    pub use crate::client::{
        AuthClient, FavoriteMarketUpdate, FavoriteMarkets, GlobalDepositAssetsResult,
        LightconeClient, LightconeClientBuilder, MarketsClient, MarketsResult, MetricsClient,
        NotificationsClient, OrderbooksClient, OrdersClient, PositionsClient,
        PriceHistorySubClient, ReferralsClient, RpcClient, TradesClient,
    };
    #[cfg(feature = "http")]
    pub use crate::http::retry::{RetryConfig, RetryPolicy};
    #[cfg(feature = "http")]
    pub use crate::http::{CookieSession, LightconeHttp};
    #[cfg(feature = "http")]
    pub use crate::rpc_failover::ActiveRpc;

    // WebSocket types
    pub use crate::ws::{Kind, MessageIn, MessageOut, SubscribeParams, UnsubscribeParams, WsEvent};

    // State containers
    pub use crate::domain::orderbook::state::{ApplyResult, OrderbookState, RefreshReason};
    pub use crate::domain::trade::TradeHistory;
}
