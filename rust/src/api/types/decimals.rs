//! Decimals response type for orderbook metadata.

use serde::Deserialize;

/// API response for orderbook decimal metadata.
#[derive(Debug, Clone, Deserialize)]
pub struct DecimalsResponse {
    pub orderbook_id: String,
    pub base_decimals: u8,
    pub quote_decimals: u8,
    pub price_decimals: u8,
}
