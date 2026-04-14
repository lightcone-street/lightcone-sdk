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
        ConditionalToken, DepositAsset, Token, TokenMetadata, ValidatedTokens,
    };
    pub use crate::domain::market::{Market, Status};

    // Domain types — orderbook
    pub use crate::domain::orderbook::{OrderBookPair, OrderBookValidationError, OutcomeImpact};

    // Domain types — order
    pub use crate::domain::order::{
        CancelAllBody, CancelAllSuccess, CancelBody, CancelSuccess, CancelTriggerBody,
        CancelTriggerSuccess, ConditionalBalance, FillInfo, GlobalDepositBalance,
        GlobalDepositUpdate, Order, OrderEvent, OrderStatus, OrderType, SubmitOrderResponse,
        TriggerOrder, TriggerOrderResponse, TriggerOrderUpdate, UserOpenOrders, UserOrdersResponse,
        UserSnapshotBalance, UserSnapshotOrder, UserTriggerOrders,
    };

    // Domain types — position (includes portfolio + token balances)
    pub use crate::domain::position::{
        DepositAssetMetadata, DepositTokenBalance, Portfolio, Position, PositionOutcome,
        TokenBalance, TokenBalanceComputedBase, TokenBalanceTokenType, WalletHolding,
    };

    // Domain types — trade, price history
    pub use crate::domain::price_history::{
        DepositPrice, DepositPriceCandle, DepositPriceCandleUpdate, DepositPriceHistoryQuery,
        DepositPriceHistoryResponse, DepositPriceKey, DepositPriceSnapshot, DepositPriceState,
        DepositPriceTick, DepositTokenCandle, LatestDepositPrice, LineData, OrderbookPriceCandle,
        OrderbookPriceHistoryQuery, OrderbookPriceHistoryResponse, PriceHistoryDecimals,
        PriceHistoryState,
    };
    pub use crate::domain::trade::Trade;

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
        AdminClient, AuthClient, LightconeClient, LightconeClientBuilder, MarketsClient,
        MarketsResult, NotificationsClient, OrderbooksClient, OrdersClient, PositionsClient,
        PriceHistorySubClient, ReferralsClient, RpcClient, TradesClient,
    };
    #[cfg(feature = "http")]
    pub use crate::http::retry::{RetryConfig, RetryPolicy};

    // WebSocket types
    pub use crate::ws::{Kind, MessageIn, MessageOut, SubscribeParams, UnsubscribeParams, WsEvent};

    // State containers
    pub use crate::domain::orderbook::state::{ApplyResult, OrderbookState};
    pub use crate::domain::trade::TradeHistory;
}
