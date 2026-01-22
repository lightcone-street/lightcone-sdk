//! Admin-related types for the Lightcone REST API.

use serde::{Deserialize, Serialize};

/// Response for GET /api/admin/test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminResponse {
    /// Status (usually "success")
    pub status: String,
    /// Human-readable message
    pub message: String,
}

/// Request for POST /api/admin/create-orderbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderbookRequest {
    /// Market address (Base58)
    pub market_pubkey: String,
    /// Base conditional token (Base58)
    pub base_token: String,
    /// Quote conditional token (Base58)
    pub quote_token: String,
    /// Price granularity (default: 1000)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick_size: Option<u32>,
}

impl CreateOrderbookRequest {
    /// Create a new request with required fields.
    pub fn new(
        market_pubkey: impl Into<String>,
        base_token: impl Into<String>,
        quote_token: impl Into<String>,
    ) -> Self {
        Self {
            market_pubkey: market_pubkey.into(),
            base_token: base_token.into(),
            quote_token: quote_token.into(),
            tick_size: None,
        }
    }

    /// Set custom tick size.
    pub fn with_tick_size(mut self, tick_size: u32) -> Self {
        self.tick_size = Some(tick_size);
        self
    }
}

/// Response for POST /api/admin/create-orderbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderbookResponse {
    /// Status (usually "success")
    pub status: String,
    /// Created orderbook ID
    pub orderbook_id: String,
    /// Market pubkey
    pub market_pubkey: String,
    /// Base token address
    pub base_token: String,
    /// Quote token address
    pub quote_token: String,
    /// Tick size
    pub tick_size: u32,
    /// Human-readable message
    pub message: String,
}
