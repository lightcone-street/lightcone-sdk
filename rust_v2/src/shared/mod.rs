//! Shared newtypes and utilities used across all domain modules.
//!
//! These types are serialization-transparent: they serialize/deserialize identically
//! to the raw format the backend sends, so they can be used directly in wire types
//! without conversion overhead.

pub mod fmt;
pub mod price;
pub mod scaling;

pub use price::{format_decimal, parse_decimal};
pub use scaling::{scale_price_size, OrderbookDecimals, ScaledAmounts, ScalingError};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

// ─── OrderBookId ─────────────────────────────────────────────────────────────

/// Newtype for orderbook identifiers (e.g. `"7BgBvyjr_EPjFWdd5"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OrderBookId(String);

impl OrderBookId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OrderBookId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for OrderBookId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for OrderBookId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl FromStr for OrderBookId {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(OrderBookId(s.to_string()))
    }
}

impl Serialize for OrderBookId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for OrderBookId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(OrderBookId(s))
    }
}

// ─── PubkeyStr ───────────────────────────────────────────────────────────────

/// A Solana public key stored as a base58 string.
///
/// Serializes transparently as a JSON string. Can be used as a HashMap key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PubkeyStr(String);

impl PubkeyStr {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_pubkey(&self) -> Result<solana_pubkey::Pubkey, String> {
        solana_pubkey::Pubkey::from_str(&self.0).map_err(|e| e.to_string())
    }

    pub fn from_pubkey(pk: solana_pubkey::Pubkey) -> Self {
        Self(pk.to_string())
    }
}

impl Default for PubkeyStr {
    fn default() -> Self {
        Self(solana_pubkey::Pubkey::default().to_string())
    }
}

impl std::fmt::Display for PubkeyStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PubkeyStr {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for PubkeyStr {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<solana_pubkey::Pubkey> for PubkeyStr {
    fn from(pk: solana_pubkey::Pubkey) -> Self {
        Self(pk.to_string())
    }
}

impl Serialize for PubkeyStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for PubkeyStr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(PubkeyStr(s))
    }
}

// ─── Side ────────────────────────────────────────────────────────────────────

/// Order side: Bid (buy) or Ask (sell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Bid,
    Ask,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Side::Bid => write!(f, "Buy"),
            Side::Ask => write!(f, "Sell"),
        }
    }
}

// ─── Resolution ──────────────────────────────────────────────────────────────

/// Price history candle resolution.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resolution {
    #[default]
    #[serde(rename = "1m")]
    Minute1,
    #[serde(rename = "5m")]
    Minute5,
    #[serde(rename = "15m")]
    Minute15,
    #[serde(rename = "1h")]
    Hour1,
    #[serde(rename = "4h")]
    Hour4,
    #[serde(rename = "1d")]
    Day1,
}

impl Resolution {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minute1 => "1m",
            Self::Minute5 => "5m",
            Self::Minute15 => "15m",
            Self::Hour1 => "1h",
            Self::Hour4 => "4h",
            Self::Day1 => "1d",
        }
    }

    /// Duration of one candle in seconds.
    pub fn seconds(&self) -> u64 {
        match self {
            Self::Minute1 => 60,
            Self::Minute5 => 300,
            Self::Minute15 => 900,
            Self::Hour1 => 3600,
            Self::Hour4 => 21600,
            Self::Day1 => 86400,
        }
    }
}

impl std::fmt::Display for Resolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ─── Utilities ───────────────────────────────────────────────────────────────

/// Derive orderbook ID from base and quote token pubkeys.
///
/// Format: `{base_token[0:8]}_{quote_token[0:8]}`
pub fn derive_orderbook_id(base_token: &str, quote_token: &str) -> OrderBookId {
    let base_prefix = &base_token[..8.min(base_token.len())];
    let quote_prefix = &quote_token[..8.min(quote_token.len())];
    OrderBookId(format!("{}_{}", base_prefix, quote_prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_orderbook_id() {
        let id = derive_orderbook_id(
            "7BgBvyjrZX1YKz4oh9mjb8ZScatkkwb8DzFx7LoiVkM3",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        );
        assert_eq!(id.as_str(), "7BgBvyjr_EPjFWdd5");
    }

    #[test]
    fn test_orderbook_id_serde() {
        let id = OrderBookId::from("test_id");
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"test_id\"");
        let back: OrderBookId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn test_pubkey_str_serde() {
        let pk = PubkeyStr::new("7BgBvyjrZX1YKz4oh9mjb8ZScatkkwb8DzFx7LoiVkM3");
        let json = serde_json::to_string(&pk).unwrap();
        assert_eq!(json, "\"7BgBvyjrZX1YKz4oh9mjb8ZScatkkwb8DzFx7LoiVkM3\"");
    }

    #[test]
    fn test_side_serde() {
        let bid: Side = serde_json::from_str("\"bid\"").unwrap();
        assert_eq!(bid, Side::Bid);
        let ask: Side = serde_json::from_str("\"ask\"").unwrap();
        assert_eq!(ask, Side::Ask);
    }

    #[test]
    fn test_resolution_serde() {
        let r: Resolution = serde_json::from_str("\"1h\"").unwrap();
        assert_eq!(r, Resolution::Hour1);
        assert_eq!(r.seconds(), 3600);
    }
}

// ─── SubmitOrderRequest ──────────────────────────────────────────────────────

/// Request for submitting a signed order via REST API.
///
/// Bridges the program module (on-chain order signing) with the API module
/// (REST order submission).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitOrderRequest {
    pub maker: String,
    pub nonce: u32,
    pub market_pubkey: String,
    pub base_token: String,
    pub quote_token: String,
    pub side: u32,
    pub maker_amount: u64,
    pub taker_amount: u64,
    #[serde(default)]
    pub expiration: i64,
    pub signature: String,
    pub orderbook_id: String,
}
