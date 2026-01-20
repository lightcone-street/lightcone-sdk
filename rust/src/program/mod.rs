//! On-chain program interaction module for Lightcone.
//!
//! This module provides the client and utilities for interacting with
//! the Lightcone smart contract on Solana.

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

// Re-export commonly used items
pub use accounts::{Exchange, Market, OrderStatus, Position, UserNonce};
pub use client::LightconePinocchioClient;
pub use constants::*;
pub use ed25519::{
    create_batch_ed25519_verify_instruction, create_cross_ref_ed25519_instructions,
    create_ed25519_verify_instruction, create_order_verify_instruction, Ed25519VerifyParams,
};
pub use error::{SdkError, SdkResult};
pub use instructions::*;
pub use orders::{
    calculate_taker_fill, derive_condition_id, is_order_expired, orders_can_cross, CompactOrder,
    FullOrder,
};
pub use pda::*;
pub use types::*;
pub use utils::*;
