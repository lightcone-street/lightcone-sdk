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
  display_name?: string;
  outcome?: string;
  deposit_symbol?: string;
  short_name?: string;
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
}

export interface UnifiedMetadataResponse {
  status: string;
  markets?: unknown[];
  outcomes?: unknown[];
  conditional_tokens?: unknown[];
  deposit_tokens?: unknown[];
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
