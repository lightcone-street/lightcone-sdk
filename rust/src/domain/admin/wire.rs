//! Wire types for admin requests and responses.

use crate::domain::market::{wire::MarketResponse, Status};
use crate::shared::{OrderBookId, PubkeyStr};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

fn deserialize_optional_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

// ============================================================================
// ADMIN AUTH
// ============================================================================

/// Response from `GET /api/admin/nonce` — contains the nonce and message to sign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminNonceResponse {
    pub nonce: String,
    pub message: String,
}

/// Request payload for `POST /api/admin/login`.
#[derive(Debug, Clone, Serialize)]
pub struct AdminLoginRequest {
    pub message: String,
    pub signature_bs58: String,
    pub pubkey_bytes: Vec<u8>,
}

/// Response from `POST /api/admin/login` — contains session metadata.
/// The admin token is set as an HttpOnly cookie by the backend.
#[derive(Debug, Clone, Deserialize)]
pub struct AdminLoginResponse {
    pub wallet_address: String,
    pub expires_at: i64,
}

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
    pub banner_image_url_low: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_image_url_medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_image_url_high: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
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
    /// Resolution deadline update.
    ///
    /// - `None` omits the field and preserves the backend value.
    /// - `Some(Some(ms))` sends a Unix timestamp in milliseconds and sets the deadline.
    /// - `Some(None)` sends JSON `null` and clears the deadline.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_nullable"
    )]
    pub resolution_by: Option<Option<i64>>,
}

impl MarketMetadataPayload {
    /// Set or update the market's resolution deadline.
    pub fn with_resolution_by(mut self, resolution_by_ms: i64) -> Self {
        self.resolution_by = Some(Some(resolution_by_ms));
        self
    }

    /// Clear the market's configured resolution deadline.
    pub fn with_cleared_resolution_by(mut self) -> Self {
        self.resolution_by = Some(None);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeMetadataPayload {
    pub market_id: i64,
    pub outcome_index: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_long: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalTokenMetadataPayload {
    pub conditional_mint_id: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
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
    pub icon_url_low: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_order_size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binance_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binance_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub okx_inst_id: Option<String>,
}

/// Response from `POST /api/admin/metadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedMetadataResponse {
    #[serde(default)]
    pub markets: Vec<serde_json::Value>,
    #[serde(default)]
    pub outcomes: Vec<serde_json::Value>,
    #[serde(default)]
    pub conditional_tokens: Vec<serde_json::Value>,
    #[serde(default)]
    pub deposit_tokens: Vec<DepositTokenMetadataResponse>,
}

/// Deposit token metadata row returned from `POST /api/admin/metadata`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepositTokenMetadataResponse {
    pub id: i64,
    pub deposit_asset: PubkeyStr,
    pub display_name: String,
    pub symbol: String,
    #[serde(default)]
    pub token_symbol: Option<String>,
    #[serde(default)]
    pub binance_symbol: Option<String>,
    #[serde(default)]
    pub binance_enabled: bool,
    #[serde(default)]
    pub okx_inst_id: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_url_low: Option<String>,
    #[serde(default)]
    pub icon_url_medium: Option<String>,
    #[serde(default)]
    pub icon_url_high: Option<String>,
    #[serde(default)]
    pub metadata_uri: Option<String>,
    #[serde(default)]
    pub decimals: Option<i16>,
    #[serde(default)]
    pub min_order_size: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Response from `GET /api/admin/metadata/markets/{market_id}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminMarketMetadataResponse {
    pub market: AdminMetadataMarket,
    #[serde(default)]
    pub market_metadata: Option<AdminMarketMetadataRow>,
    #[serde(default)]
    pub deposit_assets: Vec<AdminMarketDepositAsset>,
    #[serde(default)]
    pub outcomes: Vec<AdminOutcomeMetadataEntry>,
    pub missing_metadata: AdminMissingMetadata,
}

/// Canonical market row included in focused admin metadata responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminMetadataMarket {
    pub id: i64,
    pub market_pubkey: PubkeyStr,
    pub market_id: i64,
    pub num_outcomes: i16,
    pub oracle: PubkeyStr,
    pub question_id: String,
    pub condition_id: String,
    #[serde(default)]
    pub bump: Option<i16>,
    pub market_status: String,
    #[serde(default)]
    pub winning_outcome: Option<i16>,
    pub has_winning_outcome: bool,
    #[serde(default)]
    pub payout_numerators: Option<Vec<i64>>,
    #[serde(default)]
    pub payout_denominator: Option<i64>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub activated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub settled_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// Market metadata row, or `None` in [`AdminMarketMetadataResponse`] when missing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminMarketMetadataRow {
    pub id: i64,
    pub market_id: i64,
    #[serde(default)]
    pub market_name: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub definition: Option<String>,
    #[serde(default)]
    pub banner_image_url_low: Option<String>,
    #[serde(default)]
    pub banner_image_url_medium: Option<String>,
    #[serde(default)]
    pub banner_image_url_high: Option<String>,
    #[serde(default)]
    pub icon_url_low: Option<String>,
    #[serde(default)]
    pub icon_url_medium: Option<String>,
    #[serde(default)]
    pub icon_url_high: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub subcategory: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub featured_rank: Option<i16>,
    #[serde(default)]
    pub metadata_uri: Option<String>,
    #[serde(default)]
    pub resolution_by: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Market deposit asset row included in focused admin metadata responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminMarketDepositAsset {
    pub id: i32,
    pub market_id: i64,
    pub market_pubkey: PubkeyStr,
    pub deposit_asset: PubkeyStr,
    pub vault: PubkeyStr,
    pub num_outcomes: i16,
    pub created_at: DateTime<Utc>,
}

/// Outcome metadata plus conditional token rows for one outcome index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminOutcomeMetadataEntry {
    pub outcome_index: i16,
    #[serde(default)]
    pub outcome_metadata: Option<AdminOutcomeMetadataRow>,
    #[serde(default)]
    pub conditional_tokens: Vec<AdminConditionalTokenMetadataEntry>,
}

/// Outcome metadata row, or `None` in [`AdminOutcomeMetadataEntry`] when missing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminOutcomeMetadataRow {
    pub id: i64,
    pub market_id: i64,
    pub outcome_index: i16,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub name_long: Option<String>,
    #[serde(default)]
    pub icon_url_low: Option<String>,
    #[serde(default)]
    pub icon_url_medium: Option<String>,
    #[serde(default)]
    pub icon_url_high: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub metadata_uri: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Conditional mint row plus optional metadata for one conditional token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminConditionalTokenMetadataEntry {
    pub conditional_mint: AdminConditionalMintRow,
    #[serde(default)]
    pub conditional_token_metadata: Option<AdminConditionalTokenMetadataRow>,
}

/// Conditional mint database row included in focused admin metadata responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminConditionalMintRow {
    pub id: i32,
    pub market_deposit_mint_id: i32,
    pub deposit_asset: PubkeyStr,
    pub outcome_index: i16,
    pub token_address: PubkeyStr,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub uri: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Conditional token metadata row, or `None` when missing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminConditionalTokenMetadataRow {
    pub id: i64,
    pub conditional_mint_id: i32,
    pub outcome_index: i16,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub outcome: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub deposit_symbol: Option<String>,
    #[serde(default)]
    pub short_symbol: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_url_low: Option<String>,
    #[serde(default)]
    pub icon_url_medium: Option<String>,
    #[serde(default)]
    pub icon_url_high: Option<String>,
    #[serde(default)]
    pub metadata_uri: Option<String>,
    #[serde(default)]
    pub decimals: Option<i16>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Conditional token metadata database row returned by derived-field resync.
///
/// This is distinct from [`AdminConditionalTokenMetadataRow`]: the resync
/// endpoint returns the database column name `short_name`, while focused admin
/// market reads expose `short_symbol`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConditionalTokenMetadataRow {
    pub id: i64,
    pub conditional_mint_id: i32,
    pub outcome_index: i16,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub outcome: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub deposit_symbol: Option<String>,
    #[serde(default)]
    pub short_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_url_low: Option<String>,
    #[serde(default)]
    pub icon_url_medium: Option<String>,
    #[serde(default)]
    pub icon_url_high: Option<String>,
    #[serde(default)]
    pub metadata_uri: Option<String>,
    #[serde(default)]
    pub decimals: Option<i16>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Missing metadata row identifiers returned by focused market metadata reads.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdminMissingMetadata {
    #[serde(default)]
    pub market_metadata: bool,
    #[serde(default)]
    pub outcomes: Vec<i16>,
    #[serde(default)]
    pub conditional_tokens: Vec<i32>,
}

/// Request body for `PUT /api/admin/metadata/markets/{market_id}`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateMarketMetadataRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market: Option<UpdateMarketMetadataPayload>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcomes: Vec<UpdateOutcomeMetadataPayload>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditional_tokens: Vec<UpdateConditionalTokenMetadataPayload>,
}

/// Market-level update payload for focused market metadata updates.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateMarketMetadataPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_image_url_low: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_image_url_medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner_image_url_high: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
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
    /// Resolution deadline update.
    ///
    /// - `None` omits the field and preserves the backend value.
    /// - `Some(Some(ms))` sends a Unix timestamp in milliseconds and sets the deadline.
    /// - `Some(None)` sends JSON `null` and clears the deadline.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_nullable"
    )]
    pub resolution_by: Option<Option<i64>>,
}

impl UpdateMarketMetadataPayload {
    /// Set or update the market's resolution deadline.
    pub fn with_resolution_by(mut self, resolution_by_ms: i64) -> Self {
        self.resolution_by = Some(Some(resolution_by_ms));
        self
    }

    /// Clear the market's configured resolution deadline.
    pub fn with_cleared_resolution_by(mut self) -> Self {
        self.resolution_by = Some(None);
        self
    }
}

/// Outcome update payload for focused market metadata updates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateOutcomeMetadataPayload {
    pub outcome_index: i16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_long: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
}

/// Conditional token update payload for focused market metadata updates.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UpdateConditionalTokenMetadataPayload {
    pub conditional_mint_id: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
}

/// Focused market metadata update response.
pub type UpdateMarketMetadataResponse = UnifiedMetadataResponse;

/// Request body for rewriting conditional token metadata JSON.
///
/// `PUT /api/admin/metadata/markets/{market_id}/conditional-tokens/{conditional_mint_id}/metadata-json`
///
/// Image fields must be WebP data URLs when provided. The backend reuses
/// existing database image URLs for omitted variants; a high image URL must
/// already exist or be provided as `image_data_url_high`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateConditionalTokenMetadataJsonRequest {
    pub name: String,
    pub symbol: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_data_url_low: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_data_url_medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_data_url_high: Option<String>,
}

/// Response from rewriting conditional token metadata JSON.
///
/// `PUT /api/admin/metadata/markets/{market_id}/conditional-tokens/{conditional_mint_id}/metadata-json`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConditionalTokenMetadataJsonResponse {
    pub conditional_mint: PubkeyStr,
    pub metadata_uri: String,
    #[serde(default)]
    pub image_url_low: Option<String>,
    #[serde(default)]
    pub image_url_medium: Option<String>,
    #[serde(default)]
    pub image_url_high: Option<String>,
    pub database_updated: bool,
    #[serde(default)]
    pub invalidation_paths: Vec<String>,
}

/// Response from resyncing derived conditional-token metadata fields.
///
/// `POST /api/admin/metadata/markets/{market_id}/conditional-tokens/resync-derived`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResyncConditionalTokenDerivedMetadataResponse {
    #[serde(default)]
    pub conditional_tokens: Vec<ConditionalTokenMetadataRow>,
}

/// Three quality variants for metadata images.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdminImageVariants {
    pub low: String,
    pub medium: String,
    pub high: String,
}

/// Request body for `PUT /api/admin/metadata/markets/{market_id}/images`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateMarketImagesRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_icon: Option<AdminImageVariants>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_banner: Option<AdminImageVariants>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outcomes: Vec<UpdateOutcomeImageRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conditional_tokens: Vec<UpdateConditionalTokenImageRequest>,
}

/// Outcome image replacement payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateOutcomeImageRequest {
    pub outcome_index: i16,
    pub icon: AdminImageVariants,
}

/// Conditional token image replacement payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateConditionalTokenImageRequest {
    pub conditional_mint_id: i32,
    pub icon: AdminImageVariants,
}

/// Request body for `PUT /api/admin/metadata/deposit-tokens/{deposit_asset}/images`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateDepositTokenImagesRequest {
    pub icon: AdminImageVariants,
}

/// Request body for `POST /api/admin/metadata/deposit-tokens/{deposit_asset}/images/upload`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UploadDepositTokenImagesRequest {
    pub icon: AdminImageVariants,
}

/// Response from `POST /api/admin/metadata/deposit-tokens/{deposit_asset}/images/upload`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UploadDepositTokenImagesResponse {
    pub deposit_asset: PubkeyStr,
    pub icon_url_low: String,
    pub icon_url_medium: String,
    pub icon_url_high: String,
    pub database_updated: bool,
    #[serde(default)]
    pub invalidation_paths: Vec<String>,
}

/// Image replacement response from `PUT /api/admin/metadata/markets/{market_id}/images`
/// and `PUT /api/admin/metadata/deposit-tokens/{deposit_asset}/images`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetadataImageUpdateResponse {
    #[serde(default)]
    pub updated: Vec<MetadataImageUpdate>,
    pub database_updated: bool,
    #[serde(default)]
    pub invalidation_paths: Vec<String>,
}

/// One replaced metadata image target.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetadataImageUpdate {
    pub target_type: MetadataImageTargetType,
    #[serde(default)]
    pub outcome_index: Option<i16>,
    #[serde(default)]
    pub conditional_mint_id: Option<i32>,
    #[serde(default)]
    pub conditional_mint: Option<PubkeyStr>,
    #[serde(default)]
    pub deposit_asset: Option<PubkeyStr>,
    pub urls: AdminImageVariants,
}

/// Metadata image target discriminator.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MetadataImageTargetType {
    MarketIcon,
    MarketBanner,
    OutcomeIcon,
    ConditionalTokenIcon,
    DepositTokenIcon,
}

/// Response from `GET /api/admin/metadata/deposit-tokens/{deposit_asset}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdminDepositTokenMetadataResponse {
    pub deposit_asset: PubkeyStr,
    pub deposit_token_metadata: DepositTokenMetadataResponse,
}

/// Response from `GET /api/admin/metadata/deposit-tokens`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdminDepositTokenMetadataListResponse {
    #[serde(default)]
    pub deposit_tokens: Vec<DepositTokenMetadataResponse>,
}

/// Request body for `PUT /api/admin/metadata/deposit-tokens/{deposit_asset}`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateDepositTokenMetadataRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_low: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_medium: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_url_high: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_order_size: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binance_symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binance_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub okx_inst_id: Option<String>,
}

/// Response from `PUT /api/admin/metadata/deposit-tokens/{deposit_asset}`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateDepositTokenMetadataResponse {
    #[serde(default)]
    pub deposit_tokens: Vec<DepositTokenMetadataResponse>,
}

/// Response body from `GET /api/admin/metadata/categories`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetadataCategoriesResponse {
    #[serde(default)]
    pub categories: Vec<String>,
}

/// Request body for `POST /api/admin/metadata/categories`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddMetadataCategoryRequest {
    pub category: String,
}

/// Response body from `POST /api/admin/metadata/categories`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AddMetadataCategoryResponse {
    pub category: String,
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

// ============================================================================
// ADMIN MARKETS
// ============================================================================

/// Status filter for `GET /api/admin/markets`.
///
/// This is intentionally the SDK-facing filter vocabulary. Do not model or send
/// `settled`; resolved markets are selected with `Resolved`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AdminMarketStatusFilter {
    All,
    Active,
    Resolved,
}

/// Market lifecycle values returned by `GET /api/admin/markets`.
///
/// The admin table currently exposes only `Active` and `Resolved` rows, but this
/// reuses the SDK's canonical market lifecycle type so the wire shape stays
/// aligned with the rest of the Rust SDK.
pub type AdminMarketStatus = Status;

/// Query parameters for `GET /api/admin/markets`.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct AdminMarketsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_status: Option<AdminMarketStatusFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_volume_24h_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_volume_24h_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_volume_7d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_volume_7d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_volume_30d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_volume_30d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_volume_total_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_volume_total_usd: Option<Decimal>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_unique_traders_24h: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_unique_traders_24h: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_unique_traders_7d: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_unique_traders_7d: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_unique_traders_30d: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_unique_traders_30d: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_unique_traders_total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_unique_traders_total: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_open_interest_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_open_interest_usd: Option<Decimal>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_fees_24h_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fees_24h_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_fees_7d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fees_7d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_fees_30d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fees_30d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_fees_total_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fees_total_usd: Option<Decimal>,
}

/// Shared query parameters for admin metrics table endpoints.
///
/// Used by `GET /api/admin/deposit-tokens` and `GET /api/admin/categories`.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct AdminMetricsTableQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_volume_24h_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_volume_24h_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_volume_7d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_volume_7d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_volume_30d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_volume_30d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_volume_total_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_volume_total_usd: Option<Decimal>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_unique_traders_24h: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_unique_traders_24h: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_unique_traders_7d: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_unique_traders_7d: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_unique_traders_30d: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_unique_traders_30d: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_unique_traders_total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_unique_traders_total: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_open_interest_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_open_interest_usd: Option<Decimal>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_fees_24h_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fees_24h_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_fees_7d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fees_7d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_fees_30d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fees_30d_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_fees_total_usd: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fees_total_usd: Option<Decimal>,
}

/// Query parameters for `GET /api/admin/deposit-tokens`.
pub type AdminDepositTokensQuery = AdminMetricsTableQuery;

/// Query parameters for `GET /api/admin/categories`.
pub type AdminCategoriesQuery = AdminMetricsTableQuery;

/// Response from `GET /api/admin/markets`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminMarketsResponse {
    pub timestamp: i64,
    pub sort_by: String,
    pub sort_direction: String,
    pub total: u64,
    pub limit: u32,
    #[serde(default)]
    pub next_cursor: Option<u64>,
    pub has_more: bool,
    #[serde(default)]
    pub markets: Vec<AdminMarketRow>,
}

/// A single row in the admin markets table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminMarketRow {
    pub rank: u64,
    pub market_id: i64,
    pub market_pubkey: PubkeyStr,
    pub market_status: AdminMarketStatus,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub market_name: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    pub num_outcomes: u32,
    #[serde(default)]
    pub resolution_by: Option<i64>,
    pub open_interest_usd: Decimal,
    pub volume_24h_usd: Decimal,
    pub volume_7d_usd: Decimal,
    pub volume_30d_usd: Decimal,
    pub volume_total_usd: Decimal,
    pub unique_traders_24h: u64,
    pub unique_traders_7d: u64,
    pub unique_traders_30d: u64,
    pub unique_traders_total: u64,
    pub fees_24h_usd: Decimal,
    pub fees_7d_usd: Decimal,
    pub fees_30d_usd: Decimal,
    pub fees_total_usd: Decimal,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub activated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub settled_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

/// Response from `GET /api/admin/deposit-tokens`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminDepositTokensResponse {
    pub timestamp: i64,
    pub sort_by: String,
    pub sort_direction: String,
    pub total: u64,
    pub limit: u32,
    #[serde(default)]
    pub next_cursor: Option<u64>,
    pub has_more: bool,
    #[serde(default)]
    pub deposit_tokens: Vec<AdminDepositTokenRow>,
}

/// A single row in the admin deposit tokens metrics table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminDepositTokenRow {
    pub rank: u64,
    pub deposit_asset: PubkeyStr,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub token_symbol: Option<String>,
    #[serde(default)]
    pub binance_symbol: Option<String>,
    #[serde(default)]
    pub okx_inst_id: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub decimals: Option<i16>,
    #[serde(default)]
    pub min_order_size: Option<i64>,
    pub open_interest_usd: Decimal,
    pub volume_24h_usd: Decimal,
    pub volume_7d_usd: Decimal,
    pub volume_30d_usd: Decimal,
    pub volume_total_usd: Decimal,
    pub unique_traders_24h: u64,
    pub unique_traders_7d: u64,
    pub unique_traders_30d: u64,
    pub unique_traders_total: u64,
    pub fees_24h_usd: Decimal,
    pub fees_7d_usd: Decimal,
    pub fees_30d_usd: Decimal,
    pub fees_total_usd: Decimal,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Response from `GET /api/admin/categories`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminCategoriesResponse {
    pub timestamp: i64,
    pub sort_by: String,
    pub sort_direction: String,
    pub total: u64,
    pub limit: u32,
    #[serde(default)]
    pub next_cursor: Option<u64>,
    pub has_more: bool,
    #[serde(default)]
    pub categories: Vec<AdminCategoryRow>,
}

/// A single row in the admin categories metrics table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdminCategoryRow {
    pub rank: u64,
    #[serde(default)]
    pub category_id: Option<i64>,
    pub category: String,
    pub category_key: String,
    pub market_count: u64,
    pub active_market_count: u64,
    pub resolved_market_count: u64,
    pub open_interest_usd: Decimal,
    pub volume_24h_usd: Decimal,
    pub volume_7d_usd: Decimal,
    pub volume_30d_usd: Decimal,
    pub volume_total_usd: Decimal,
    pub unique_traders_24h: u64,
    pub unique_traders_7d: u64,
    pub unique_traders_30d: u64,
    pub unique_traders_total: u64,
    pub fees_24h_usd: Decimal,
    pub fees_7d_usd: Decimal,
    pub fees_30d_usd: Decimal,
    pub fees_total_usd: Decimal,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
}

// ============================================================================
// MARKETS TO SETTLE ADMIN
// ============================================================================

/// Response from `GET /api/admin/markets-to-settle/count`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketsToSettleCountResponse {
    pub markets_to_settle_count: u64,
}

/// Query parameters for `GET /api/admin/markets-to-settle`.
///
/// Pagination is cursor-based using the previous response's `next_cursor`
/// value, which is a `market_id`.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct MarketsToSettleQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Response from `GET /api/admin/markets-to-settle`.
///
/// `markets` intentionally uses the raw public REST [`MarketResponse`] shape
/// rather than the validated [`crate::domain::market::Market`] domain shape.
/// Admin settlement views must see ready-to-settle rows even when optional
/// enrichment is incomplete, while domain conversion can reject incomplete rows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketsToSettleResponse {
    #[serde(default)]
    pub markets: Vec<MarketResponse>,
    #[serde(default)]
    pub next_cursor: Option<i64>,
    pub has_more: bool,
}

// ============================================================================
// REFERRAL CONFIG / CODES ADMIN
// ============================================================================

/// Response from `POST /api/admin/referral/config/get` and
/// `POST /api/admin/referral/config/update`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferralConfig {
    pub default_code_count: i32,
    pub updated_at: DateTime<Utc>,
}

/// Request payload for `POST /api/admin/referral/config/update`.
///
/// `default_code_count: None` is accepted by the backend as a no-op; set `Some`
/// to change the server-wide default.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateConfigRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_code_count: Option<i32>,
}

/// Request payload for `POST /api/admin/referral/codes` (admin list).
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListCodesRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub limit: u32,
    pub offset: u32,
}

/// Response from `POST /api/admin/referral/codes`.
#[derive(Debug, Clone, Deserialize)]
pub struct ListCodesResponse {
    pub codes: Vec<CodeListEntry>,
    pub count: usize,
}

/// A single referral code returned from the admin list endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CodeListEntry {
    pub code: String,
    pub owner_user_id: String,
    pub batch_id: String,
    pub is_vanity: bool,
    pub max_uses: i32,
    pub use_count: i64,
    pub created_at: DateTime<Utc>,
}

/// Request payload for `POST /api/admin/referral/codes/update`.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateCodeRequest {
    pub code: String,
    pub max_uses: i32,
}

/// Response from `POST /api/admin/referral/codes/update`.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCodeResponse {
    pub status: String,
    pub code: String,
    pub max_uses: i32,
}

// ============================================================================
// ADMIN LOGS
// ============================================================================

/// Query for `GET /api/admin/logs/events`.
///
/// All filters are optional; pagination is cursor-based.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AdminLogEventsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_names: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severities: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operations: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprints: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_statuses: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_codes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejection_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejection_codes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_visible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_pubkey: Option<PubkeyStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_pubkey: Option<PubkeyStr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orderbook_id: Option<OrderBookId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

/// Response from `GET /api/admin/logs/events`.
#[derive(Debug, Clone, Deserialize)]
pub struct AdminLogEventsResponse {
    pub events: Vec<AdminLogEvent>,
    #[serde(default)]
    pub next_cursor: Option<String>,
    pub limit: u32,
}

/// A single log event from `GET /api/admin/logs/events`
/// or `GET /api/admin/logs/events/{public_id}`.
#[derive(Debug, Clone, Deserialize)]
pub struct AdminLogEvent {
    pub id: i64,
    pub public_id: String,
    pub service_name: String,
    pub environment: String,
    pub component: String,
    pub operation: String,
    pub category: String,
    pub severity: String,
    pub occurred_at_ms: i64,
    #[serde(default)]
    pub occurred_at: Option<String>,
    pub created_at_ms: i64,
    #[serde(default)]
    pub created_at: Option<String>,
    pub user_visible: bool,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub user_pubkey: Option<PubkeyStr>,
    #[serde(default)]
    pub market_pubkey: Option<PubkeyStr>,
    #[serde(default)]
    pub orderbook_id: Option<OrderBookId>,
    #[serde(default)]
    pub order_hash: Option<String>,
    #[serde(default)]
    pub trigger_order_id: Option<String>,
    #[serde(default)]
    pub tx_signature: Option<String>,
    #[serde(default)]
    pub checkpoint_signature: Option<String>,
    #[serde(default)]
    pub http_status: Option<i32>,
    #[serde(default)]
    pub grpc_code: Option<String>,
    pub message: String,
    #[serde(default)]
    pub fingerprint: Option<String>,
    #[serde(default)]
    pub response_status: Option<String>,
    #[serde(default)]
    pub error_code: Option<String>,
    #[serde(default)]
    pub rejection_code: Option<String>,
    pub context: serde_json::Value,
}

/// Query for `GET /api/admin/logs/metrics`.
///
/// `windows` and `scopes` are CSV lists (e.g. `"1h,24h"` or `"service,component"`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct AdminLogMetricsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub windows: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_per_scope: Option<u32>,
}

/// Response from `GET /api/admin/logs/metrics`.
#[derive(Debug, Clone, Deserialize)]
pub struct AdminLogMetricsResponse {
    #[serde(default)]
    pub computed_at: Option<String>,
    pub computed_at_ms: i64,
    pub breakdowns: Vec<AdminLogMetricBreakdown>,
}

/// A single (window, scope) breakdown in `AdminLogMetricsResponse`.
#[derive(Debug, Clone, Deserialize)]
pub struct AdminLogMetricBreakdown {
    pub window: String,
    pub scope: String,
    pub rows: Vec<AdminLogMetricSummary>,
}

/// A summary row within an `AdminLogMetricBreakdown`.
#[derive(Debug, Clone, Deserialize)]
pub struct AdminLogMetricSummary {
    pub scope_key: String,
    pub total_count: u64,
    pub error_count: u64,
    pub critical_count: u64,
    pub user_visible_count: u64,
    pub computed_at_ms: i64,
    #[serde(default)]
    pub computed_at: Option<String>,
}

/// Query for `GET /api/admin/logs/metrics/history`.
///
/// `scope` is required (e.g. `"service"`, `"component"`); `scope_key` narrows
/// the history to a single key within that scope.
#[derive(Debug, Clone, Serialize)]
pub struct AdminLogMetricHistoryQuery {
    pub scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_key: Option<String>,
    pub resolution: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

impl AdminLogMetricHistoryQuery {
    /// Construct a history query for the given scope with the default `"1h"` resolution.
    pub fn new(scope: impl Into<String>) -> Self {
        Self {
            scope: scope.into(),
            scope_key: None,
            resolution: "1h".to_string(),
            from_ms: None,
            to_ms: None,
            limit: None,
        }
    }
}

/// Response from `GET /api/admin/logs/metrics/history`.
#[derive(Debug, Clone, Deserialize)]
pub struct AdminLogMetricHistoryResponse {
    pub scope: String,
    pub scope_key: String,
    pub resolution: String,
    pub from_ms: i64,
    pub to_ms: i64,
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub to: Option<String>,
    pub points: Vec<AdminLogMetricPoint>,
}

/// A single bucket in `AdminLogMetricHistoryResponse`.
#[derive(Debug, Clone, Deserialize)]
pub struct AdminLogMetricPoint {
    pub bucket_start_ms: i64,
    #[serde(default)]
    pub bucket_start: Option<String>,
    pub total_count: u64,
    pub error_count: u64,
    pub critical_count: u64,
    pub user_visible_count: u64,
}

/// Response from `GET /api/admin/logs/critical-errors-24h/count`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CriticalLogErrors24hCountResponse {
    pub critical_log_errors_24h: u64,
}

// ============================================================================
// MARKET DEPLOYMENT ASSET UPLOAD
// ============================================================================

/// Request payload for `POST /api/admin/metadata/upload-market-deployment-assets`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadMarketDeploymentAssetsRequest {
    pub market_id: i64,
    pub market_pubkey: String,
    pub market: MarketDeploymentMarket,
    #[serde(default)]
    pub outcomes: Vec<MarketDeploymentOutcome>,
    #[serde(default)]
    pub deposit_assets: Vec<MarketDeploymentDepositAsset>,
    #[serde(default)]
    pub conditional_tokens: Vec<MarketDeploymentConditionalToken>,
}

/// Market-level fields for a deployment asset upload.
///
/// Image uploads are quality-specific WebP data URLs. Hosted URL fields are
/// preserved separately and are used when no matching data URL is supplied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDeploymentMarket {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub definition: Option<String>,
    #[serde(default)]
    pub banner_image_url_low: Option<String>,
    #[serde(default)]
    pub banner_image_url_medium: Option<String>,
    #[serde(default)]
    pub banner_image_url_high: Option<String>,
    #[serde(default)]
    pub icon_url_low: Option<String>,
    #[serde(default)]
    pub icon_url_medium: Option<String>,
    #[serde(default)]
    pub icon_url_high: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub subcategory: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub featured_rank: Option<i32>,
    #[serde(default)]
    pub banner_image_data_url_low: Option<String>,
    #[serde(default)]
    pub banner_image_content_type_low: Option<String>,
    #[serde(default)]
    pub banner_image_data_url_medium: Option<String>,
    #[serde(default)]
    pub banner_image_content_type_medium: Option<String>,
    #[serde(default)]
    pub banner_image_data_url_high: Option<String>,
    #[serde(default)]
    pub banner_image_content_type_high: Option<String>,
    #[serde(default)]
    pub icon_image_data_url_low: Option<String>,
    #[serde(default)]
    pub icon_image_content_type_low: Option<String>,
    #[serde(default)]
    pub icon_image_data_url_medium: Option<String>,
    #[serde(default)]
    pub icon_image_content_type_medium: Option<String>,
    #[serde(default)]
    pub icon_image_data_url_high: Option<String>,
    #[serde(default)]
    pub icon_image_content_type_high: Option<String>,
}

/// A single outcome within an upload deployment asset request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDeploymentOutcome {
    pub index: i32,
    pub name: String,
    pub symbol: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_url_low: Option<String>,
    #[serde(default)]
    pub icon_url_medium: Option<String>,
    #[serde(default)]
    pub icon_url_high: Option<String>,
    #[serde(default)]
    pub icon_image_data_url_low: Option<String>,
    #[serde(default)]
    pub icon_image_content_type_low: Option<String>,
    #[serde(default)]
    pub icon_image_data_url_medium: Option<String>,
    #[serde(default)]
    pub icon_image_content_type_medium: Option<String>,
    #[serde(default)]
    pub icon_image_data_url_high: Option<String>,
    #[serde(default)]
    pub icon_image_content_type_high: Option<String>,
}

/// A deposit asset referenced by the market being deployed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDeploymentDepositAsset {
    pub mint: String,
    pub display_name: String,
    pub symbol: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon_url_low: Option<String>,
    #[serde(default)]
    pub icon_url_medium: Option<String>,
    #[serde(default)]
    pub icon_url_high: Option<String>,
    pub decimals: i32,
}

/// A conditional token to upload image + metadata for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDeploymentConditionalToken {
    pub outcome_index: i32,
    pub deposit_mint: String,
    pub conditional_mint: String,
    pub name: String,
    pub symbol: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub image_data_url_low: Option<String>,
    #[serde(default)]
    pub image_content_type_low: Option<String>,
    #[serde(default)]
    pub image_data_url_medium: Option<String>,
    #[serde(default)]
    pub image_content_type_medium: Option<String>,
    pub image_data_url_high: String,
    pub image_content_type_high: String,
}

/// Response from `POST /api/admin/metadata/upload-market-deployment-assets`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadMarketDeploymentAssetsResponse {
    pub market_metadata_uri: String,
    pub market: UploadedMarketImages,
    #[serde(default)]
    pub outcomes: Vec<UploadedOutcomeImages>,
    #[serde(default)]
    pub deposit_assets: Vec<UploadedDepositAssetImages>,
    #[serde(default)]
    pub tokens: Vec<UploadedConditionalToken>,
}

/// Uploaded market banner/icon URLs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UploadedMarketImages {
    #[serde(default)]
    pub banner_image_url_low: Option<String>,
    #[serde(default)]
    pub banner_image_url_medium: Option<String>,
    #[serde(default)]
    pub banner_image_url_high: Option<String>,
    #[serde(default)]
    pub icon_url_low: Option<String>,
    #[serde(default)]
    pub icon_url_medium: Option<String>,
    #[serde(default)]
    pub icon_url_high: Option<String>,
}

/// Uploaded icon URL for a single outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedOutcomeImages {
    pub index: i32,
    #[serde(default)]
    pub icon_url_low: Option<String>,
    #[serde(default)]
    pub icon_url_medium: Option<String>,
    #[serde(default)]
    pub icon_url_high: Option<String>,
}

/// Uploaded icon URLs for a single deposit asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedDepositAssetImages {
    pub mint: String,
    #[serde(default)]
    pub icon_url_low: Option<String>,
    #[serde(default)]
    pub icon_url_medium: Option<String>,
    #[serde(default)]
    pub icon_url_high: Option<String>,
}

/// Uploaded conditional token image + metadata URIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedConditionalToken {
    pub conditional_mint: String,
    pub metadata_uri: String,
    #[serde(default)]
    pub image_url_low: Option<String>,
    #[serde(default)]
    pub image_url_medium: Option<String>,
    #[serde(default)]
    pub image_url_high: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use serde_json::{json, Value};
    use std::str::FromStr;

    #[test]
    fn add_metadata_category_request_serializes_category() {
        let request = AddMetadataCategoryRequest {
            category: "Crypto".to_string(),
        };

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "category": "Crypto"
            })
        );
    }

    #[test]
    fn metadata_categories_response_deserializes_categories() {
        let response: MetadataCategoriesResponse = serde_json::from_value(json!({
            "categories": ["Politics", "Crypto", "Sports"]
        }))
        .unwrap();

        assert_eq!(
            response.categories,
            vec![
                "Politics".to_string(),
                "Crypto".to_string(),
                "Sports".to_string()
            ]
        );
    }

    #[test]
    fn add_metadata_category_response_deserializes_canonical_category() {
        let response: AddMetadataCategoryResponse = serde_json::from_value(json!({
            "category": "Politics"
        }))
        .unwrap();

        assert_eq!(response.category, "Politics");
    }

    #[test]
    fn market_metadata_payload_omits_resolution_fields_when_none() {
        let request = MarketMetadataPayload {
            market_id: 1,
            market_name: Some("Updated name".to_string()),
            ..Default::default()
        };

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "market_id": 1,
                "market_name": "Updated name"
            })
        );
    }

    #[test]
    fn market_metadata_payload_serializes_resolution_by_timestamp() {
        let request = MarketMetadataPayload {
            market_id: 1,
            resolution_by: Some(Some(1_735_689_600_000)),
            ..Default::default()
        };

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "market_id": 1,
                "resolution_by": 1_735_689_600_000i64
            })
        );
    }

    #[test]
    fn market_metadata_payload_serializes_resolution_by_null_to_clear() {
        let request = MarketMetadataPayload {
            market_id: 1,
            resolution_by: Some(None),
            ..Default::default()
        };

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "market_id": 1,
                "resolution_by": null
            })
        );
    }

    #[test]
    fn market_metadata_payload_preserves_explicit_null_when_deserialized() {
        let request: MarketMetadataPayload = serde_json::from_value(json!({
            "market_id": 1,
            "resolution_by": null
        }))
        .unwrap();

        assert_eq!(request.resolution_by, Some(None));

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "market_id": 1,
                "resolution_by": null
            })
        );
    }

    #[test]
    fn market_metadata_payload_helpers_set_and_clear_resolution_by() {
        let request = MarketMetadataPayload {
            market_id: 1,
            ..Default::default()
        }
        .with_resolution_by(1_735_689_600_000);

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "market_id": 1,
                "resolution_by": 1_735_689_600_000i64
            })
        );

        let request = MarketMetadataPayload {
            market_id: 1,
            ..Default::default()
        }
        .with_cleared_resolution_by();

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "market_id": 1,
                "resolution_by": null
            })
        );
    }

    #[test]
    fn unified_metadata_response_reads_market_resolution_by_values() {
        let response: UnifiedMetadataResponse = serde_json::from_value(json!({
            "markets": [{
                "market_id": 1,
                "resolution_by": 1_735_689_600_000i64
            }, {
                "market_id": 2,
                "resolution_by": null
            }]
        }))
        .unwrap();

        assert_eq!(response.markets.len(), 2);
        assert_eq!(
            response.markets[0]["resolution_by"],
            json!(1_735_689_600_000i64)
        );
        assert!(response.markets[1]["resolution_by"].is_null());
    }

    #[test]
    fn admin_market_status_filter_serializes_supported_values() {
        let all = AdminMarketsQuery {
            market_status: Some(AdminMarketStatusFilter::All),
            ..Default::default()
        };
        let active = AdminMarketsQuery {
            market_status: Some(AdminMarketStatusFilter::Active),
            ..Default::default()
        };
        let resolved = AdminMarketsQuery {
            market_status: Some(AdminMarketStatusFilter::Resolved),
            ..Default::default()
        };

        assert_eq!(
            serde_urlencoded::to_string(all).unwrap(),
            "market_status=all"
        );
        assert_eq!(
            serde_urlencoded::to_string(active).unwrap(),
            "market_status=active"
        );
        assert_eq!(
            serde_urlencoded::to_string(resolved).unwrap(),
            "market_status=resolved"
        );
    }

    #[test]
    fn admin_markets_query_serializes_sort_filters_and_ranges() {
        let query = AdminMarketsQuery {
            cursor: Some(100),
            limit: Some(50),
            sort_by: Some("open_interest_usd".to_string()),
            sort_direction: Some("asc".to_string()),
            market_status: Some(AdminMarketStatusFilter::Resolved),
            category: Some("Crypto".to_string()),
            search: Some("btc".to_string()),
            min_volume_24h_usd: Some(Decimal::from_str("1000").unwrap()),
            max_open_interest_usd: Some(Decimal::from_str("50000").unwrap()),
            min_unique_traders_total: Some(10),
            ..Default::default()
        };

        let query_string = serde_urlencoded::to_string(query).unwrap();
        assert_eq!(
            query_string,
            "cursor=100&limit=50&sort_by=open_interest_usd&sort_direction=asc&market_status=resolved&category=Crypto&search=btc&min_volume_24h_usd=1000&min_unique_traders_total=10&max_open_interest_usd=50000"
        );
    }

    #[test]
    fn admin_markets_response_deserializes_market_rows() {
        let response: AdminMarketsResponse = serde_json::from_value(json!({
            "timestamp": 1_710_000_000_000i64,
            "sort_by": "volume_24h_usd",
            "sort_direction": "desc",
            "total": 123,
            "limit": 100,
            "next_cursor": 100,
            "has_more": true,
            "markets": [{
                "rank": 1,
                "market_id": 123,
                "market_pubkey": "MarketPubkey",
                "market_status": "Active",
                "slug": "btc-100k",
                "market_name": "Will BTC hit $100k?",
                "category": "Crypto",
                "icon_url": "https://example.com/icon.png",
                "num_outcomes": 2,
                "resolution_by": 1_760_000_000_000i64,
                "open_interest_usd": "12345.67",
                "volume_24h_usd": "1000.00",
                "volume_7d_usd": "7000.00",
                "volume_30d_usd": "30000.00",
                "volume_total_usd": "50000.00",
                "unique_traders_24h": 50,
                "unique_traders_7d": 200,
                "unique_traders_30d": 600,
                "unique_traders_total": 900,
                "fees_24h_usd": "0",
                "fees_7d_usd": "0",
                "fees_30d_usd": "0",
                "fees_total_usd": "0",
                "created_at": "2026-01-01T00:00:00+00:00",
                "activated_at": "2026-01-02T00:00:00+00:00",
                "settled_at": null,
                "updated_at": "2026-01-03T00:00:00+00:00"
            }]
        }))
        .unwrap();

        assert_eq!(response.timestamp, 1_710_000_000_000);
        assert_eq!(response.next_cursor, Some(100));
        assert!(response.has_more);
        assert_eq!(response.markets.len(), 1);

        let market = &response.markets[0];
        assert_eq!(market.market_status, AdminMarketStatus::Active);
        assert_eq!(market.resolution_by, Some(1_760_000_000_000));
        assert_eq!(
            market.open_interest_usd,
            Decimal::from_str("12345.67").unwrap()
        );
        assert_eq!(
            market.volume_total_usd,
            Decimal::from_str("50000.00").unwrap()
        );
        assert_eq!(market.unique_traders_total, 900);
        assert_eq!(market.fees_total_usd, Decimal::ZERO);
        assert!(market.settled_at.is_none());
    }

    #[test]
    fn admin_metrics_table_query_serializes_shared_filters() {
        let query = AdminDepositTokensQuery {
            cursor: Some(200),
            limit: Some(25),
            sort_by: Some("fees_total_usd".to_string()),
            sort_direction: Some("ascending".to_string()),
            search: Some("sol".to_string()),
            min_volume_7d_usd: Some(Decimal::from_str("100.25").unwrap()),
            max_unique_traders_30d: Some(500),
            min_open_interest_usd: Some(Decimal::from_str("10").unwrap()),
            ..Default::default()
        };

        let query_string = serde_urlencoded::to_string(query).unwrap();
        assert_eq!(
            query_string,
            "cursor=200&limit=25&sort_by=fees_total_usd&sort_direction=ascending&search=sol&min_volume_7d_usd=100.25&max_unique_traders_30d=500&min_open_interest_usd=10"
        );
    }

    #[test]
    fn admin_deposit_tokens_response_deserializes_metric_rows() {
        let response: AdminDepositTokensResponse = serde_json::from_value(json!({
            "timestamp": 1_770_000_000_000i64,
            "sort_by": "volume_24h_usd",
            "sort_direction": "desc",
            "total": 2,
            "limit": 200,
            "next_cursor": null,
            "has_more": false,
            "deposit_tokens": [{
                "rank": 1,
                "deposit_asset": "So11111111111111111111111111111111111111112",
                "display_name": "Wrapped SOL",
                "symbol": "SOL",
                "token_symbol": "SOL",
                "binance_symbol": "SOLUSDT",
                "okx_inst_id": "SOL-USDT",
                "icon_url": "https://metadata.example/deposit-tokens/sol/icon.webp",
                "decimals": 9,
                "min_order_size": 1000000,
                "open_interest_usd": "125000.50",
                "volume_24h_usd": "42000.25",
                "volume_7d_usd": "250000.75",
                "volume_30d_usd": "900000.00",
                "volume_total_usd": "1250000.00",
                "unique_traders_24h": 40,
                "unique_traders_7d": 180,
                "unique_traders_30d": 420,
                "unique_traders_total": 700,
                "fees_24h_usd": "21.00",
                "fees_7d_usd": "125.50",
                "fees_30d_usd": "450.00",
                "fees_total_usd": "625.00",
                "created_at": "2026-05-21T10:00:00Z",
                "updated_at": "2026-05-21T10:15:00Z"
            }]
        }))
        .unwrap();

        assert_eq!(response.total, 2);
        assert_eq!(response.next_cursor, None);
        assert_eq!(response.deposit_tokens.len(), 1);

        let token = &response.deposit_tokens[0];
        assert_eq!(token.symbol.as_deref(), Some("SOL"));
        assert_eq!(token.decimals, Some(9));
        assert_eq!(
            token.open_interest_usd,
            Decimal::from_str("125000.50").unwrap()
        );
        assert_eq!(token.unique_traders_total, 700);
        assert_eq!(token.fees_total_usd, Decimal::from_str("625.00").unwrap());
        assert!(token.created_at.is_some());
    }

    #[test]
    fn admin_categories_response_deserializes_metric_rows() {
        let response: AdminCategoriesResponse = serde_json::from_value(json!({
            "timestamp": 1_770_000_000_000i64,
            "sort_by": "volume_24h_usd",
            "sort_direction": "desc",
            "total": 1,
            "limit": 200,
            "has_more": false,
            "categories": [{
                "rank": 1,
                "category": "Crypto",
                "category_key": "crypto",
                "market_count": 18,
                "active_market_count": 14,
                "resolved_market_count": 4,
                "open_interest_usd": "350000.00",
                "volume_24h_usd": "115000.50",
                "volume_7d_usd": "700000.25",
                "volume_30d_usd": "2500000.00",
                "volume_total_usd": "4200000.00",
                "unique_traders_24h": 120,
                "unique_traders_7d": 550,
                "unique_traders_30d": 1300,
                "unique_traders_total": 2200,
                "fees_24h_usd": "57.50",
                "fees_7d_usd": "350.00",
                "fees_30d_usd": "1250.00",
                "fees_total_usd": "2100.00"
            }]
        }))
        .unwrap();

        assert_eq!(response.next_cursor, None);
        assert_eq!(response.categories.len(), 1);

        let category = &response.categories[0];
        assert_eq!(category.category_key, "crypto");
        assert_eq!(category.category_id, None);
        assert_eq!(category.active_market_count, 14);
        assert_eq!(
            category.volume_total_usd,
            Decimal::from_str("4200000.00").unwrap()
        );
        assert_eq!(category.unique_traders_total, 2200);
        assert!(category.created_at.is_none());
    }

    #[test]
    fn markets_to_settle_count_response_deserializes_count() {
        let response: MarketsToSettleCountResponse = serde_json::from_value(json!({
            "markets_to_settle_count": 3
        }))
        .unwrap();

        assert_eq!(response.markets_to_settle_count, 3);
    }

    #[test]
    fn markets_to_settle_query_serializes_cursor_and_limit() {
        let query = MarketsToSettleQuery {
            cursor: Some(123),
            limit: Some(200),
        };

        let query_string = serde_urlencoded::to_string(query).unwrap();
        assert_eq!(query_string, "cursor=123&limit=200");
    }

    #[test]
    fn markets_to_settle_response_deserializes_pagination_fields() {
        let response: MarketsToSettleResponse = serde_json::from_value(json!({
            "markets": [],
            "next_cursor": 456,
            "has_more": true
        }))
        .unwrap();

        assert!(response.markets.is_empty());
        assert_eq!(response.next_cursor, Some(456));
        assert!(response.has_more);
    }

    #[test]
    fn critical_log_errors_24h_count_response_deserializes_count() {
        let response: CriticalLogErrors24hCountResponse = serde_json::from_value(json!({
            "critical_log_errors_24h": 1
        }))
        .unwrap();

        assert_eq!(response.critical_log_errors_24h, 1);
    }

    #[test]
    fn admin_log_events_query_serializes_error_identity_filters() {
        let query = AdminLogEventsQuery {
            error_code: Some("MARKET_METADATA_FETCH_FAILED".to_string()),
            error_codes: Some(
                "MARKET_METADATA_FETCH_FAILED,METADATA_IMAGE_URL_MISSING".to_string(),
            ),
            rejection_code: Some("BROADCAST_FAILURE".to_string()),
            rejection_codes: Some("ORDER_NOT_FOUND,SELF_TRADE".to_string()),
            limit: Some(100),
            ..Default::default()
        };

        let query_string = serde_urlencoded::to_string(query).unwrap();
        assert_eq!(
            query_string,
            "error_code=MARKET_METADATA_FETCH_FAILED&error_codes=MARKET_METADATA_FETCH_FAILED%2CMETADATA_IMAGE_URL_MISSING&rejection_code=BROADCAST_FAILURE&rejection_codes=ORDER_NOT_FOUND%2CSELF_TRADE&limit=100"
        );
    }

    #[test]
    fn admin_log_events_query_serializes_plural_dimension_filters() {
        let query = AdminLogEventsQuery {
            service_names: Some("api,engine".to_string()),
            environments: Some("production,staging".to_string()),
            categories: Some("api_error,business_rejection".to_string()),
            severities: Some("error,critical".to_string()),
            components: Some("admin_handler,grpc".to_string()),
            operations: Some("get_market_metadata_admin,submit_order".to_string()),
            fingerprints: Some(
                "api_error|admin_handler|get_market_metadata_admin,business_rejection|grpc|submit_order|BROADCAST_FAILURE"
                    .to_string(),
            ),
            response_statuses: Some("error,rejected".to_string()),
            cursor: Some("opaque-cursor".to_string()),
            ..Default::default()
        };

        let query_string = serde_urlencoded::to_string(query).unwrap();
        assert_eq!(
            query_string,
            "service_names=api%2Cengine&environments=production%2Cstaging&categories=api_error%2Cbusiness_rejection&severities=error%2Ccritical&components=admin_handler%2Cgrpc&operations=get_market_metadata_admin%2Csubmit_order&fingerprints=api_error%7Cadmin_handler%7Cget_market_metadata_admin%2Cbusiness_rejection%7Cgrpc%7Csubmit_order%7CBROADCAST_FAILURE&response_statuses=error%2Crejected&cursor=opaque-cursor"
        );
    }

    #[test]
    fn admin_log_event_deserializes_error_and_rejection_codes() {
        let event: AdminLogEvent = serde_json::from_value(json!({
            "id": 991,
            "public_id": "LCERR_0198D0F3B07B7AA8A24F73DCD6C68E12",
            "service_name": "api",
            "environment": "production",
            "component": "admin_handler",
            "operation": "get_market_metadata_admin",
            "category": "api_error",
            "severity": "error",
            "occurred_at_ms": 1770000000000i64,
            "created_at_ms": 1770000000100i64,
            "user_visible": true,
            "request_id": "0198d0f3-b07b-7aa8-a24f-73dcd6c68e12",
            "http_status": 500,
            "message": "Failed to fetch market metadata",
            "fingerprint": "api_error|admin_handler|get_market_metadata_admin",
            "response_status": "error",
            "error_code": "MARKET_METADATA_FETCH_FAILED",
            "rejection_code": "BROADCAST_FAILURE",
            "context": {
                "rejection_code": "BROADCAST_FAILURE"
            }
        }))
        .unwrap();

        assert_eq!(
            event.error_code.as_deref(),
            Some("MARKET_METADATA_FETCH_FAILED")
        );
        assert_eq!(event.rejection_code.as_deref(), Some("BROADCAST_FAILURE"));
    }

    #[test]
    fn admin_log_event_deserializes_when_error_identity_fields_are_absent() {
        let event: AdminLogEvent = serde_json::from_value(json!({
            "id": 1002,
            "public_id": "LCERR_0198D0F4AA537D2C9C7D4BA640A91E20",
            "service_name": "engine",
            "environment": "production",
            "component": "grpc",
            "operation": "submit_order",
            "category": "business_rejection",
            "severity": "error",
            "occurred_at_ms": 1770000600000i64,
            "created_at_ms": 1770000600050i64,
            "user_visible": true,
            "message": "Order broadcast failed",
            "context": {}
        }))
        .unwrap();

        assert!(event.error_code.is_none());
        assert!(event.rejection_code.is_none());
    }

    #[test]
    fn deposit_token_metadata_payload_serializes_price_feed_and_min_order_fields() {
        let request = DepositTokenMetadataPayload {
            deposit_asset: "TOKEN_MINT".to_string(),
            display_name: None,
            symbol: None,
            token_symbol: None,
            description: None,
            icon_url_low: None,
            icon_url_medium: None,
            icon_url_high: None,
            metadata_uri: None,
            decimals: None,
            min_order_size: Some(1_000_000),
            binance_symbol: Some("BTCUSDT".to_string()),
            binance_enabled: Some(true),
            okx_inst_id: Some("BTC-USDT".to_string()),
        };

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "deposit_asset": "TOKEN_MINT",
                "min_order_size": 1_000_000,
                "binance_symbol": "BTCUSDT",
                "binance_enabled": true,
                "okx_inst_id": "BTC-USDT"
            })
        );
    }

    #[test]
    fn unified_metadata_response_reads_deposit_token_metadata_fields() {
        let response: UnifiedMetadataResponse = serde_json::from_value(json!({
            "deposit_tokens": [{
                "id": 1,
                "deposit_asset": "TOKEN_MINT",
                "display_name": "Bitcoin",
                "symbol": "BTC",
                "token_symbol": null,
                "binance_symbol": "BTCUSDT",
                "binance_enabled": true,
                "okx_inst_id": "BTC-USDT",
                "description": null,
                "icon_url_low": null,
                "icon_url_medium": null,
                "icon_url_high": null,
                "metadata_uri": null,
                "decimals": 8,
                "min_order_size": 100_000,
                "created_at": "2026-05-12T00:00:00Z",
                "updated_at": "2026-05-12T00:00:00Z"
            }]
        }))
        .unwrap();

        assert_eq!(response.deposit_tokens.len(), 1);
        let token = &response.deposit_tokens[0];
        assert_eq!(token.deposit_asset.as_str(), "TOKEN_MINT");
        assert_eq!(token.binance_symbol.as_deref(), Some("BTCUSDT"));
        assert!(token.binance_enabled);
        assert_eq!(token.okx_inst_id.as_deref(), Some("BTC-USDT"));
        assert_eq!(token.min_order_size, Some(100_000));
    }

    #[test]
    fn deposit_token_metadata_response_accepts_null_and_missing_min_order_size() {
        let with_null: DepositTokenMetadataResponse = serde_json::from_value(json!({
            "id": 1,
            "deposit_asset": "TOKEN_MINT",
            "display_name": "Bitcoin",
            "symbol": "BTC",
            "min_order_size": null,
            "created_at": "2026-05-12T00:00:00Z",
            "updated_at": "2026-05-12T00:00:00Z"
        }))
        .unwrap();
        assert_eq!(with_null.min_order_size, None);

        let with_missing: DepositTokenMetadataResponse = serde_json::from_value(json!({
            "id": 1,
            "deposit_asset": "TOKEN_MINT",
            "display_name": "Bitcoin",
            "symbol": "BTC",
            "created_at": "2026-05-12T00:00:00Z",
            "updated_at": "2026-05-12T00:00:00Z"
        }))
        .unwrap();
        assert_eq!(with_missing.min_order_size, None);
    }

    #[test]
    fn admin_market_metadata_response_reads_nested_rows_and_missing_metadata() {
        let response: AdminMarketMetadataResponse = serde_json::from_value(json!({
            "market": {
                "id": 1,
                "market_pubkey": "11111111111111111111111111111111",
                "market_id": 42,
                "num_outcomes": 2,
                "oracle": "11111111111111111111111111111111",
                "question_id": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "condition_id": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "bump": null,
                "market_status": "Active",
                "winning_outcome": null,
                "has_winning_outcome": false,
                "payout_numerators": null,
                "payout_denominator": null,
                "created_at": "2026-05-21T10:00:00Z",
                "activated_at": "2026-05-21T10:00:00Z",
                "settled_at": null,
                "updated_at": "2026-05-21T10:00:00Z"
            },
            "market_metadata": {
                "id": 10,
                "market_id": 42,
                "market_name": "Will BTC close above $100k?",
                "slug": "will-btc-close-above-100k",
                "description": "Market description",
                "definition": "Resolution rules",
                "category": "Crypto",
                "tags": ["btc", "crypto"],
                "featured_rank": 0,
                "resolution_by": 1770000000000i64,
                "created_at": "2026-05-21T10:00:00Z",
                "updated_at": "2026-05-21T10:00:00Z"
            },
            "deposit_assets": [{
                "id": 7,
                "market_id": 42,
                "market_pubkey": "11111111111111111111111111111111",
                "deposit_asset": "So11111111111111111111111111111111111111112",
                "vault": "11111111111111111111111111111111",
                "num_outcomes": 2,
                "created_at": "2026-05-21T10:00:00Z"
            }],
            "outcomes": [{
                "outcome_index": 0,
                "outcome_metadata": null,
                "conditional_tokens": [{
                    "conditional_mint": {
                        "id": 31,
                        "market_deposit_mint_id": 7,
                        "deposit_asset": "So11111111111111111111111111111111111111112",
                        "outcome_index": 0,
                        "token_address": "11111111111111111111111111111111",
                        "name": "YES SOL",
                        "symbol": "YES-SOL",
                        "uri": "https://metadata.example/token.json",
                        "created_at": "2026-05-21T10:00:00Z"
                    },
                    "conditional_token_metadata": null
                }]
            }],
            "missing_metadata": {
                "market_metadata": false,
                "outcomes": [0],
                "conditional_tokens": [31]
            }
        }))
        .unwrap();

        assert_eq!(response.market.market_id, 42);
        assert_eq!(response.market.bump, None);
        assert_eq!(
            response
                .market_metadata
                .as_ref()
                .and_then(|metadata| metadata.resolution_by),
            Some(1_770_000_000_000)
        );
        assert_eq!(response.deposit_assets[0].num_outcomes, 2);
        assert!(response.outcomes[0].outcome_metadata.is_none());
        assert!(response.outcomes[0].conditional_tokens[0]
            .conditional_token_metadata
            .is_none());
        assert_eq!(response.missing_metadata.outcomes, vec![0]);
        assert_eq!(response.missing_metadata.conditional_tokens, vec![31]);
    }

    #[test]
    fn update_market_metadata_request_serializes_path_scoped_body() {
        let request = UpdateMarketMetadataRequest {
            market: Some(
                UpdateMarketMetadataPayload {
                    market_name: Some("Updated market".to_string()),
                    tags: Some(vec!["btc".to_string(), "crypto".to_string()]),
                    ..Default::default()
                }
                .with_cleared_resolution_by(),
            ),
            outcomes: vec![UpdateOutcomeMetadataPayload {
                outcome_index: 0,
                name: Some("YES".to_string()),
                name_long: None,
                icon_url_low: None,
                icon_url_medium: None,
                icon_url_high: None,
                description: None,
                metadata_uri: None,
            }],
            conditional_tokens: vec![UpdateConditionalTokenMetadataPayload {
                conditional_mint_id: 31,
                short_symbol: Some("YES".to_string()),
                description: Some("YES token".to_string()),
                icon_url_low: None,
                icon_url_medium: None,
                icon_url_high: Some("https://cdn/token-high.webp".to_string()),
            }],
        };

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "market": {
                    "market_name": "Updated market",
                    "tags": ["btc", "crypto"],
                    "resolution_by": null
                },
                "outcomes": [{
                    "outcome_index": 0,
                    "name": "YES"
                }],
                "conditional_tokens": [{
                    "conditional_mint_id": 31,
                    "short_symbol": "YES",
                    "description": "YES token",
                    "icon_url_high": "https://cdn/token-high.webp"
                }]
            })
        );
        assert!(value["market"].get("market_id").is_none());
        let token = value["conditional_tokens"][0].as_object().unwrap();
        assert!(!token.contains_key("metadata_uri"));
        assert!(!token.contains_key("outcome"));
        assert!(!token.contains_key("outcome_index"));
        assert!(!token.contains_key("deposit_symbol"));
        assert!(!token.contains_key("decimals"));
    }

    #[test]
    fn conditional_token_metadata_payload_serializes_dashboard_writable_fields_only() {
        let request = ConditionalTokenMetadataPayload {
            conditional_mint_id: 31,
            short_symbol: Some("YES".to_string()),
            description: Some("YES token".to_string()),
            icon_url_low: Some("https://cdn/token-low.webp".to_string()),
            icon_url_medium: None,
            icon_url_high: Some("https://cdn/token-high.webp".to_string()),
        };

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "conditional_mint_id": 31,
                "short_symbol": "YES",
                "description": "YES token",
                "icon_url_low": "https://cdn/token-low.webp",
                "icon_url_high": "https://cdn/token-high.webp"
            })
        );
    }

    #[test]
    fn conditional_token_metadata_json_request_and_response_roundtrip() {
        let request = UpdateConditionalTokenMetadataJsonRequest {
            name: "YES USDC".to_string(),
            symbol: "YES-USDC".to_string(),
            description: Some("YES token".to_string()),
            image_data_url_low: None,
            image_data_url_medium: None,
            image_data_url_high: Some("data:image/webp;base64,high".to_string()),
        };

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "name": "YES USDC",
                "symbol": "YES-USDC",
                "description": "YES token",
                "image_data_url_high": "data:image/webp;base64,high"
            })
        );

        let response: ConditionalTokenMetadataJsonResponse = serde_json::from_value(json!({
            "conditional_mint": "11111111111111111111111111111111",
            "metadata_uri": "https://cdn/token.json",
            "image_url_low": null,
            "image_url_medium": "https://cdn/token-medium.webp",
            "image_url_high": "https://cdn/token-high.webp",
            "database_updated": true,
            "invalidation_paths": ["/metadata/token.json"]
        }))
        .unwrap();

        assert_eq!(
            response.conditional_mint.as_str(),
            "11111111111111111111111111111111"
        );
        assert_eq!(response.image_url_low, None);
        assert_eq!(
            response.image_url_high.as_deref(),
            Some("https://cdn/token-high.webp")
        );
        assert!(response.database_updated);
    }

    #[test]
    fn resync_conditional_token_derived_metadata_response_reads_database_rows() {
        let response: ResyncConditionalTokenDerivedMetadataResponse =
            serde_json::from_value(json!({
                "conditional_tokens": [{
                    "id": 99,
                    "conditional_mint_id": 31,
                    "outcome_index": 0,
                    "display_name": "YES USDC",
                    "outcome": "YES",
                    "symbol": "YES-USDC",
                    "deposit_symbol": "USDC",
                    "short_name": "YES",
                    "description": "YES token",
                    "icon_url_low": null,
                    "icon_url_medium": null,
                    "icon_url_high": "https://cdn/token-high.webp",
                    "metadata_uri": "https://cdn/token.json",
                    "decimals": 6,
                    "created_at": "2026-05-21T10:00:00Z",
                    "updated_at": "2026-05-21T10:00:00Z"
                }]
            }))
            .unwrap();

        let token = &response.conditional_tokens[0];
        assert_eq!(token.conditional_mint_id, 31);
        assert_eq!(token.short_name.as_deref(), Some("YES"));
        assert_eq!(token.deposit_symbol.as_deref(), Some("USDC"));
        assert_eq!(token.decimals, Some(6));
    }

    #[test]
    fn metadata_image_requests_and_response_use_variant_triplets() {
        let variants = AdminImageVariants {
            low: "data:image/webp;base64,low".to_string(),
            medium: "data:image/webp;base64,medium".to_string(),
            high: "data:image/webp;base64,high".to_string(),
        };
        let request = UpdateMarketImagesRequest {
            market_icon: Some(variants.clone()),
            market_banner: None,
            outcomes: vec![UpdateOutcomeImageRequest {
                outcome_index: 0,
                icon: variants.clone(),
            }],
            conditional_tokens: vec![UpdateConditionalTokenImageRequest {
                conditional_mint_id: 31,
                icon: variants.clone(),
            }],
        };

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "market_icon": {
                    "low": "data:image/webp;base64,low",
                    "medium": "data:image/webp;base64,medium",
                    "high": "data:image/webp;base64,high"
                },
                "outcomes": [{
                    "outcome_index": 0,
                    "icon": {
                        "low": "data:image/webp;base64,low",
                        "medium": "data:image/webp;base64,medium",
                        "high": "data:image/webp;base64,high"
                    }
                }],
                "conditional_tokens": [{
                    "conditional_mint_id": 31,
                    "icon": {
                        "low": "data:image/webp;base64,low",
                        "medium": "data:image/webp;base64,medium",
                        "high": "data:image/webp;base64,high"
                    }
                }]
            })
        );

        let response: MetadataImageUpdateResponse = serde_json::from_value(json!({
            "updated": [{
                "target_type": "deposit_token_icon",
                "outcome_index": null,
                "conditional_mint_id": null,
                "conditional_mint": null,
                "deposit_asset": "So11111111111111111111111111111111111111112",
                "urls": {
                    "low": "https://cdn/low.webp",
                    "medium": "https://cdn/medium.webp",
                    "high": "https://cdn/high.webp"
                }
            }],
            "database_updated": false,
            "invalidation_paths": ["/metadata/deposit-tokens/low.webp"]
        }))
        .unwrap();

        assert_eq!(
            response.updated[0].target_type,
            MetadataImageTargetType::DepositTokenIcon
        );
        assert_eq!(
            response.updated[0]
                .deposit_asset
                .as_ref()
                .map(|mint| mint.as_str()),
            Some("So11111111111111111111111111111111111111112")
        );
        assert!(!response.database_updated);
        assert_eq!(response.invalidation_paths.len(), 1);
    }

    #[test]
    fn deposit_token_metadata_list_response_deserializes_rows() {
        let response: AdminDepositTokenMetadataListResponse = serde_json::from_value(json!({
            "deposit_tokens": [{
                "id": 50,
                "deposit_asset": "So11111111111111111111111111111111111111112",
                "display_name": "Solana",
                "symbol": "SOL",
                "token_symbol": "SOL",
                "binance_symbol": "SOLUSDT",
                "okx_inst_id": "SOL-USDT",
                "description": "Solana deposit token",
                "icon_url_low": "https://cdn/sol-low.webp",
                "icon_url_medium": "https://cdn/sol-medium.webp",
                "icon_url_high": "https://cdn/sol-high.webp",
                "metadata_uri": "https://cdn/sol.json",
                "decimals": 9,
                "min_order_size": 1000000,
                "created_at": "2026-05-21T10:00:00Z",
                "updated_at": "2026-05-21T10:00:00Z"
            }]
        }))
        .unwrap();

        assert_eq!(response.deposit_tokens.len(), 1);
        assert_eq!(response.deposit_tokens[0].symbol, "SOL");
        assert!(!response.deposit_tokens[0].binance_enabled);
    }

    #[test]
    fn focused_deposit_token_metadata_types_roundtrip() {
        let response: AdminDepositTokenMetadataResponse = serde_json::from_value(json!({
            "deposit_asset": "So11111111111111111111111111111111111111112",
            "deposit_token_metadata": {
                "id": 50,
                "deposit_asset": "So11111111111111111111111111111111111111112",
                "display_name": "Solana",
                "symbol": "SOL",
                "token_symbol": "SOL",
                "binance_symbol": "SOLUSDT",
                "okx_inst_id": "SOL-USDT",
                "description": "Solana deposit token",
                "icon_url_low": "https://cdn/sol-low.webp",
                "icon_url_medium": "https://cdn/sol-medium.webp",
                "icon_url_high": "https://cdn/sol-high.webp",
                "metadata_uri": "https://cdn/sol.json",
                "decimals": 9,
                "min_order_size": 1000000,
                "created_at": "2026-05-21T10:00:00Z",
                "updated_at": "2026-05-21T10:00:00Z"
            }
        }))
        .unwrap();

        assert_eq!(response.deposit_token_metadata.symbol, "SOL");
        assert!(!response.deposit_token_metadata.binance_enabled);

        let request = UpdateDepositTokenMetadataRequest {
            display_name: Some("Solana".to_string()),
            symbol: Some("SOL".to_string()),
            token_symbol: Some("SOL".to_string()),
            min_order_size: Some(1_000_000),
            okx_inst_id: Some("SOL-USDT".to_string()),
            ..Default::default()
        };
        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "display_name": "Solana",
                "symbol": "SOL",
                "token_symbol": "SOL",
                "min_order_size": 1_000_000,
                "okx_inst_id": "SOL-USDT"
            })
        );
        assert!(value.get("deposit_asset").is_none());

        let images = UpdateDepositTokenImagesRequest {
            icon: AdminImageVariants {
                low: "data:image/webp;base64,low".to_string(),
                medium: "data:image/webp;base64,medium".to_string(),
                high: "data:image/webp;base64,high".to_string(),
            },
        };
        assert_eq!(
            serde_json::to_value(images).unwrap(),
            json!({
                "icon": {
                    "low": "data:image/webp;base64,low",
                    "medium": "data:image/webp;base64,medium",
                    "high": "data:image/webp;base64,high"
                }
            })
        );
    }

    #[test]
    fn upload_deposit_token_images_request_and_response_roundtrip() {
        let request = UploadDepositTokenImagesRequest {
            icon: AdminImageVariants {
                low: "data:image/webp;base64,low".to_string(),
                medium: "data:image/webp;base64,medium".to_string(),
                high: "data:image/webp;base64,high".to_string(),
            },
        };

        let value = serde_json::to_value(request).unwrap();
        assert_eq!(
            value,
            json!({
                "icon": {
                    "low": "data:image/webp;base64,low",
                    "medium": "data:image/webp;base64,medium",
                    "high": "data:image/webp;base64,high"
                }
            })
        );

        let response: UploadDepositTokenImagesResponse = serde_json::from_value(json!({
            "deposit_asset": "4o5Vsd7iPu97qkKypojXDbpu8BR3t5poD8ThGo8hnUKy",
            "icon_url_low": "https://cdn.example/metadata/deposit-tokens/4o5Vsd7iPu97qkKypojXDbpu8BR3t5poD8ThGo8hnUKy/icon-low.webp",
            "icon_url_medium": "https://cdn.example/metadata/deposit-tokens/4o5Vsd7iPu97qkKypojXDbpu8BR3t5poD8ThGo8hnUKy/icon-medium.webp",
            "icon_url_high": "https://cdn.example/metadata/deposit-tokens/4o5Vsd7iPu97qkKypojXDbpu8BR3t5poD8ThGo8hnUKy/icon-high.webp",
            "database_updated": false,
            "invalidation_paths": [
                "/metadata/deposit-tokens/4o5Vsd7iPu97qkKypojXDbpu8BR3t5poD8ThGo8hnUKy/icon-low.webp",
                "/metadata/deposit-tokens/4o5Vsd7iPu97qkKypojXDbpu8BR3t5poD8ThGo8hnUKy/icon-medium.webp",
                "/metadata/deposit-tokens/4o5Vsd7iPu97qkKypojXDbpu8BR3t5poD8ThGo8hnUKy/icon-high.webp"
            ]
        }))
        .unwrap();

        assert_eq!(
            response.deposit_asset.as_str(),
            "4o5Vsd7iPu97qkKypojXDbpu8BR3t5poD8ThGo8hnUKy"
        );
        assert_eq!(
            response.icon_url_medium,
            "https://cdn.example/metadata/deposit-tokens/4o5Vsd7iPu97qkKypojXDbpu8BR3t5poD8ThGo8hnUKy/icon-medium.webp"
        );
        assert!(!response.database_updated);
        assert_eq!(response.invalidation_paths.len(), 3);
    }

    #[test]
    fn upload_market_deployment_assets_request_uses_quality_specific_upload_fields() {
        let request: UploadMarketDeploymentAssetsRequest = serde_json::from_value(json!({
            "market_id": 7,
            "market_pubkey": "market-pubkey",
            "market": {
                "name": "Market",
                "slug": "market",
                "banner_image_data_url_high": "data:image/webp;base64,banner-high",
                "banner_image_content_type_high": "image/webp",
                "icon_image_data_url_low": "data:image/webp;base64,icon-low",
                "icon_image_content_type_low": "image/webp",
                "icon_image_data_url_high": "data:image/webp;base64,icon-high",
                "icon_image_content_type_high": "image/webp"
            },
            "outcomes": [{
                "index": 0,
                "name": "Yes",
                "symbol": "YES",
                "icon_image_data_url_high": "data:image/webp;base64,outcome-high",
                "icon_image_content_type_high": "image/webp"
            }],
            "conditional_tokens": [{
                "outcome_index": 0,
                "deposit_mint": "deposit-mint",
                "conditional_mint": "conditional-mint",
                "name": "Yes USDC",
                "symbol": "YES-USDC",
                "image_data_url_low": "data:image/webp;base64,token-low",
                "image_content_type_low": "image/webp",
                "image_data_url_high": "data:image/webp;base64,token-high",
                "image_content_type_high": "image/webp"
            }]
        }))
        .unwrap();

        let value = serde_json::to_value(request).unwrap();
        let market = value["market"].as_object().unwrap();
        assert_eq!(
            market
                .get("banner_image_data_url_high")
                .and_then(Value::as_str),
            Some("data:image/webp;base64,banner-high")
        );
        assert_eq!(
            market
                .get("icon_image_data_url_low")
                .and_then(Value::as_str),
            Some("data:image/webp;base64,icon-low")
        );
        assert!(!market.contains_key("banner_image_data_url"));
        assert!(!market.contains_key("icon_image_data_url"));

        let outcome = value["outcomes"][0].as_object().unwrap();
        assert_eq!(
            outcome
                .get("icon_image_content_type_high")
                .and_then(Value::as_str),
            Some("image/webp")
        );
        assert!(!outcome.contains_key("icon_image_content_type"));

        let token = value["conditional_tokens"][0].as_object().unwrap();
        assert_eq!(
            token.get("image_data_url_high").and_then(Value::as_str),
            Some("data:image/webp;base64,token-high")
        );
        assert!(!token.contains_key("image_data_url"));
        assert!(!token.contains_key("image_content_type"));
    }

    #[test]
    fn upload_market_deployment_assets_response_reads_variant_token_urls() {
        let response: UploadMarketDeploymentAssetsResponse = serde_json::from_value(json!({
            "market_metadata_uri": "s3://metadata/market.json",
            "market": {
                "banner_image_url_low": "https://cdn/banner-low.webp",
                "banner_image_url_medium": "https://cdn/banner-medium.webp",
                "banner_image_url_high": "https://cdn/banner-high.webp"
            },
            "outcomes": [{
                "index": 0,
                "icon_url_high": "https://cdn/outcome-high.webp"
            }],
            "deposit_assets": [{
                "mint": "deposit-mint",
                "icon_url_high": "https://cdn/deposit-high.webp"
            }],
            "tokens": [{
                "conditional_mint": "conditional-mint",
                "metadata_uri": "s3://metadata/token.json",
                "image_url_low": "https://cdn/token-low.webp",
                "image_url_medium": "https://cdn/token-medium.webp",
                "image_url_high": "https://cdn/token-high.webp"
            }]
        }))
        .unwrap();

        assert_eq!(response.deposit_assets[0].mint, "deposit-mint");
        assert_eq!(
            response.tokens[0].image_url_high.as_deref(),
            Some("https://cdn/token-high.webp")
        );

        let token = serde_json::to_value(&response.tokens[0]).unwrap();
        assert!(token.get("image_url").is_none());
    }
}
