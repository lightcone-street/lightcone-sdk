//! State management for WebSocket subscriptions.
//!
//! This module provides local state management for various subscription types:
//! - `orderbook`: Local orderbook state with delta application
//! - `user`: User orders and balances state
//! - `price`: Price history and candles state

pub mod orderbook;
pub mod price;
pub mod user;

pub use orderbook::LocalOrderbook;
pub use price::PriceHistory;
pub use user::UserState;
