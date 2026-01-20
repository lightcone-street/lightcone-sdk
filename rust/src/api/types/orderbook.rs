//! Orderbook-related types for the Lightcone REST API.

use serde::{Deserialize, Serialize};

/// Price level in the orderbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Price (scaled by 1e6)
    pub price: i64,
    /// Total size at this price level
    pub size: u64,
    /// Number of orders at this level
    pub orders: u32,
}

/// Response for GET /api/orderbook/{orderbook_id}.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookResponse {
    /// Market pubkey
    pub market_pubkey: String,
    /// Orderbook identifier
    pub orderbook_id: String,
    /// Bid levels (buy orders), sorted by price descending
    pub bids: Vec<PriceLevel>,
    /// Ask levels (sell orders), sorted by price ascending
    pub asks: Vec<PriceLevel>,
    /// Best bid price
    pub best_bid: Option<i64>,
    /// Best ask price
    pub best_ask: Option<i64>,
    /// Spread (best_ask - best_bid)
    pub spread: Option<i64>,
    /// Tick size for this orderbook
    pub tick_size: i64,
}
