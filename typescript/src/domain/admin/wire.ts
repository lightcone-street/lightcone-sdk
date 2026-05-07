export interface UnifiedMetadataRequest {
  markets?: MarketMetadataPayload[];
  outcomes?: OutcomeMetadataPayload[];
  conditional_tokens?: ConditionalTokenMetadataPayload[];
  deposit_tokens?: DepositTokenMetadataPayload[];
}

export interface MarketMetadataPayload {
  market_id: number;
  market_name?: string;
  slug?: string;
  description?: string;
  definition?: string;
  banner_image_url_low?: string;
  banner_image_url_medium?: string;
  banner_image_url_high?: string;
  icon_url_low?: string;
  icon_url_medium?: string;
  icon_url_high?: string;
  category?: string;
  subcategory?: string;
  tags?: string[];
  featured_rank?: number;
  metadata_uri?: string;
  s3_synced?: boolean;
  s3_synced_at?: string;
  s3_error?: string;
}

export interface OutcomeMetadataPayload {
  market_id: number;
  outcome_index: number;
  name?: string;
  icon_url_low?: string;
  icon_url_medium?: string;
  icon_url_high?: string;
  description?: string;
  metadata_uri?: string;
  s3_synced?: boolean;
  s3_synced_at?: string;
  s3_error?: string;
}

export interface ConditionalTokenMetadataPayload {
  conditional_mint_id: number;
  outcome_index?: number;
  outcome?: string;
  deposit_symbol?: string;
  short_symbol?: string;
  description?: string;
  icon_url_low?: string;
  icon_url_medium?: string;
  icon_url_high?: string;
  metadata_uri?: string;
  decimals?: number;
  s3_synced?: boolean;
  s3_synced_at?: string;
  s3_error?: string;
}

export interface DepositTokenMetadataPayload {
  deposit_asset: string;
  display_name?: string;
  symbol?: string;
  token_symbol?: string;
  description?: string;
  icon_url_low?: string;
  icon_url_medium?: string;
  icon_url_high?: string;
  metadata_uri?: string;
  decimals?: number;
  s3_synced?: boolean;
  s3_synced_at?: string;
  s3_error?: string;
  binance_symbol?: string;
  binance_enabled?: boolean;
}

export interface MarketMetadataResponse extends MarketMetadataPayload {
  id: number;
  created_at: string;
  updated_at: string;
}

export interface OutcomeMetadataResponse extends OutcomeMetadataPayload {
  id: number;
  created_at: string;
  updated_at: string;
}

export interface ConditionalTokenMetadataResponse extends ConditionalTokenMetadataPayload {
  id: number;
  created_at: string;
  updated_at: string;
}

export interface DepositTokenMetadataResponse {
  id: number;
  deposit_asset: string;
  display_name: string;
  symbol: string;
  token_symbol: string | null;
  binance_symbol: string | null;
  binance_enabled: boolean;
  description: string | null;
  icon_url_low: string | null;
  icon_url_medium: string | null;
  icon_url_high: string | null;
  metadata_uri: string | null;
  decimals: number | null;
  s3_synced: boolean;
  s3_synced_at: string | null;
  s3_error: string | null;
  created_at: string;
  updated_at: string;
}

export interface UnifiedMetadataResponse {
  markets?: MarketMetadataResponse[];
  outcomes?: OutcomeMetadataResponse[];
  conditional_tokens?: ConditionalTokenMetadataResponse[];
  deposit_tokens?: DepositTokenMetadataResponse[];
}

export type TargetSpec =
  | "all"
  | { user_id: string }
  | { wallet_address: string }
  | { code: string }
  | { batch_id: string };

export interface AllocateCodesRequest {
  target: TargetSpec;
  batch_id?: string;
  vanity_codes?: string[];
  count?: number;
  max_uses?: number;
}

export interface AllocateCodesResponse {
  status: string;
  users_count?: number;
  codes_allocated?: number;
  user_id?: string;
  codes?: string[];
}

export interface WhitelistRequest {
  wallet_addresses: string[];
  allocate_codes?: boolean;
}

export interface WhitelistResponse {
  status: string;
  wallets_added: number;
  codes_allocated: number;
}

export interface RevokeRequest {
  target: TargetSpec;
  reason?: string;
}

export interface RevokeResponse {
  revoked_count: number;
  user_ids: string[];
}

export interface UnrevokeRequest {
  target: TargetSpec;
}

export interface UnrevokeResponse {
  restored_count: number;
  user_ids: string[];
}

export interface CreateNotificationRequest {
  title: string;
  message: string;
  expires_at?: string;
}

export interface CreateNotificationResponse {
  status: string;
}

export interface DismissNotificationRequest {
  notification_id: string;
}

export interface DismissNotificationResponse {
  status: string;
}

// ============================================================================
// Referral config / codes
// ============================================================================

/** Response of `POST /api/admin/referral/config/get` and `/update`. */
export interface ReferralConfig {
  default_code_count: number;
  /** RFC-3339 timestamp. */
  updated_at: string;
}

/** Request for `POST /api/admin/referral/config/update`. */
export interface UpdateConfigRequest {
  default_code_count?: number;
}

/** Request for `POST /api/admin/referral/codes` (admin list). */
export interface ListCodesRequest {
  limit: number;
  offset: number;
  owner_user_id?: string;
  batch_id?: string;
  code?: string;
}

/** A single referral code returned from the admin list endpoint. */
export interface CodeListEntry {
  code: string;
  owner_user_id: string;
  batch_id: string;
  is_vanity: boolean;
  max_uses: number;
  use_count: number;
  /** RFC-3339 timestamp. */
  created_at: string;
}

export interface ListCodesResponse {
  codes: CodeListEntry[];
  count: number;
}

/** Request for `POST /api/admin/referral/codes/update`. */
export interface UpdateCodeRequest {
  code: string;
  max_uses: number;
}

export interface UpdateCodeResponse {
  status: string;
  code: string;
  max_uses: number;
}

// ============================================================================
// Admin logs
// ============================================================================

/** Filter set for `GET /api/admin/logs/events`. All fields optional. */
export interface AdminLogEventsQuery {
  from_ms?: number;
  to_ms?: number;
  service_name?: string;
  environment?: string;
  category?: string;
  severity?: string;
  component?: string;
  operation?: string;
  fingerprint?: string;
  response_status?: string;
  user_visible?: boolean;
  request_id?: string;
  user_pubkey?: string;
  market_pubkey?: string;
  orderbook_id?: string;
  order_hash?: string;
  trigger_order_id?: string;
  tx_signature?: string;
  checkpoint_signature?: string;
  limit?: number;
  cursor?: string;
}

export interface AdminLogEvent {
  id: number;
  public_id: string;
  service_name: string;
  environment: string;
  component: string;
  operation: string;
  category: string;
  severity: string;
  occurred_at_ms: number;
  created_at_ms: number;
  user_visible: boolean;
  message: string;
  /** Arbitrary structured context. */
  context: unknown;
  occurred_at?: string;
  created_at?: string;
  request_id?: string;
  user_pubkey?: string;
  market_pubkey?: string;
  orderbook_id?: string;
  order_hash?: string;
  trigger_order_id?: string;
  tx_signature?: string;
  checkpoint_signature?: string;
  http_status?: number;
  grpc_code?: string;
  fingerprint?: string;
  response_status?: string;
}

export interface AdminLogEventsResponse {
  events: AdminLogEvent[];
  limit: number;
  next_cursor?: string;
}

/** Query for `GET /api/admin/logs/metrics`. */
export interface AdminLogMetricsQuery {
  /** CSV (e.g. `"1h,24h"`). */
  windows?: string;
  /** CSV (e.g. `"service,component"`). */
  scopes?: string;
  limit_per_scope?: number;
}

export interface AdminLogMetricSummary {
  scope_key: string;
  total_count: number;
  error_count: number;
  critical_count: number;
  user_visible_count: number;
  computed_at_ms: number;
  computed_at?: string;
}

export interface AdminLogMetricBreakdown {
  window: string;
  scope: string;
  rows: AdminLogMetricSummary[];
}

export interface AdminLogMetricsResponse {
  computed_at_ms: number;
  breakdowns: AdminLogMetricBreakdown[];
  computed_at?: string;
}

/** Query for `GET /api/admin/logs/metrics/history`. */
export interface AdminLogMetricHistoryQuery {
  scope: string;
  scope_key?: string;
  /** Defaults to `"1h"` on the backend. */
  resolution: string;
  from_ms?: number;
  to_ms?: number;
  limit?: number;
}

export interface AdminLogMetricPoint {
  bucket_start_ms: number;
  total_count: number;
  error_count: number;
  critical_count: number;
  user_visible_count: number;
  bucket_start?: string;
}

export interface AdminLogMetricHistoryResponse {
  scope: string;
  scope_key: string;
  resolution: string;
  from_ms: number;
  to_ms: number;
  points: AdminLogMetricPoint[];
  from?: string;
  to?: string;
}

// ============================================================================
// Market deployment asset upload
// ============================================================================

/** Request for `POST /api/admin/metadata/upload-market-deployment-assets`. */
export interface UploadMarketDeploymentAssetsRequest {
  market_id: number;
  market_pubkey: string;
  market: MarketDeploymentMarket;
  outcomes?: MarketDeploymentOutcome[];
  deposit_assets?: MarketDeploymentDepositAsset[];
  conditional_tokens?: MarketDeploymentConditionalToken[];
}

/**
 * Market-level fields for a deployment asset upload.
 *
 * Image uploads are quality-specific WebP data URLs. Hosted URL fields are
 * preserved separately and are used when no matching data URL is supplied.
 */
export interface MarketDeploymentMarket {
  name: string;
  slug: string;
  description?: string;
  definition?: string;
  banner_image_url_low?: string;
  banner_image_url_medium?: string;
  banner_image_url_high?: string;
  icon_url_low?: string;
  icon_url_medium?: string;
  icon_url_high?: string;
  category?: string;
  subcategory?: string;
  tags?: string[];
  featured_rank?: number;
  banner_image_data_url_low?: string;
  banner_image_content_type_low?: string;
  banner_image_data_url_medium?: string;
  banner_image_content_type_medium?: string;
  banner_image_data_url_high?: string;
  banner_image_content_type_high?: string;
  icon_image_data_url_low?: string;
  icon_image_content_type_low?: string;
  icon_image_data_url_medium?: string;
  icon_image_content_type_medium?: string;
  icon_image_data_url_high?: string;
  icon_image_content_type_high?: string;
}

export interface MarketDeploymentOutcome {
  index: number;
  name: string;
  symbol: string;
  description?: string;
  icon_url_low?: string;
  icon_url_medium?: string;
  icon_url_high?: string;
  icon_image_data_url_low?: string;
  icon_image_content_type_low?: string;
  icon_image_data_url_medium?: string;
  icon_image_content_type_medium?: string;
  icon_image_data_url_high?: string;
  icon_image_content_type_high?: string;
}

export interface MarketDeploymentDepositAsset {
  mint: string;
  display_name: string;
  symbol: string;
  description?: string;
  icon_url_low?: string;
  icon_url_medium?: string;
  icon_url_high?: string;
  decimals: number;
}

export interface MarketDeploymentConditionalToken {
  outcome_index: number;
  deposit_mint: string;
  conditional_mint: string;
  name: string;
  symbol: string;
  description?: string;
  image_data_url_low?: string;
  image_content_type_low?: string;
  image_data_url_medium?: string;
  image_content_type_medium?: string;
  image_data_url_high: string;
  image_content_type_high: string;
}

export interface UploadMarketDeploymentAssetsResponse {
  market_metadata_uri: string;
  market: UploadedMarketImages;
  outcomes: UploadedOutcomeImages[];
  deposit_assets: UploadedDepositAssetImages[];
  tokens: UploadedConditionalToken[];
}

export interface UploadedMarketImages {
  banner_image_url_low?: string;
  banner_image_url_medium?: string;
  banner_image_url_high?: string;
  icon_url_low?: string;
  icon_url_medium?: string;
  icon_url_high?: string;
}

export interface UploadedOutcomeImages {
  index: number;
  icon_url_low?: string;
  icon_url_medium?: string;
  icon_url_high?: string;
}

export interface UploadedDepositAssetImages {
  mint: string;
  icon_url_low?: string;
  icon_url_medium?: string;
  icon_url_high?: string;
}

export interface UploadedConditionalToken {
  conditional_mint: string;
  metadata_uri: string;
  image_url_low?: string;
  image_url_medium?: string;
  image_url_high?: string;
}
