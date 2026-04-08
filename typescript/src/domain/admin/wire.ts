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
  banner_image_url?: string;
  icon_url?: string;
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
  icon_url?: string;
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
  icon_url?: string;
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
  icon_url?: string;
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
  icon_url: string | null;
  metadata_uri: string | null;
  decimals: number | null;
  s3_synced: boolean;
  s3_synced_at: string | null;
  s3_error: string | null;
  created_at: string;
  updated_at: string;
}

export interface UnifiedMetadataResponse {
  status: string;
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
