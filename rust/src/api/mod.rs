//! REST API client module for Lightcone.
//!
//! This module provides a type-safe HTTP client for interacting with
//! the Lightcone REST API for market data, orderbooks, orders, and more.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use lightcone_sdk::api::LightconeApiClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client with default settings
//!     let client = LightconeApiClient::new("https://api.lightcone.xyz");
//!
//!     // Get all markets (first page)
//!     let markets = client.get_markets(None, None).await?;
//!     println!("Has more: {}", markets.has_more);
//!
//!     // Get a specific market
//!     let market = client.get_market("market_pubkey").await?;
//!     println!("Market: {:?}", market.market.market_name);
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
//! use lightcone_sdk::api::LightconeApiClient;
//! use std::time::Duration;
//!
//! let client = LightconeApiClient::builder("https://api.lightcone.xyz")
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
//! use lightcone_sdk::api::{LightconeApiClient, ApiError};
//!
//! match client.get_market("invalid_pubkey").await {
//!     Ok(market) => println!("Found market"),
//!     Err(ApiError::NotFound(resp)) => println!("Market not found: {}", resp),
//!     Err(ApiError::BadRequest(resp)) => println!("Invalid request: {}", resp),
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
//! use lightcone_sdk::api::LightconeApiClient;
//! use lightcone_sdk::program::{OrderBuilder, SignedCancelOrder};
//!
//! let order = OrderBuilder::new()
//!     .maker(keypair.pubkey())
//!     .market(market)
//!     .base_mint(base_mint)
//!     .quote_mint(quote_mint)
//!     .bid()
//!     .price("0.55")
//!     .size("1.0")
//!     .apply_scaling(&decimals)?
//!     .build_and_sign(&keypair);
//!
//! let response = client.submit_full_order(&order, orderbook_id).await?;
//!
//! // Cancel the order
//! let cancel = SignedCancelOrder::new_signed(&response.order_hash, keypair.pubkey(), &keypair);
//! client.cancel_signed_order(&cancel).await?;
//! ```

pub mod client;
pub mod error;
pub mod types;

// Re-export main types for convenience
pub use client::{LightconeApiClient, LightconeApiClientBuilder, RetryConfig};
pub use crate::network::DEFAULT_API_URL;
pub use error::{ApiError, ApiResult, ErrorResponse};
pub use types::*;
