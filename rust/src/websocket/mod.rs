//! WebSocket client module for Lightcone.
//!
//! This module provides real-time data streaming functionality for live orderbook
//! updates, trade notifications, user events, price history, and market events.
//!
//! # Features
//!
//! - **Real-time orderbook updates**: Subscribe to delta-based orderbook updates with
//!   automatic local state management
//! - **Trade stream**: Receive trade execution notifications
//! - **User events**: Track your orders and balances in real-time
//! - **Price history**: OHLCV candles at various resolutions
//! - **Market events**: Market status changes and new orderbook creation
//! - **Auto-reconnect**: Configurable reconnection with exponential backoff
//! - **State management**: Automatic local state maintenance for subscriptions
//!
//! # Example
//!
//! ```ignore
//! use lightcone_sdk::websocket::*;
//! use futures_util::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), WebSocketError> {
//!     // Connect with default config
//!     let mut client = LightconeWebSocketClient::connect("ws://api.lightcone.xyz:8081/ws").await?;
//!
//!     // Subscribe to orderbook
//!     client.subscribe_book_updates(vec!["market1:ob1".to_string()]).await?;
//!
//!     // Subscribe to user stream
//!     client.subscribe_user("user_public_key".to_string()).await?;
//!
//!     // Iterate over events using Stream
//!     while let Some(event) = client.next().await {
//!         match event {
//!             WsEvent::Connected => {
//!                 println!("Connected to WebSocket server");
//!             }
//!             WsEvent::BookUpdate { orderbook_id, is_snapshot } => {
//!                 // State is automatically maintained
//!                 if let Some(book) = client.get_orderbook(&orderbook_id) {
//!                     println!("Best bid: {:?}, Best ask: {:?}",
//!                         book.best_bid(), book.best_ask());
//!                 }
//!             }
//!             WsEvent::Trade { orderbook_id, trade } => {
//!                 println!("Trade: {} @ {} size {}",
//!                     orderbook_id, trade.price, trade.size);
//!             }
//!             WsEvent::UserUpdate { event_type, .. } => {
//!                 if let Some(state) = client.get_user_state("user_public_key") {
//!                     println!("Open orders: {}", state.orders.len());
//!                 }
//!             }
//!             WsEvent::ResyncRequired { orderbook_id } => {
//!                 println!("Resync required for {}", orderbook_id);
//!                 // Client auto-resubscribes by default
//!             }
//!             WsEvent::Disconnected { reason } => {
//!                 println!("Disconnected: {}", reason);
//!                 // Client auto-reconnects by default
//!             }
//!             WsEvent::Error { error } => {
//!                 println!("Error: {:?}", error);
//!             }
//!             _ => {}
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! # Configuration
//!
//! The client can be configured with custom settings:
//!
//! ```ignore
//! use lightcone_sdk::websocket::*;
//!
//! let config = WebSocketConfig {
//!     reconnect_attempts: 5,
//!     base_delay_ms: 500,
//!     max_delay_ms: 15000,
//!     ping_interval_secs: 30,
//!     auto_reconnect: true,
//!     auto_resubscribe: true,
//! };
//!
//! let client = LightconeWebSocketClient::connect_with_config(
//!     "ws://api.lightcone.xyz:8081/ws",
//!     config,
//! ).await?;
//! ```

pub mod auth;
pub mod client;
pub mod error;
pub mod handlers;
pub mod state;
pub mod subscriptions;
pub mod types;

// Re-export main types
pub use auth::{authenticate, generate_signin_message, AuthCredentials, AUTH_API_URL};
pub use client::{ConnectionState, LightconeWebSocketClient, WebSocketConfig, DEFAULT_WS_URL};
pub use error::{WebSocketError, WsResult};
pub use state::{LocalOrderbook, PriceHistory, UserState};
pub use subscriptions::{Subscription, SubscriptionManager};
pub use types::{
    // Events
    WsEvent,
    // Book updates
    BookUpdateData,
    PriceLevel,
    // Trades
    TradeData,
    // User events
    Balance,
    BalanceEntry,
    Order,
    OrderUpdate,
    OutcomeBalance,
    UserEventData,
    // Price history
    Candle,
    PriceHistoryData,
    // Market events
    MarketEventData,
    MarketEventType,
    // Errors
    ErrorCode,
    ErrorData,
    // Helpers
    MessageType,
    PriceLevelSide,
    Side,
};

// Re-export shared utilities
pub use crate::shared::{format_decimal, parse_decimal, Resolution};
