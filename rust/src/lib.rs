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

// ── Layer 2: Auth ────────────────────────────────────────────────────────────

/// Authentication: message generation, credentials, login/logout.
pub mod auth;

/// Privy embedded wallet RPC operations.
pub mod privy;

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
        DepositSource, OrderBookId, PubkeyStr, Resolution, Side, TimeInForce, TriggerType,
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
    pub use crate::domain::orderbook::{OrderBookPair, OrderBookValidationError, OutcomeImpact};

    // Domain types — order
    pub use crate::domain::order::{
        AnyOrder, CancelAllBody, CancelAllSuccess, CancelBody, CancelSuccess, CancelTriggerBody,
        CancelTriggerSuccess, ConditionalBalance, FillInfo, GlobalDepositBalance,
        GlobalDepositUpdate, LimitOrder, Order, OrderEvent, OrderStatus, OrderType,
        SubmitOrderResponse, TriggerOrder, TriggerOrderResponse, TriggerOrderUpdate,
        UserOpenLimitOrders, UserOrdersResponse, UserSnapshotBalance, UserSnapshotOrder,
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
        CategoriesMetrics, CategoryMetricsQuery, CategoryVolumeMetrics, DepositTokenVolumeMetrics,
        DepositTokensMetrics, HistoryPoint, Leaderboard, LeaderboardEntry, MarketDetailMetrics,
        MarketMetricsQuery, MarketOrderbookVolumeMetrics, MarketVolumeMetrics, MarketsMetrics,
        MarketsMetricsQuery, MetricsHistory, MetricsHistoryQuery, OrderbookMetricsQuery,
        OrderbookVolumeMetrics, OutcomeVolumeMetrics, PlatformMetrics,
    };

    // Domain types — faucet
    pub use crate::domain::faucet::{FaucetRequest, FaucetResponse, FaucetToken};

    // Domain types — market wire (deposit-asset responses exposed by
    // `client.markets().deposit_assets(...)`)
    pub use crate::domain::market::wire::{
        ConditionalTokenResponse, DepositAssetResponse, DepositMintsResponse,
    };

    // Domain types — admin (referral config/codes and logs)
    pub use crate::domain::admin::{
        AdminLogEvent, AdminLogEventsQuery, AdminLogEventsResponse, AdminLogMetricBreakdown,
        AdminLogMetricHistoryQuery, AdminLogMetricHistoryResponse, AdminLogMetricPoint,
        AdminLogMetricSummary, AdminLogMetricsQuery, AdminLogMetricsResponse, CodeListEntry,
        ListCodesRequest, ListCodesResponse, ReferralConfig, UpdateCodeRequest, UpdateCodeResponse,
        UpdateConfigRequest,
    };

    // Errors
    pub use crate::error::SdkError;

    // Environment
    pub use crate::env::LightconeEnv;

    // Auth + User types
    pub use crate::auth::{
        AuthCredentials, ChainType, EmbeddedWallet, LinkedAccount, LinkedAccountType, User,
    };

    // Program — order envelopes, trait, payload
    pub use crate::program::{
        generate_cancel_all_salt, LimitOrderEnvelope, OrderEnvelope, OrderPayload,
        TriggerOrderEnvelope,
    };

    // Position builders
    pub use crate::domain::position::{
        DepositBuilder, DepositToGlobalBuilder, ExtendPositionTokensBuilder,
        GlobalToMarketDepositBuilder, InitPositionTokensBuilder, MergeBuilder,
        RedeemWinningsBuilder, WithdrawBuilder, WithdrawFromGlobalBuilder,
        WithdrawFromPositionBuilder,
    };

    // Privy RPC types
    pub use crate::privy::{
        ExportWalletRequest, ExportWalletResponse, PrivyOrderEnvelope, SignAndSendOrderRequest,
        SignAndSendTxRequest, SignAndSendTxResponse,
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
        AdminClient, AuthClient, GlobalDepositAssetsResult, LightconeClient,
        LightconeClientBuilder, MarketsClient, MarketsResult, MetricsClient, NotificationsClient,
        OrderbooksClient, OrdersClient, PositionsClient, PriceHistorySubClient, ReferralsClient,
        RpcClient, TradesClient,
    };
    #[cfg(feature = "http")]
    pub use crate::http::retry::{RetryConfig, RetryPolicy};

    // WebSocket types
    pub use crate::ws::{Kind, MessageIn, MessageOut, SubscribeParams, UnsubscribeParams, WsEvent};

    // State containers
    pub use crate::domain::orderbook::state::{
        ApplyResult, IgnoreReason, OrderbookState, RefreshReason,
    };
    pub use crate::domain::trade::TradeHistory;
}
