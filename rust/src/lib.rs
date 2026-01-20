//! # Lightcone Pinocchio Rust SDK
//!
//! A production-quality Rust SDK for interacting with the Lightcone prediction markets platform.
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
//! use lightcone_pinocchio_sdk::api::LightconeApiClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create API client
//!     let api = LightconeApiClient::new("https://api.lightcone.io");
//!
//!     // Get all markets
//!     let markets = api.get_markets().await?;
//!     println!("Found {} markets", markets.total);
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
//! use lightcone_pinocchio_sdk::program::LightconePinocchioClient;
//! use lightcone_pinocchio_sdk::shared::types::*;
//! use solana_sdk::pubkey::Pubkey;
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
//!         maker_amount: 1000,
//!         taker_amount: 500,
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

/// REST API client module for market data, orders, and positions.
pub mod api;

/// WebSocket client module for real-time data streaming.
pub mod websocket;

// ============================================================================
// PRELUDE
// ============================================================================

/// Prelude module for convenient imports.
///
/// ```rust,ignore
/// use lightcone_pinocchio_sdk::prelude::*;
/// ```
pub mod prelude {
    // Program module exports
    pub use crate::program::{
        // Account types
        Exchange, Market, OrderStatus, Position, UserNonce,
        // Client
        LightconePinocchioClient,
        // Errors
        SdkError, SdkResult,
        // Ed25519 verification
        create_batch_ed25519_verify_instruction, create_cross_ref_ed25519_instructions,
        create_ed25519_verify_instruction, create_order_verify_instruction, Ed25519VerifyParams,
        // Order utilities
        calculate_taker_fill, derive_condition_id, is_order_expired, orders_can_cross,
        CompactOrder, FullOrder,
        // PDA functions
        get_exchange_pda, get_market_pda, get_vault_pda, get_mint_authority_pda,
        get_conditional_mint_pda, get_order_status_pda, get_user_nonce_pda, get_position_pda,
        get_all_conditional_mint_pdas,
        // Types (moved from shared)
        MarketStatus, OrderSide, OutcomeMetadata,
        BidOrderParams, AskOrderParams, CreateMarketParams, MatchOrdersMultiParams,
        MintCompleteSetParams, MergeCompleteSetParams, SettleMarketParams, RedeemWinningsParams,
        AddDepositMintParams, ActivateMarketParams, WithdrawFromPositionParams,
        // Constants (moved from shared)
        PROGRAM_ID, TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID,
    };

    // API module exports
    pub use crate::api::{
        LightconeApiClient, LightconeApiClientBuilder, ApiError, ApiResult,
        // Common types
        MarketsResponse, MarketInfoResponse, Market as ApiMarket, DepositAsset, ConditionalToken,
        OrderbookResponse, PriceLevel,
        SubmitOrderRequest, OrderResponse, CancelResponse, CancelAllResponse,
        PositionsResponse, Position as ApiPosition, OutcomeBalance,
        PriceHistoryParams, PriceHistoryResponse,
        TradesParams, TradesResponse, Trade,
    };

    // Shared price types (used by both API and WebSocket)
    pub use crate::shared::{decimal_to_scaled, scaled_to_decimal, Resolution, PRICE_SCALE};

    // WebSocket module exports
    pub use crate::websocket::{
        LightconeWebSocketClient, WebSocketConfig, WebSocketError, WsResult,
        ConnectionState, WsEvent,
        BookUpdateData, TradeData, UserEventData, PriceHistoryData, MarketEventData,
        LocalOrderbook, UserState, PriceHistory,
    };
}
