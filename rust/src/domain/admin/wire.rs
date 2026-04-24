//! Wire types for admin requests and responses.

use crate::shared::{OrderBookId, PubkeyStr};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Serializer};

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
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_status: Option<String>,
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
/// When a `*_image_data_url` + `*_image_content_type` pair is provided the
/// backend uploads the image and ignores the matching `*_image_url` field;
/// otherwise the existing `*_image_url` is preserved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDeploymentMarket {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub definition: Option<String>,
    #[serde(default)]
    pub banner_image_url: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub subcategory: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub featured_rank: Option<i32>,
    #[serde(default)]
    pub banner_image_data_url: Option<String>,
    #[serde(default)]
    pub banner_image_content_type: Option<String>,
    #[serde(default)]
    pub icon_image_data_url: Option<String>,
    #[serde(default)]
    pub icon_image_content_type: Option<String>,
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
    pub icon_url: Option<String>,
    #[serde(default)]
    pub icon_image_data_url: Option<String>,
    #[serde(default)]
    pub icon_image_content_type: Option<String>,
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
    pub icon_url: Option<String>,
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
    pub image_data_url: String,
    pub image_content_type: String,
}

/// Response from `POST /api/admin/metadata/upload-market-deployment-assets`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadMarketDeploymentAssetsResponse {
    pub market_metadata_uri: String,
    pub market: UploadedMarketImages,
    #[serde(default)]
    pub outcomes: Vec<UploadedOutcomeImages>,
    #[serde(default)]
    pub tokens: Vec<UploadedConditionalToken>,
}

/// Uploaded market banner/icon URLs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UploadedMarketImages {
    #[serde(default)]
    pub banner_image_url: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
}

/// Uploaded icon URL for a single outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedOutcomeImages {
    pub index: i32,
    #[serde(default)]
    pub icon_url: Option<String>,
}

/// Uploaded conditional token image + metadata URIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadedConditionalToken {
    pub conditional_mint: String,
    pub image_url: String,
    pub metadata_uri: String,
}
