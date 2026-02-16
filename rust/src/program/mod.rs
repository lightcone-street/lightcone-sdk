//! On-chain program interaction module for Lightcone.
//!
//! This module provides the client and utilities for interacting with
//! the Lightcone smart contract on Solana.

pub mod accounts;
pub mod builder;
#[cfg(feature = "client")]
pub mod client;
pub mod constants;
pub mod error;
pub mod instructions;
pub mod orders;
pub mod pda;
pub mod types;
pub mod utils;

// Re-export commonly used items
pub use accounts::{Exchange, GlobalDepositToken, Market, Orderbook, OrderStatus, Position, UserNonce};
pub use builder::OrderBuilder;
#[cfg(feature = "client")]
pub use client::LightconePinocchioClient;
pub use constants::*;
pub use error::{SdkError, SdkResult};
pub use instructions::*;
pub use orders::{
    calculate_taker_fill, cancel_all_message, cancel_order_message, derive_condition_id,
    is_order_expired, orders_can_cross, Order, SignedOrder,
};
#[cfg(feature = "client")]
pub use orders::{sign_cancel_all, sign_cancel_order};
pub use pda::*;
pub use types::*;
pub use utils::*;
