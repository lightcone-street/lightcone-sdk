//! REST API client module for Lightcone.
//!
//! This module provides a type-safe HTTP client for interacting with
//! the Lightcone REST API for market data, orderbooks, orders, and more.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use lightcone_pinocchio_sdk::api::LightconeApiClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client with default settings
//!     let client = LightconeApiClient::new("https://api.lightcone.io");
//!
//!     // Get all markets
//!     let markets = client.get_markets().await?;
//!     println!("Found {} markets", markets.total);
//!
//!     // Get a specific market
//!     let market = client.get_market("market_pubkey").await?;
//!     println!("Market: {}", market.market.market_name);
//!
//!     // Get orderbook
//!     let orderbook = client.get_orderbook("orderbook_id", Some(10)).await?;
//!     println!("Best bid: {:?}, Best ask: {:?}", orderbook.best_bid, orderbook.best_ask);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Client Configuration
//!
//! Use the builder pattern for custom configuration:
//!
//! ```rust,ignore
//! use lightcone_pinocchio_sdk::api::LightconeApiClient;
//! use std::time::Duration;
//!
//! let client = LightconeApiClient::builder("https://api.lightcone.io")
//!     .timeout(Duration::from_secs(60))
//!     .header("X-Custom-Header", "value")
//!     .build()?;
//! ```
//!
//! # Error Handling
//!
//! All methods return `ApiResult<T>` which is an alias for `Result<T, ApiError>`.
//! The [`ApiError`] enum covers all possible error cases:
//!
//! ```rust,ignore
//! use lightcone_pinocchio_sdk::api::{LightconeApiClient, ApiError};
//!
//! match client.get_market("invalid_pubkey").await {
//!     Ok(market) => println!("Found market"),
//!     Err(ApiError::NotFound(msg)) => println!("Market not found: {}", msg),
//!     Err(ApiError::BadRequest(msg)) => println!("Invalid request: {}", msg),
//!     Err(e) => println!("Other error: {}", e),
//! }
//! ```
//!
//! # Order Submission
//!
//! Orders must be pre-signed with Ed25519. Use the program module to create
//! and sign orders, then submit via the API:
//!
//! ```rust,ignore
//! use lightcone_pinocchio_sdk::api::{LightconeApiClient, SubmitOrderRequest};
//!
//! let request = SubmitOrderRequest {
//!     maker: "maker_pubkey".to_string(),
//!     nonce: 1,
//!     market_pubkey: "market_pubkey".to_string(),
//!     base_token: "base_token".to_string(),
//!     quote_token: "quote_token".to_string(),
//!     side: 0, // BID
//!     maker_amount: 1000000,
//!     taker_amount: 500000,
//!     expiration: 0, // No expiration
//!     signature: "hex_encoded_signature".to_string(),
//!     orderbook_id: "orderbook_id".to_string(),
//! };
//!
//! let response = client.submit_order(request).await?;
//! println!("Order hash: {}", response.order_hash);
//! ```

pub mod client;
pub mod error;
pub mod types;

// Re-export main types for convenience
pub use client::{LightconeApiClient, LightconeApiClientBuilder};
pub use error::{ApiError, ApiResult, ErrorResponse};
pub use types::*;
