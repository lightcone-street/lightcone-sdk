//! Shared type definitions for the Lightcone SDK.
//!
//! This module contains types that are used by both the REST API and WebSocket modules.

use serde::{Deserialize, Serialize};

// ============================================================================
// SubmitOrderRequest (shared between program and API modules)
// ============================================================================

/// Request for submitting an order via REST API.
///
/// This type bridges the program module (on-chain order signing) with the API module
/// (REST order submission). Use `FullOrder::to_submit_request()` to convert a signed
/// order to this format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitOrderRequest {
    /// Order creator's pubkey (Base58)
    pub maker: String,
    /// User's nonce for uniqueness (u32 range)
    pub nonce: u32,
    /// Market address (Base58)
    pub market_pubkey: String,
    /// Token being bought/sold (Base58)
    pub base_token: String,
    /// Token used for payment (Base58)
    pub quote_token: String,
    /// Order side (0=BID, 1=ASK)
    pub side: u32,
    /// Amount maker gives
    pub maker_amount: u64,
    /// Amount maker wants to receive
    pub taker_amount: u64,
    /// Unix timestamp, 0=no expiration
    #[serde(default)]
    pub expiration: i64,
    /// Ed25519 signature (hex, 128 chars)
    pub signature: String,
    /// Target orderbook
    pub orderbook_id: String,
}

// ============================================================================
// Resolution Enum (shared between API and WebSocket)
// ============================================================================

/// Price history candle resolution.
///
/// Used by both REST API and WebSocket for price history queries.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Resolution {
    /// 1 minute candles
    #[default]
    #[serde(rename = "1m")]
    OneMinute,
    /// 5 minute candles
    #[serde(rename = "5m")]
    FiveMinutes,
    /// 15 minute candles
    #[serde(rename = "15m")]
    FifteenMinutes,
    /// 1 hour candles
    #[serde(rename = "1h")]
    OneHour,
    /// 4 hour candles
    #[serde(rename = "4h")]
    FourHours,
    /// 1 day candles
    #[serde(rename = "1d")]
    OneDay,
}

impl Resolution {
    /// Get the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneMinute => "1m",
            Self::FiveMinutes => "5m",
            Self::FifteenMinutes => "15m",
            Self::OneHour => "1h",
            Self::FourHours => "4h",
            Self::OneDay => "1d",
        }
    }
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
