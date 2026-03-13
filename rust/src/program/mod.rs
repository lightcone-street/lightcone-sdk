#![doc = include_str!("README.md")]

pub mod accounts;
pub mod envelope;
#[cfg(feature = "native")]
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
pub use envelope::{LimitOrderEnvelope, OrderEnvelope, TriggerOrderEnvelope};
#[cfg(feature = "native")]
pub use client::LightconePinocchioClient;
pub use constants::*;
pub use error::{SdkError, SdkResult};
pub use instructions::*;
pub use orders::{
    calculate_taker_fill, cancel_all_message, cancel_order_message,
    derive_condition_id, generate_cancel_all_salt, is_order_expired,
    orders_can_cross, Order, OrderPayload,
};
pub use pda::*;
pub use types::*;
pub use utils::*;
