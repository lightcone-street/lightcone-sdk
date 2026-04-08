//! Machine-readable rejection codes from the backend API.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Machine-readable rejection code from the backend.
///
/// Deserializes from any case format (snake_case, SCREAMING_SNAKE_CASE).
/// Unrecognized codes fall back to `Unknown(String)` for forward compatibility.
#[derive(Debug, Clone, PartialEq)]
pub enum RejectionCode {
    InsufficientBalance,
    Expired,
    NonceMismatch,
    SelfTrade,
    MarketInactive,
    BelowMinOrderSize,
    InvalidNonce,
    BroadcastFailure,
    OrderNotFound,
    NotOrderMaker,
    OrderAlreadyFilled,
    OrderAlreadyCancelled,
    Unknown(String),
}

impl RejectionCode {
    /// Human-readable label for UI display.
    ///
    /// `InsufficientBalance` → `"Insufficient Balance"`
    pub fn label(&self) -> String {
        match self {
            Self::InsufficientBalance => "Insufficient Balance".to_string(),
            Self::Expired => "Expired".to_string(),
            Self::NonceMismatch => "Nonce Mismatch".to_string(),
            Self::SelfTrade => "Self Trade".to_string(),
            Self::MarketInactive => "Market Inactive".to_string(),
            Self::BelowMinOrderSize => "Below Min Order Size".to_string(),
            Self::InvalidNonce => "Invalid Nonce".to_string(),
            Self::BroadcastFailure => "Broadcast Failure".to_string(),
            Self::OrderNotFound => "Order Not Found".to_string(),
            Self::NotOrderMaker => "Not Order Maker".to_string(),
            Self::OrderAlreadyFilled => "Order Already Filled".to_string(),
            Self::OrderAlreadyCancelled => "Order Already Cancelled".to_string(),
            Self::Unknown(code) => code.clone(),
        }
    }

    /// Wire format (SCREAMING_SNAKE_CASE).
    fn wire_name(&self) -> String {
        match self {
            Self::InsufficientBalance => "INSUFFICIENT_BALANCE".to_string(),
            Self::Expired => "EXPIRED".to_string(),
            Self::NonceMismatch => "NONCE_MISMATCH".to_string(),
            Self::SelfTrade => "SELF_TRADE".to_string(),
            Self::MarketInactive => "MARKET_INACTIVE".to_string(),
            Self::BelowMinOrderSize => "BELOW_MIN_ORDER_SIZE".to_string(),
            Self::InvalidNonce => "INVALID_NONCE".to_string(),
            Self::BroadcastFailure => "BROADCAST_FAILURE".to_string(),
            Self::OrderNotFound => "ORDER_NOT_FOUND".to_string(),
            Self::NotOrderMaker => "NOT_ORDER_MAKER".to_string(),
            Self::OrderAlreadyFilled => "ORDER_ALREADY_FILLED".to_string(),
            Self::OrderAlreadyCancelled => "ORDER_ALREADY_CANCELLED".to_string(),
            Self::Unknown(code) => code.clone(),
        }
    }

    fn from_str(raw: &str) -> Self {
        match raw.to_uppercase().as_str() {
            "INSUFFICIENT_BALANCE" => Self::InsufficientBalance,
            "EXPIRED" => Self::Expired,
            "NONCE_MISMATCH" => Self::NonceMismatch,
            "SELF_TRADE" => Self::SelfTrade,
            "MARKET_INACTIVE" => Self::MarketInactive,
            "BELOW_MIN_ORDER_SIZE" => Self::BelowMinOrderSize,
            "INVALID_NONCE" => Self::InvalidNonce,
            "BROADCAST_FAILURE" => Self::BroadcastFailure,
            "ORDER_NOT_FOUND" => Self::OrderNotFound,
            "NOT_ORDER_MAKER" => Self::NotOrderMaker,
            "ORDER_ALREADY_FILLED" => Self::OrderAlreadyFilled,
            "ORDER_ALREADY_CANCELLED" => Self::OrderAlreadyCancelled,
            _ => Self::Unknown(raw.to_string()),
        }
    }
}

impl fmt::Display for RejectionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl Serialize for RejectionCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.wire_name())
    }
}

impl<'de> Deserialize<'de> for RejectionCode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(Self::from_str(&raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_screaming_snake_case() {
        let code: RejectionCode = serde_json::from_str("\"INSUFFICIENT_BALANCE\"").unwrap();
        assert_eq!(code, RejectionCode::InsufficientBalance);
    }

    #[test]
    fn test_deserialize_lower_snake_case() {
        let code: RejectionCode = serde_json::from_str("\"insufficient_balance\"").unwrap();
        assert_eq!(code, RejectionCode::InsufficientBalance);
    }

    #[test]
    fn test_deserialize_unknown_preserves_original() {
        let code: RejectionCode = serde_json::from_str("\"new_thing\"").unwrap();
        assert_eq!(code, RejectionCode::Unknown("new_thing".to_string()));
    }

    #[test]
    fn test_label_known_code() {
        assert_eq!(
            RejectionCode::InsufficientBalance.label(),
            "Insufficient Balance"
        );
        assert_eq!(RejectionCode::SelfTrade.label(), "Self Trade");
        assert_eq!(
            RejectionCode::BelowMinOrderSize.label(),
            "Below Min Order Size"
        );
    }

    #[test]
    fn test_label_unknown_code() {
        assert_eq!(
            RejectionCode::Unknown("CUSTOM_CODE".to_string()).label(),
            "CUSTOM_CODE"
        );
    }

    #[test]
    fn test_display_uses_label() {
        assert_eq!(
            format!("{}", RejectionCode::InsufficientBalance),
            "Insufficient Balance"
        );
    }

    #[test]
    fn test_serialize_known_code() {
        let json = serde_json::to_string(&RejectionCode::InsufficientBalance).unwrap();
        assert_eq!(json, "\"INSUFFICIENT_BALANCE\"");
    }

    #[test]
    fn test_serialize_unknown_code() {
        let json =
            serde_json::to_string(&RejectionCode::Unknown("CUSTOM_CODE".to_string())).unwrap();
        assert_eq!(json, "\"CUSTOM_CODE\"");
    }

    #[test]
    fn test_roundtrip_all_known_codes() {
        let codes = vec![
            RejectionCode::InsufficientBalance,
            RejectionCode::Expired,
            RejectionCode::NonceMismatch,
            RejectionCode::SelfTrade,
            RejectionCode::MarketInactive,
            RejectionCode::BelowMinOrderSize,
            RejectionCode::InvalidNonce,
            RejectionCode::BroadcastFailure,
            RejectionCode::OrderNotFound,
            RejectionCode::NotOrderMaker,
            RejectionCode::OrderAlreadyFilled,
            RejectionCode::OrderAlreadyCancelled,
        ];
        for code in codes {
            let json = serde_json::to_string(&code).unwrap();
            let back: RejectionCode = serde_json::from_str(&json).unwrap();
            assert_eq!(code, back);
        }
    }
}
