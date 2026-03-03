//! Wire types for referral API responses.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralStatusResponse {
    pub is_beta: bool,
    pub source: Option<String>,
    pub referral_codes: Vec<ReferralCodeWire>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralCodeWire {
    pub code: String,
    pub max_uses: i32,
    pub use_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemRequest {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemResponse {
    pub success: bool,
    pub is_beta: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemErrorResponse {
    pub error: String,
    pub code: String,
}
