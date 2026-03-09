#![doc = include_str!("README.md")]

pub mod client;
pub mod wire;

use serde::{Deserialize, Serialize};

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
