//! Orderbook-related types for the Lightcone REST API.

use serde::{Deserialize, Serialize};

/// Price level in the orderbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Price as decimal string (e.g., "0.500000")
    pub price: String,
    /// Total size at this price level as decimal string
    pub size: String,
    /// Number of orders at this level
    pub orders: i32,
}

/// Decimal precision info for an orderbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookPrecision {
    /// Price decimal places
    pub price: u8,
    /// Size (base token) decimal places
    pub size: u8,
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
    /// Best bid price as decimal string
    pub best_bid: Option<String>,
    /// Best ask price as decimal string
    pub best_ask: Option<String>,
    /// Spread (best_ask - best_bid) as decimal string
    pub spread: Option<String>,
    /// Tick size for this orderbook as decimal string
    pub tick_size: String,
    /// Decimal precision for price and size
    #[serde(default)]
    pub decimals: Option<OrderbookPrecision>,
}
