//! Domain modules organized as vertical slices.
//!
//! Each sub-module contains:
//! - `mod.rs` — Rich domain types (validated, business-logic-ready)
//! - `wire.rs` — Raw serde structs matching backend responses
//! - `convert.rs` — `TryFrom`/`From` conversions with validation
//! - `state.rs` — State containers with update methods (for WS-driven data)
//! - `client.rs` — Sub-client with HTTP methods and caching

pub mod admin;
pub mod market;
pub mod order;
pub mod orderbook;
pub mod position;
pub mod price_history;
pub mod trade;
