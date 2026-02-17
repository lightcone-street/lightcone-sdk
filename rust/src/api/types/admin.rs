//! Admin-related types for the Lightcone REST API.
//!
//! The backend exposes a single admin endpoint: `POST /api/admin/metadata`
//! which uses a signed envelope pattern for authentication.

use serde::{Deserialize, Serialize};

/// Signed admin request envelope.
///
/// All admin requests must be wrapped in this envelope. The `payload` is
/// serialized to canonical JSON (sorted keys) and signed with an ED25519
/// key that is authorized in the backend's `ADMIN_PUBKEYS` configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminEnvelope<T: Serialize> {
    /// The request payload
    pub payload: T,
    /// Base58-encoded ED25519 signature over the canonical JSON of the payload
    pub signature: String,
}

/// Top-level request payload for `POST /api/admin/metadata`.
///
/// At least one section must be non-empty.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnifiedMetadataRequest {
    /// Market metadata to upsert
    #[serde(default)]
    pub markets: Vec<MarketMetadataPayload>,
    /// Outcome metadata to upsert
    #[serde(default)]
    pub outcomes: Vec<OutcomeMetadataPayload>,
    /// Conditional token metadata to upsert
    #[serde(default)]
    pub conditional_tokens: Vec<ConditionalTokenMetadataPayload>,
    /// Deposit token metadata to upsert
    #[serde(default)]
    pub deposit_tokens: Vec<DepositTokenMetadataPayload>,
}

/// Market metadata payload.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarketMetadataPayload {
    /// Market identifier (required)
    pub market_id: i64,
    /// Market name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_name: Option<String>,
    /// URL-friendly slug
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Market description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Market definition/rules
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    /// Banner image URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_image_url: Option<String>,
    /// Icon URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Category
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Subcategory
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subcategory: Option<String>,
    /// Tags for filtering
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Featured rank (0-10)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub featured_rank: Option<i16>,
    /// Metadata URI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    /// S3 sync status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced: Option<bool>,
    /// S3 sync timestamp (ISO 8601)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced_at: Option<String>,
    /// S3 sync error
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_error: Option<String>,
}

/// Outcome metadata payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeMetadataPayload {
    /// Market identifier (required)
    pub market_id: i64,
    /// Outcome index (required)
    pub outcome_index: i16,
    /// Outcome name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Icon URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Metadata URI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    /// S3 sync status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced: Option<bool>,
    /// S3 sync timestamp (ISO 8601)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced_at: Option<String>,
    /// S3 sync error
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_error: Option<String>,
}

/// Conditional token metadata payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalTokenMetadataPayload {
    /// Conditional mint database ID (required)
    pub conditional_mint_id: i32,
    /// Outcome index
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome_index: Option<i16>,
    /// Display name for UI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Outcome label (e.g., "YES", "NO")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    /// Deposit symbol (e.g., "USDC")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deposit_symbol: Option<String>,
    /// Short name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    /// Description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Icon URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Metadata URI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    /// Token decimals (0-18)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<i16>,
    /// S3 sync status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced: Option<bool>,
    /// S3 sync timestamp (ISO 8601)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced_at: Option<String>,
    /// S3 sync error
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_error: Option<String>,
}

/// Deposit token metadata payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositTokenMetadataPayload {
    /// Deposit asset mint address (required)
    pub deposit_asset: String,
    /// Display name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Symbol
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// On-chain token symbol
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_symbol: Option<String>,
    /// Description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Icon URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    /// Metadata URI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    /// Token decimals (0-18)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<i16>,
    /// S3 sync status
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced: Option<bool>,
    /// S3 sync timestamp (ISO 8601)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_synced_at: Option<String>,
    /// S3 sync error
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s3_error: Option<String>,
}

/// Response from `POST /api/admin/metadata`.
///
/// Empty sections are omitted from the JSON response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMetadataResponse {
    /// Status (always "success" on 200)
    pub status: String,
    /// Upserted market metadata records
    #[serde(default)]
    pub markets: Vec<serde_json::Value>,
    /// Upserted outcome metadata records
    #[serde(default)]
    pub outcomes: Vec<serde_json::Value>,
    /// Upserted conditional token metadata records
    #[serde(default)]
    pub conditional_tokens: Vec<serde_json::Value>,
    /// Upserted deposit token metadata records
    #[serde(default)]
    pub deposit_tokens: Vec<serde_json::Value>,
}
