#![doc = include_str!("README.md")]

pub mod client;
pub mod wire;

use serde::{Deserialize, Serialize};

/// Machine-readable error code returned when redeeming a referral code.
///
/// Parsing is case-sensitive to match the API wire contract. Unrecognized
/// values are preserved for forward compatibility.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferralRedeemErrorCode {
    InvalidCode,
    CodeFullyRedeemed,
    AlreadyBeta,
    SelfReferral,
    AlreadyRedeemedByUser,
    RateLimited,
    InternalError,
    Unknown(String),
}

impl ReferralRedeemErrorCode {
    /// Return the exact API wire value.
    pub fn as_str(&self) -> &str {
        match self {
            Self::InvalidCode => "INVALID_CODE",
            Self::CodeFullyRedeemed => "CODE_FULLY_REDEEMED",
            Self::AlreadyBeta => "ALREADY_BETA",
            Self::SelfReferral => "SELF_REFERRAL",
            Self::AlreadyRedeemedByUser => "ALREADY_REDEEMED_BY_USER",
            Self::RateLimited => "RATE_LIMITED",
            Self::InternalError => "INTERNAL_ERROR",
            Self::Unknown(code) => code,
        }
    }
}

impl From<&str> for ReferralRedeemErrorCode {
    fn from(value: &str) -> Self {
        match value {
            "INVALID_CODE" => Self::InvalidCode,
            "CODE_FULLY_REDEEMED" => Self::CodeFullyRedeemed,
            "ALREADY_BETA" => Self::AlreadyBeta,
            "SELF_REFERRAL" => Self::SelfReferral,
            "ALREADY_REDEEMED_BY_USER" => Self::AlreadyRedeemedByUser,
            "RATE_LIMITED" => Self::RateLimited,
            "INTERNAL_ERROR" => Self::InternalError,
            code => Self::Unknown(code.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferralCodeInfo {
    pub code: String,
    pub max_uses: i32,
    pub use_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReferralStatus {
    pub is_beta: bool,
    pub source: Option<String>,
    pub referral_codes: Vec<ReferralCodeInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedeemResult {
    pub success: bool,
    pub is_beta: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn referral_redeem_error_codes_use_exact_wire_values() {
        let cases = [
            (ReferralRedeemErrorCode::InvalidCode, "INVALID_CODE"),
            (
                ReferralRedeemErrorCode::CodeFullyRedeemed,
                "CODE_FULLY_REDEEMED",
            ),
            (ReferralRedeemErrorCode::AlreadyBeta, "ALREADY_BETA"),
            (ReferralRedeemErrorCode::SelfReferral, "SELF_REFERRAL"),
            (
                ReferralRedeemErrorCode::AlreadyRedeemedByUser,
                "ALREADY_REDEEMED_BY_USER",
            ),
            (ReferralRedeemErrorCode::RateLimited, "RATE_LIMITED"),
            (ReferralRedeemErrorCode::InternalError, "INTERNAL_ERROR"),
        ];

        for (code, wire_value) in cases {
            assert_eq!(code.as_str(), wire_value);
            assert_eq!(ReferralRedeemErrorCode::from(wire_value), code);
        }
    }

    #[test]
    fn referral_redeem_error_code_preserves_unknown_values() {
        for value in ["invalid_code", "FUTURE_ERROR"] {
            let code = ReferralRedeemErrorCode::from(value);

            assert_eq!(code, ReferralRedeemErrorCode::Unknown(value.to_string()));
            assert_eq!(code.as_str(), value);
        }
    }
}
