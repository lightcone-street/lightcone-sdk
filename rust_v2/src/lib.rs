//! # Lightcone SDK v2
//!
//! A unified Rust SDK for the Lightcone protocol supporting both native and WASM targets.
//!
//! ## Architecture
//!
//! The SDK is organized in layers:
//!
//! 1. **Core** — Types, program logic, domain models (always available, WASM-safe)
//! 2. **Auth** — Message generation + platform-dependent signing
//! 3. **HTTP API** — `LightconeHttp` with per-endpoint retry policies
//! 4. **WebSocket** — Compile-time dispatch: `tokio-tungstenite` (native) / `web-sys` (WASM)
//! 5. **High-Level Client** — `LightconeClient` with nested sub-clients and caching
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use lightcone_sdk::prelude::*;
//!
//! let client = LightconeClient::builder()
//!     .base_url("https://tapi.lightcone.xyz")
//!     .build()?;
//!
//! let markets = client.markets().featured().await?;
//! let orderbook = client.orderbooks().get("orderbook_id", Some(10)).await?;
//! ```

// ── Layer 1: Core ────────────────────────────────────────────────────────────

/// Shared newtypes used across all domains.
pub mod shared;

/// Domain modules (vertical slices): types, wire types, conversions, state.
pub mod domain;

/// On-chain program interaction: instructions, orders, PDAs, accounts.
pub mod program;

/// Unified SDK error types.
pub mod error;

/// Network URL constants.
pub mod network;

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
    pub use crate::shared::{OrderBookId, PubkeyStr, Resolution, Side};

    // Domain types — market (includes outcome + tokens)
    pub use crate::domain::market::outcome::Outcome;
    pub use crate::domain::market::tokens::{
        ConditionalToken, DepositAsset, Token, TokenMetadata, ValidatedTokens,
    };
    pub use crate::domain::market::{Market, Status};

    // Domain types — orderbook
    pub use crate::domain::orderbook::{OrderBookPair, OrderBookValidationError, OutcomeImpact};

    // Domain types — order
    pub use crate::domain::order::{Order, OrderStatus, OrderType, UserOpenOrders};

    // Domain types — position (includes portfolio + token balances)
    pub use crate::domain::position::{
        DepositAssetMetadata, DepositTokenBalance, Portfolio, Position, PositionOutcome,
        TokenBalance, TokenBalanceComputedBase, TokenBalanceTokenType, WalletHolding,
    };

    // Domain types — trade, price history
    pub use crate::domain::price_history::{LineData, PriceHistoryState};
    pub use crate::domain::trade::Trade;

    // Errors
    pub use crate::error::SdkError;

    // Network
    pub use crate::network::{DEFAULT_API_URL, DEFAULT_WS_URL};

    // Auth + User types
    pub use crate::auth::{
        AuthCredentials, ChainType, EmbeddedWallet, LinkedAccount, LinkedAccountType, User,
    };

    // Privy RPC types
    pub use crate::privy::{
        ExportWalletRequest, ExportWalletResponse, OrderForSigning, SignAndSendOrderRequest,
        SignAndSendTxRequest, SignAndSendTxResponse,
    };

    // HTTP client + sub-clients
    #[cfg(feature = "http")]
    pub use crate::client::{
        AdminClient, AuthClient, LightconeClient, LightconeClientBuilder, MarketsClient,
        MarketsResult, OrderbooksClient, OrdersClient, PositionsClient, PriceHistorySubClient,
        TradesClient,
    };
    #[cfg(feature = "http")]
    pub use crate::http::retry::{RetryConfig, RetryPolicy};

    // WebSocket types
    pub use crate::ws::{Kind, MessageIn, MessageOut, SubscribeParams, UnsubscribeParams, WsEvent};

    // State containers
    pub use crate::domain::orderbook::state::OrderbookSnapshot;
    pub use crate::domain::trade::TradeHistory;
}
