//! # Lightcone Pinocchio Rust SDK
//!
//! A Rust SDK for interacting with the Lightcone protocol.
//!
//! ## Modules
//!
//! This SDK provides three main modules:
//! - [`program`]: On-chain program interaction (smart contract)
//! - [`api`]: REST API client for market data, orders, and positions
//! - [`websocket`]: Real-time data streaming via WebSocket
//!
//! Plus a shared module:
//! - [`shared`]: Shared utilities, types, and constants
//!
//! ## Quick Start - REST API
//!
//! ```rust,ignore
//! use lightcone_sdk::api::LightconeApiClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create API client
//!     let api = LightconeApiClient::new("https://api.lightcone.xyz");
//!
//!     // Get all markets (first page)
//!     let markets = api.get_markets(None, None).await?;
//!     println!("Has more: {}", markets.has_more);
//!
//!     // Get orderbook
//!     let orderbook = api.get_orderbook("orderbook_id", Some(10)).await?;
//!     println!("Best bid: {:?}", orderbook.best_bid);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Quick Start - On-Chain Program
//!
//! ```rust,ignore
//! use lightcone_sdk::program::LightconePinocchioClient;
//! use lightcone_sdk::shared::types::*;
//! use solana_pubkey::Pubkey;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create on-chain client
//!     let client = LightconePinocchioClient::new("https://api.devnet.solana.com");
//!
//!     // Fetch exchange state
//!     let exchange = client.get_exchange().await.unwrap();
//!     println!("Market count: {}", exchange.market_count);
//!
//!     // Create an order
//!     let order = client.create_bid_order(BidOrderParams {
//!         nonce: 1,
//!         maker: Pubkey::new_unique(),
//!         market: Pubkey::new_unique(),
//!         base_mint: Pubkey::new_unique(),
//!         quote_mint: Pubkey::new_unique(),
//!         amount_in: 1000,
//!         amount_out: 500,
//!         expiration: 0,
//!     });
//! }
//! ```

// ============================================================================
// MODULES
// ============================================================================

/// On-chain program interaction module.
/// Contains the client and utilities for interacting with the Lightcone smart contract.
pub mod program;

/// Shared utilities, types, and constants.
/// Used across all SDK modules.
pub mod shared;

/// Network URL constants (API and WebSocket endpoints).
pub mod network;

/// Authentication module for obtaining JWT tokens.
/// Types are always available; `authenticate()` requires the `auth` feature.
pub mod auth;

/// REST API client module for market data, orders, and positions.
pub mod api;

/// WebSocket client module for real-time data streaming.
#[cfg(feature = "websocket")]
pub mod websocket;

// ============================================================================
// PRELUDE
// ============================================================================

/// Prelude module for convenient imports.
///
/// ```rust,ignore
/// use lightcone_sdk::prelude::*;
/// ```
pub mod prelude {
    // Program module exports
    pub use crate::program::{
        // Order utilities
        calculate_taker_fill,
        derive_condition_id,
        is_order_expired,
        orders_can_cross,
        Order,
        // Order builder
        OrderBuilder,
        OrderSide,
        OrderStatus,
        Orderbook,
        OutcomeMetadata,
        Position,
        RedeemWinningsParams,
        // Errors
        SdkError,
        SdkResult,
        SetAuthorityParams,
        SettleMarketParams,
        SignedCancelAll,
        SignedCancelOrder,
        SignedOrder,
        UserNonce,
        WhitelistDepositTokenParams,
        WithdrawFromPositionParams,
        ASSOCIATED_TOKEN_PROGRAM_ID,
        // Constants (moved from shared)
        PROGRAM_ID,
        TOKEN_2022_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    };

    // Client (conditionally exported)
    #[cfg(feature = "native-client")]
    pub use crate::program::LightconePinocchioClient;

    // API module exports
    pub use crate::api::{
        ApiError,
        ApiResult,
        CancelAllResponse,
        CancelResponse,
        ConditionalToken,
        DecimalsResponse,
        DepositAsset,
        LightconeApiClient,
        LightconeApiClientBuilder,
        Market as ApiMarket,
        MarketInfoResponse,
        MarketSearchResult,
        // Common types
        MarketsResponse,
        OrderResponse,
        OrderbookResponse,
        OutcomeBalance,
        Position as ApiPosition,
        PositionsResponse,
        PriceHistoryParams,
        PriceHistoryResponse,
        PriceLevel,
        SearchOrderbook,
        Trade,
        TradesParams,
        TradesResponse,
    };

    // Network constants
    pub use crate::network::{DEFAULT_API_URL, DEFAULT_WS_URL};

    // Auth module exports
    #[cfg(feature = "auth")]
    pub use crate::auth::{authenticate, authenticate_with_transaction};

    // Shared utilities (used by both API and WebSocket)
    pub use crate::shared::{
        derive_orderbook_id, format_decimal, parse_decimal, scale_price_size, OrderbookDecimals,
        Resolution, ScaledAmounts, ScalingError,
    };

    // WebSocket module exports
    #[cfg(feature = "websocket")]
    pub use crate::websocket::{
        BookUpdateData, ConnectionState, LightconeWebSocketClient, LocalOrderbook, MarketEventData,
        PriceHistory, PriceHistoryData, TradeData, UserEventData, UserState, WebSocketConfig,
        WebSocketError, WsEvent, WsResult,
    };
}
