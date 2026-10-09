//! Machine-readable rejection and error codes from the backend API.
//!
//! Two independent code families ride the error envelope
//! (`{"status":"error","error_details":{...}}`):
//!
//! - [`RejectionCode`] (`rejection_code`): a business outcome of a trading
//!   mutation that the engine evaluated, delivered with HTTP 200.
//! - [`ErrorCode`] (`error_code`): a transport, validation, authorization, or
//!   availability failure, delivered with a 4xx/5xx status.
//!
//! Both fall back to an `Unknown(String)` variant, so an unrecognized code never
//! fails deserialization of the surrounding error.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Business rejection code for a trading mutation (HTTP 200 error envelope).
///
/// Deserializes from any case format (snake_case, SCREAMING_SNAKE_CASE).
/// Unrecognized codes fall back to `Unknown(String)` for forward compatibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RejectionCode {
    /// This signed order identity was already accepted.
    DuplicateOrder,
    /// Requested funding cannot cover the complete order.
    InsufficientBalance,
    /// A fill-or-kill order's full base target is not executable.
    FokInsufficientLiquidity,
    /// The order would cross the same wallet's executable liquidity.
    SelfTrade,
    /// The order violates current trading rules (price/size precision, tick,
    /// or the minimum order size).
    InvalidOrder,
    /// The request exceeds supported limits.
    InvalidRequest,
    /// The signed order had already expired.
    OrderExpired,
    /// Trading is paused for this market, exchange, or deposit token.
    TradingPaused,
    /// The canonical mutation signature did not verify.
    InvalidSignature,
    /// A cancel-all salt was already consumed.
    CancelAllReplay,
    /// Trading state changed while processing; the request may be retried.
    StaleState,
    /// The request expired before acceptance; the request may be retried.
    RequestExpired,
    /// Trading capacity is unavailable, including the per-wallet open-order cap.
    TradingCapacityUnavailable,
    /// Committed trading state is not ready for this request yet.
    TradingNotReady,
    /// Committed order state could not be read.
    InternalError,
    /// The cancelled order hash was never accepted.
    OrderNotFound,
    Unknown(String),
}

impl RejectionCode {
    /// Human-readable label for UI display.
    ///
    /// `InsufficientBalance` → `"Insufficient Balance"`
    pub fn label(&self) -> String {
        match self {
            Self::DuplicateOrder => "Duplicate Order".to_string(),
            Self::InsufficientBalance => "Insufficient Balance".to_string(),
            Self::FokInsufficientLiquidity => "FOK Insufficient Liquidity".to_string(),
            Self::SelfTrade => "Self Trade".to_string(),
            Self::InvalidOrder => "Invalid Order".to_string(),
            Self::InvalidRequest => "Invalid Request".to_string(),
            Self::OrderExpired => "Order Expired".to_string(),
            Self::TradingPaused => "Trading Paused".to_string(),
            Self::InvalidSignature => "Invalid Signature".to_string(),
            Self::CancelAllReplay => "Cancel-All Replay".to_string(),
            Self::StaleState => "Stale State".to_string(),
            Self::RequestExpired => "Request Expired".to_string(),
            Self::TradingCapacityUnavailable => "Trading Capacity Unavailable".to_string(),
            Self::TradingNotReady => "Trading Not Ready".to_string(),
            Self::InternalError => "Internal Error".to_string(),
            Self::OrderNotFound => "Order Not Found".to_string(),
            Self::Unknown(code) => code.clone(),
        }
    }

    /// True for rejections where the order was not accepted because of a
    /// transient engine condition, so the same signed order may be submitted
    /// again (while it has not expired).
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::StaleState
                | Self::RequestExpired
                | Self::TradingCapacityUnavailable
                | Self::TradingNotReady
        )
    }

    /// Wire format (SCREAMING_SNAKE_CASE).
    fn wire_name(&self) -> String {
        match self {
            Self::DuplicateOrder => "DUPLICATE_ORDER".to_string(),
            Self::InsufficientBalance => "INSUFFICIENT_BALANCE".to_string(),
            Self::FokInsufficientLiquidity => "FOK_INSUFFICIENT_LIQUIDITY".to_string(),
            Self::SelfTrade => "SELF_TRADE".to_string(),
            Self::InvalidOrder => "INVALID_ORDER".to_string(),
            Self::InvalidRequest => "INVALID_REQUEST".to_string(),
            Self::OrderExpired => "ORDER_EXPIRED".to_string(),
            Self::TradingPaused => "TRADING_PAUSED".to_string(),
            Self::InvalidSignature => "INVALID_SIGNATURE".to_string(),
            Self::CancelAllReplay => "CANCEL_ALL_REPLAY".to_string(),
            Self::StaleState => "STALE_STATE".to_string(),
            Self::RequestExpired => "REQUEST_EXPIRED".to_string(),
            Self::TradingCapacityUnavailable => "TRADING_CAPACITY_UNAVAILABLE".to_string(),
            Self::TradingNotReady => "TRADING_NOT_READY".to_string(),
            Self::InternalError => "INTERNAL_ERROR".to_string(),
            Self::OrderNotFound => "ORDER_NOT_FOUND".to_string(),
            Self::Unknown(code) => code.clone(),
        }
    }

    fn from_str(raw: &str) -> Self {
        match raw.to_uppercase().as_str() {
            "DUPLICATE_ORDER" => Self::DuplicateOrder,
            "INSUFFICIENT_BALANCE" => Self::InsufficientBalance,
            "FOK_INSUFFICIENT_LIQUIDITY" => Self::FokInsufficientLiquidity,
            "SELF_TRADE" => Self::SelfTrade,
            "INVALID_ORDER" => Self::InvalidOrder,
            "INVALID_REQUEST" => Self::InvalidRequest,
            "ORDER_EXPIRED" => Self::OrderExpired,
            "TRADING_PAUSED" => Self::TradingPaused,
            "INVALID_SIGNATURE" => Self::InvalidSignature,
            "CANCEL_ALL_REPLAY" => Self::CancelAllReplay,
            "STALE_STATE" => Self::StaleState,
            "REQUEST_EXPIRED" => Self::RequestExpired,
            "TRADING_CAPACITY_UNAVAILABLE" => Self::TradingCapacityUnavailable,
            "TRADING_NOT_READY" => Self::TradingNotReady,
            "INTERNAL_ERROR" => Self::InternalError,
            "ORDER_NOT_FOUND" => Self::OrderNotFound,
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

/// Transport-level `error_code` carried by 4xx/5xx error envelopes.
///
/// Engine gRPC failures map to fixed HTTP statuses; the status is listed on
/// each variant. Read it from a rejection with
/// [`ApiRejectedDetails::error_code_kind`](crate::shared::ApiRejectedDetails::error_code_kind).
/// Unrecognized codes fall back to `Unknown(String)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// 400: malformed request field, bad side, zero amount, non-canonical key,
    /// bad cancel-all salt, or stale cancel-all timestamp.
    InvalidArgument,
    /// 400: engine precondition failed outside the business-rejection path.
    FailedPrecondition,
    /// 404.
    NotFound,
    /// 403: mutation signature rejected or stale cancel-all timestamp.
    Forbidden,
    /// 409: duplicate order identity or reused cancel-all salt.
    AlreadyExists,
    /// 409: trading state changed; retry the request.
    Aborted,
    /// 429: engine admission queue or capacity exhausted; retry later.
    ResourceExhausted,
    /// 503: engine unavailable or deadline exceeded. For order submission the
    /// outcome is unknown: resubmit the identical signed request, where
    /// [`RejectionCode::DuplicateOrder`] or [`Self::AlreadyExists`] proves the
    /// first was accepted.
    EngineUnavailable,
    /// 500: engine failure without a public reason.
    EngineInternalError,
    /// 503: committed trading state (or orderbook metadata) is temporarily
    /// unavailable; also returned for an unknown orderbook on submission.
    TradingUnavailable,
    /// 401: an order mutation had no authenticated session.
    AuthRequired,
    /// 403: the session wallet does not match the order maker or
    /// cancellation wallet.
    AuthWalletMismatch,
    /// 400: unsupported time-in-force (only GTC, IOC and FOK are accepted).
    InvalidTif,
    /// 400: unsupported deposit source.
    InvalidDepositSource,
    /// 400: signature is not 64 bytes of hex.
    InvalidSignature,
    /// 400: a path or query public key is not canonical base58.
    InvalidPubkey,
    /// 400: a pagination cursor is malformed or combined incorrectly.
    InvalidCursor,
    /// 400: order hash is not 64 lowercase hex characters.
    InvalidOrderHash,
    /// 400: the route does not accept a query string.
    UnexpectedQuery,
    /// 400: a page `limit` is outside the route's accepted range.
    InvalidLimit,
    /// 429: request rate limit exceeded.
    RateLimited,
    Unknown(String),
}

impl ErrorCode {
    /// Wire format (SCREAMING_SNAKE_CASE).
    pub fn as_str(&self) -> &str {
        match self {
            Self::InvalidArgument => "INVALID_ARGUMENT",
            Self::FailedPrecondition => "FAILED_PRECONDITION",
            Self::NotFound => "NOT_FOUND",
            Self::Forbidden => "FORBIDDEN",
            Self::AlreadyExists => "ALREADY_EXISTS",
            Self::Aborted => "ABORTED",
            Self::ResourceExhausted => "RESOURCE_EXHAUSTED",
            Self::EngineUnavailable => "ENGINE_UNAVAILABLE",
            Self::EngineInternalError => "ENGINE_INTERNAL_ERROR",
            Self::TradingUnavailable => "TRADING_UNAVAILABLE",
            Self::AuthRequired => "AUTH_REQUIRED",
            Self::AuthWalletMismatch => "AUTH_WALLET_MISMATCH",
            Self::InvalidTif => "INVALID_TIF",
            Self::InvalidDepositSource => "INVALID_DEPOSIT_SOURCE",
            Self::InvalidSignature => "INVALID_SIGNATURE",
            Self::InvalidPubkey => "INVALID_PUBKEY",
            Self::InvalidCursor => "INVALID_CURSOR",
            Self::InvalidOrderHash => "INVALID_ORDER_HASH",
            Self::UnexpectedQuery => "UNEXPECTED_QUERY",
            Self::InvalidLimit => "INVALID_LIMIT",
            Self::RateLimited => "RATE_LIMITED",
            Self::Unknown(code) => code,
        }
    }

    /// Parse a wire code; unrecognized codes become `Unknown`.
    pub fn from_wire(raw: &str) -> Self {
        match raw {
            "INVALID_ARGUMENT" => Self::InvalidArgument,
            "FAILED_PRECONDITION" => Self::FailedPrecondition,
            "NOT_FOUND" => Self::NotFound,
            "FORBIDDEN" => Self::Forbidden,
            "ALREADY_EXISTS" => Self::AlreadyExists,
            "ABORTED" => Self::Aborted,
            "RESOURCE_EXHAUSTED" => Self::ResourceExhausted,
            "ENGINE_UNAVAILABLE" => Self::EngineUnavailable,
            "ENGINE_INTERNAL_ERROR" => Self::EngineInternalError,
            "TRADING_UNAVAILABLE" => Self::TradingUnavailable,
            "AUTH_REQUIRED" => Self::AuthRequired,
            "AUTH_WALLET_MISMATCH" => Self::AuthWalletMismatch,
            "INVALID_TIF" => Self::InvalidTif,
            "INVALID_DEPOSIT_SOURCE" => Self::InvalidDepositSource,
            "INVALID_SIGNATURE" => Self::InvalidSignature,
            "INVALID_PUBKEY" => Self::InvalidPubkey,
            "INVALID_CURSOR" => Self::InvalidCursor,
            "INVALID_ORDER_HASH" => Self::InvalidOrderHash,
            "UNEXPECTED_QUERY" => Self::UnexpectedQuery,
            "INVALID_LIMIT" => Self::InvalidLimit,
            "RATE_LIMITED" => Self::RateLimited,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// True for conditions that may clear on their own, where retrying the
    /// same request later is reasonable. For order submission,
    /// [`Self::EngineUnavailable`] additionally means the outcome is unknown:
    /// retry only the identical signed request, never a re-signed order.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Aborted
                | Self::ResourceExhausted
                | Self::EngineUnavailable
                | Self::TradingUnavailable
                | Self::RateLimited
        )
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ErrorCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(Self::from_wire(&raw))
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
    fn removed_codes_decode_as_unknown() {
        for removed in [
            "NONCE_MISMATCH",
            "FOK_NO_FILL",
            "BELOW_MIN_ORDER_SIZE",
            "EXPIRED",
        ] {
            let code: RejectionCode = serde_json::from_value(serde_json::json!(removed)).unwrap();
            assert_eq!(code, RejectionCode::Unknown(removed.to_string()));
        }
    }

    #[test]
    fn test_label_known_code() {
        assert_eq!(
            RejectionCode::InsufficientBalance.label(),
            "Insufficient Balance"
        );
        assert_eq!(RejectionCode::SelfTrade.label(), "Self Trade");
        assert_eq!(
            RejectionCode::FokInsufficientLiquidity.label(),
            "FOK Insufficient Liquidity"
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
            RejectionCode::DuplicateOrder,
            RejectionCode::InsufficientBalance,
            RejectionCode::FokInsufficientLiquidity,
            RejectionCode::SelfTrade,
            RejectionCode::InvalidOrder,
            RejectionCode::InvalidRequest,
            RejectionCode::OrderExpired,
            RejectionCode::TradingPaused,
            RejectionCode::InvalidSignature,
            RejectionCode::CancelAllReplay,
            RejectionCode::StaleState,
            RejectionCode::RequestExpired,
            RejectionCode::TradingCapacityUnavailable,
            RejectionCode::TradingNotReady,
            RejectionCode::InternalError,
            RejectionCode::OrderNotFound,
        ];
        for code in codes {
            let json = serde_json::to_string(&code).unwrap();
            let back: RejectionCode = serde_json::from_str(&json).unwrap();
            assert_eq!(code, back);
            assert!(!matches!(back, RejectionCode::Unknown(_)), "{json}");
        }
    }

    #[test]
    fn transient_rejections_are_the_retryable_admission_failures() {
        assert!(RejectionCode::StaleState.is_transient());
        assert!(RejectionCode::TradingNotReady.is_transient());
        assert!(!RejectionCode::DuplicateOrder.is_transient());
        assert!(!RejectionCode::InsufficientBalance.is_transient());
    }

    #[test]
    fn error_codes_round_trip_and_tolerate_unknown_values() {
        let codes = [
            ErrorCode::InvalidArgument,
            ErrorCode::FailedPrecondition,
            ErrorCode::NotFound,
            ErrorCode::Forbidden,
            ErrorCode::AlreadyExists,
            ErrorCode::Aborted,
            ErrorCode::ResourceExhausted,
            ErrorCode::EngineUnavailable,
            ErrorCode::EngineInternalError,
            ErrorCode::TradingUnavailable,
            ErrorCode::AuthRequired,
            ErrorCode::AuthWalletMismatch,
            ErrorCode::InvalidTif,
            ErrorCode::InvalidDepositSource,
            ErrorCode::InvalidSignature,
            ErrorCode::InvalidPubkey,
            ErrorCode::InvalidCursor,
            ErrorCode::InvalidOrderHash,
            ErrorCode::UnexpectedQuery,
            ErrorCode::InvalidLimit,
            ErrorCode::RateLimited,
        ];
        for code in codes {
            let json = serde_json::to_string(&code).unwrap();
            assert_eq!(serde_json::from_str::<ErrorCode>(&json).unwrap(), code);
        }
        let unknown: ErrorCode = serde_json::from_str("\"WALLET_NOT_AUTHORIZED\"").unwrap();
        assert_eq!(
            unknown,
            ErrorCode::Unknown("WALLET_NOT_AUTHORIZED".to_string())
        );
        assert!(ErrorCode::EngineUnavailable.is_retryable());
        assert!(!ErrorCode::AuthWalletMismatch.is_retryable());
    }
}
