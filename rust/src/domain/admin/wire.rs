//! Wire types for admin requests and responses.

use serde::{Deserialize, Serialize, Serializer};

/// Request payload for `POST /api/admin/metadata`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnifiedMetadataRequest {
    #[serde(default)]
    pub markets: Vec<MarketMetadataPayload>,
    #[serde(default)]
    pub outcomes: Vec<OutcomeMetadataPayload>,
    #[serde(default)]
    pub conditional_tokens: Vec<ConditionalTokenMetadataPayload>,
    #[serde(default)]
    pub deposit_tokens: Vec<DepositTokenMetadataPayload>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketMetadataPayload {
    pub market_id: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subcategory: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub featured_rank: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeMetadataPayload {
    pub market_id: i64,
    pub outcome_index: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalTokenMetadataPayload {
    pub conditional_mint_id: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome_index: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deposit_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositTokenMetadataPayload {
    pub deposit_asset: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_error: Option<String>,
}

/// Response from `POST /api/admin/metadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMetadataResponse {
    pub status: String,
    #[serde(default)]
    pub markets: Vec<serde_json::Value>,
    #[serde(default)]
    pub outcomes: Vec<serde_json::Value>,
    #[serde(default)]
    pub conditional_tokens: Vec<serde_json::Value>,
    #[serde(default)]
    pub deposit_tokens: Vec<serde_json::Value>,
}

// ============================================================================
// REFERRAL ADMIN
// ============================================================================

/// Target specifier for admin referral operations.
///
/// Serializes to the shapes the backend expects:
/// - `TargetSpec::All` → `"all"`
/// - `TargetSpec::ById { .. }` → `{ "user_id": "..." }`
/// - `TargetSpec::ByWallet { .. }` → `{ "wallet_address": "..." }`
/// - `TargetSpec::ByCode { .. }` → `{ "code": "..." }`
/// - `TargetSpec::ByBatch { .. }` → `{ "batch_id": "..." }`
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum TargetSpec {
    All(AllTarget),
    ById { user_id: String },
    ByWallet { wallet_address: String },
    ByCode { code: String },
    ByBatch { batch_id: String },
}

impl TargetSpec {
    pub fn all() -> Self {
        Self::All(AllTarget)
    }

    pub fn user_id(id: impl Into<String>) -> Self {
        Self::ById { user_id: id.into() }
    }

    pub fn wallet_address(addr: impl Into<String>) -> Self {
        Self::ByWallet {
            wallet_address: addr.into(),
        }
    }

    pub fn code(code: impl Into<String>) -> Self {
        Self::ByCode { code: code.into() }
    }

    pub fn batch_id(id: impl Into<String>) -> Self {
        Self::ByBatch {
            batch_id: id.into(),
        }
    }
}

/// Marker type that serializes to the string `"all"`.
#[derive(Debug, Clone)]
pub struct AllTarget;

impl Serialize for AllTarget {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("all")
    }
}

/// Request payload for `POST /api/admin/referral/allocate`.
#[derive(Debug, Clone, Serialize)]
pub struct AllocateCodesRequest {
    pub target: TargetSpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vanity_codes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_uses: Option<i32>,
}

/// Response from `POST /api/admin/referral/allocate`.
///
/// The backend returns different shapes for "all" vs single-user targets,
/// so optional fields cover both cases.
#[derive(Debug, Clone, Deserialize)]
pub struct AllocateCodesResponse {
    pub status: String,
    #[serde(default)]
    pub users_count: Option<u32>,
    #[serde(default)]
    pub codes_allocated: Option<u32>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub codes: Option<Vec<String>>,
}

/// Request payload for `POST /api/admin/referral/whitelist`.
#[derive(Debug, Clone, Serialize)]
pub struct WhitelistRequest {
    pub wallet_addresses: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocate_codes: Option<bool>,
}

/// Response from `POST /api/admin/referral/whitelist`.
#[derive(Debug, Clone, Deserialize)]
pub struct WhitelistResponse {
    pub status: String,
    pub wallets_added: u32,
    pub codes_allocated: u32,
}

/// Request payload for `POST /api/admin/referral/revoke`.
#[derive(Debug, Clone, Serialize)]
pub struct RevokeRequest {
    pub target: TargetSpec,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Response from `POST /api/admin/referral/revoke`.
#[derive(Debug, Clone, Deserialize)]
pub struct RevokeResponse {
    pub revoked_count: u32,
    pub user_ids: Vec<String>,
}

/// Request payload for `POST /api/admin/referral/unrevoke`.
#[derive(Debug, Clone, Serialize)]
pub struct UnrevokeRequest {
    pub target: TargetSpec,
}

/// Response from `POST /api/admin/referral/unrevoke`.
#[derive(Debug, Clone, Deserialize)]
pub struct UnrevokeResponse {
    pub restored_count: u32,
    pub user_ids: Vec<String>,
}

// ============================================================================
// NOTIFICATION ADMIN
// ============================================================================

/// Request payload for `POST /api/admin/notifications`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNotificationRequest {
    pub title: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Response from `POST /api/admin/notifications`.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateNotificationResponse {
    pub status: String,
}

/// Request payload for `POST /api/admin/notifications/dismiss`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DismissNotificationRequest {
    pub notification_id: String,
}

/// Response from `POST /api/admin/notifications/dismiss`.
#[derive(Debug, Clone, Deserialize)]
pub struct DismissNotificationResponse {
    pub status: String,
}
