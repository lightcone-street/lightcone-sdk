//! Wire types for admin metadata requests and responses.

use serde::{Deserialize, Serialize};

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
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deposit_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
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
