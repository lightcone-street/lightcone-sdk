//! Shared newtypes and utilities used across all domain modules.
//!
//! These types are serialization-transparent: they serialize/deserialize identically
//! to the raw format the backend sends, so they can be used directly in wire types
//! without conversion overhead.

pub mod api_response;
pub mod fmt;
pub mod price;
pub mod rejection;
pub mod scaling;
pub mod serde_util;
pub mod signing;

pub use api_response::{ApiRejectedDetails, ApiResponse};
pub use price::{format_decimal, parse_decimal};
pub use rejection::RejectionCode;
pub use scaling::{scale_price_size, OrderbookDecimals, ScaledAmounts, ScalingError};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{path::Display, str::FromStr};

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
///
/// Serializes as `"bid"`/`"ask"`. Deserializes from `"bid"`/`"ask"` or `"buy"`/`"sell"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    #[serde(rename = "bid", alias = "buy")]
    Bid,
    #[serde(rename = "ask", alias = "sell")]
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

// ─── TimeInForce ─────────────────────────────────────────────────────────────

/// Time-in-force policy for order execution.
///
/// Serializes as uppercase strings: `"GTC"`, `"IOC"`, `"FOK"`, `"ALO"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TimeInForce {
    /// Good-til-cancelled (default)
    #[default]
    #[serde(rename = "GTC")]
    Gtc,
    /// Immediate-or-cancel
    #[serde(rename = "IOC")]
    Ioc,
    /// Fill-or-kill
    #[serde(rename = "FOK")]
    Fok,
    /// Add-liquidity-only (post-only)
    #[serde(rename = "ALO")]
    Alo,
}

// ─── TriggerType ─────────────────────────────────────────────────────────────

/// Trigger order type.
///
/// Serializes as `"TP"` (take-profit) or `"SL"` (stop-loss).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TriggerType {
    #[serde(rename = "TP")]
    TakeProfit,
    #[serde(rename = "SL")]
    StopLoss,
}

impl std::fmt::Display for TriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerType::TakeProfit => write!(f, "TP"),
            TriggerType::StopLoss => write!(f, "SL"),
        }
    }
}

// ─── TriggerStatus ──────────────────────────────────────────────────────────

/// Lifecycle status of a trigger order from WS updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerStatus {
    /// Trigger order was just created and is now pending.
    Created,
    /// Trigger condition met, order was submitted.
    Triggered,
    /// Trigger condition met, but order submission failed.
    Failed,
    /// Trigger condition met, but the pre-signed order had expired.
    Expired,
    /// Trigger order was invalidated.
    Invalidated,
}

// ─── OrderUpdateType ────────────────────────────────────────────────────────

/// WS limit order update type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderUpdateType {
    Placement,
    #[default]
    Update,
    Cancellation,
}

// ─── TriggerUpdateType ─────────────────────────────────────────────────────

/// WS trigger order update type (uppercase version of TriggerStatus).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TriggerUpdateType {
    /// Trigger order was just created.
    Created,
    #[default]
    Triggered,
    Failed,
    Expired,
    Invalidated,
}

// ─── TriggerResultStatus ────────────────────────────────────────────────────

/// Result status of a triggered order after matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerResultStatus {
    /// Order matched (at least partially).
    Filled,
    /// Order placed on book (GTC with no immediate match).
    Accepted,
    /// FOK/IOC that couldn't fill.
    Rejected,
}

// ─── DepositSource ──────────────────────────────────────────────────────────

/// Where collateral should be sourced when matching an order.
///
/// Use `None` for the default behavior (auto: global if available, then market).
/// Serializes as `"global"` or `"market"` to match the REST API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DepositSource {
    /// Always use the user's global deposit balance.
    Global,
    /// Only use market-level balance (no global fallback).
    Market,
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
            Self::Hour4 => 14400,
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

    #[test]
    fn test_time_in_force_serde_roundtrip() {
        let cases = [
            (TimeInForce::Gtc, "\"GTC\""),
            (TimeInForce::Ioc, "\"IOC\""),
            (TimeInForce::Fok, "\"FOK\""),
            (TimeInForce::Alo, "\"ALO\""),
        ];
        for (variant, expected_json) in &cases {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json);
            let back: TimeInForce = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant);
        }
    }

    #[test]
    fn test_time_in_force_default() {
        assert_eq!(TimeInForce::default(), TimeInForce::Gtc);
    }

    #[test]
    fn test_trigger_type_serde_roundtrip() {
        let cases = [
            (TriggerType::TakeProfit, "\"TP\""),
            (TriggerType::StopLoss, "\"SL\""),
        ];
        for (variant, expected_json) in &cases {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json);
            let back: TriggerType = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant);
        }
    }

    #[test]
    fn test_submit_order_request_without_tif_trigger() {
        let req = SubmitOrderRequest {
            maker: "maker".into(),
            nonce: 1,
            salt: 0,
            market_pubkey: "market".into(),
            base_token: "base".into(),
            quote_token: "quote".into(),
            side: 0,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: "sig".into(),
            orderbook_id: "ob".into(),
            time_in_force: None,
            trigger_price: None,
            trigger_type: None,
            deposit_source: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        // Optional fields should be omitted when None
        assert!(!json.contains("tif"));
        assert!(!json.contains("trigger_price"));
        assert!(!json.contains("trigger_type"));
        assert!(!json.contains("deposit_source"));
    }

    #[test]
    fn test_submit_order_request_with_tif_trigger() {
        let req = SubmitOrderRequest {
            maker: "maker".into(),
            nonce: 1,
            salt: 0,
            market_pubkey: "market".into(),
            base_token: "base".into(),
            quote_token: "quote".into(),
            side: 0,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: "sig".into(),
            orderbook_id: "ob".into(),
            time_in_force: Some(TimeInForce::Ioc),
            trigger_price: Some(0.55),
            trigger_type: Some(TriggerType::TakeProfit),
            deposit_source: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"tif\":\"IOC\""));
        assert!(json.contains("\"trigger_price\":0.55"));
        assert!(json.contains("\"trigger_type\":\"TP\""));

        let back: SubmitOrderRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.time_in_force, Some(TimeInForce::Ioc));
        assert_eq!(back.trigger_price, Some(0.55));
        assert_eq!(back.trigger_type, Some(TriggerType::TakeProfit));
    }

    #[test]
    fn test_deposit_source_serde_roundtrip() {
        let cases = [
            (DepositSource::Global, "\"global\""),
            (DepositSource::Market, "\"market\""),
        ];
        for (variant, expected_json) in &cases {
            let json = serde_json::to_string(variant).unwrap();
            assert_eq!(&json, expected_json);
            let back: DepositSource = serde_json::from_str(&json).unwrap();
            assert_eq!(&back, variant);
        }
    }

    #[test]
    fn test_submit_order_request_with_deposit_source() {
        let req = SubmitOrderRequest {
            maker: "maker".into(),
            nonce: 1,
            salt: 0,
            market_pubkey: "market".into(),
            base_token: "base".into(),
            quote_token: "quote".into(),
            side: 0,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: "sig".into(),
            orderbook_id: "ob".into(),
            time_in_force: None,
            trigger_price: None,
            trigger_type: None,
            deposit_source: Some(DepositSource::Global),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"deposit_source\":\"global\""));

        let back: SubmitOrderRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.deposit_source, Some(DepositSource::Global));
    }

    #[test]
    fn test_submit_order_request_deposit_source_omitted_when_none() {
        let req = SubmitOrderRequest {
            maker: "maker".into(),
            nonce: 1,
            salt: 0,
            market_pubkey: "market".into(),
            base_token: "base".into(),
            quote_token: "quote".into(),
            side: 0,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: "sig".into(),
            orderbook_id: "ob".into(),
            time_in_force: None,
            trigger_price: None,
            trigger_type: None,
            deposit_source: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("deposit_source"));
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
    pub nonce: u64,
    pub salt: u64,
    pub market_pubkey: String,
    pub base_token: String,
    pub quote_token: String,
    pub side: u32,
    pub amount_in: u64,
    pub amount_out: u64,
    #[serde(default)]
    pub expiration: i64,
    pub signature: String,
    pub orderbook_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "tif")]
    pub time_in_force: Option<TimeInForce>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_type: Option<TriggerType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deposit_source: Option<DepositSource>,
}
