//! Shared utilities and types used across API and WebSocket modules.

pub mod price;
pub mod types;

// Re-export commonly used items
pub use price::{decimal_to_scaled, scaled_to_decimal, PRICE_SCALE};
pub use types::*;
