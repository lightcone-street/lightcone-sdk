//! # Lightcone Pinocchio Rust SDK
//!
//! A production-quality Rust SDK for interacting with the Lightcone Pinocchio Program
//!
//! ## Features
//!
//! - **Account Fetching**: Fetch and deserialize all on-chain account types
//! - **Transaction Building**: Build all 14 program instructions
//! - **Order Management**: Create, sign, and verify orders with Ed25519 signatures
//! - **PDA Derivation**: Derive all program PDAs
//! - **Ed25519 Verification**: Multiple strategies for signature verification
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use lightcone_pinocchio_sdk::{LightconePinocchioClient, types::*};
//! use solana_sdk::pubkey::Pubkey;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create client
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
//!
//! ## Modules
//!
//! - [`accounts`]: On-chain account structures and deserialization
//! - [`client`]: Main SDK client with account fetchers and transaction builders
//! - [`constants`]: Program IDs, seeds, discriminators, and sizes
//! - [`ed25519`]: Ed25519 signature verification instruction helpers
//! - [`error`]: Custom error types
//! - [`instructions`]: Instruction builders for all 14 program instructions
//! - [`orders`]: Order types, serialization, hashing, and signing
//! - [`pda`]: PDA derivation functions
//! - [`types`]: Type definitions (enums and parameter structs)
//! - [`utils`]: Utility functions

pub mod accounts;
pub mod client;
pub mod constants;
pub mod ed25519;
pub mod error;
pub mod instructions;
pub mod orders;
pub mod pda;
pub mod types;
pub mod utils;

// Re-export main types at crate root for convenience
pub use client::LightconePinocchioClient;
pub use error::{SdkError, SdkResult};

/// Prelude module for convenient imports.
pub mod prelude {
    pub use crate::accounts::{Exchange, Market, OrderStatus, Position, UserNonce};
    pub use crate::client::LightconePinocchioClient;
    pub use crate::constants::*;
    pub use crate::ed25519::{
        create_batch_ed25519_verify_instruction, create_cross_ref_ed25519_instructions,
        create_ed25519_verify_instruction, create_order_verify_instruction, Ed25519VerifyParams,
    };
    pub use crate::error::{SdkError, SdkResult};
    pub use crate::instructions::*;
    pub use crate::orders::{
        calculate_taker_fill, derive_condition_id, is_order_expired, orders_can_cross,
        CompactOrder, FullOrder,
    };
    pub use crate::pda::*;
    pub use crate::types::*;
    pub use crate::utils::{
        get_conditional_token_ata, get_deposit_token_ata, validate_outcome_count,
        validate_outcome_index,
    };
}
