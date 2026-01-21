/**
 * Market-related types for the Lightcone REST API.
 */

/**
 * Market status enum matching the API specification.
 */
export type ApiMarketStatus = "Pending" | "Active" | "Settled";

/**
 * Outcome information for a market.
 */
export interface Outcome {
  /** Outcome index (0-based) */
  index: number;
  /** Outcome name */
  name: string;
  /** Optional thumbnail URL */
  thumbnail_url?: string;
}

/**
 * Orderbook summary embedded in market response.
 */
export interface OrderbookSummary {
  /** Orderbook identifier */
  orderbook_id: string;
  /** Market pubkey */
  market_pubkey: string;
  /** Base token address */
  base_token: string;
  /** Quote token address */
  quote_token: string;
  /** Tick size for price granularity */
  tick_size: number;
  /** Creation timestamp */
  created_at: string;
}

/**
 * Conditional token information.
 */
export interface ConditionalToken {
  /** Database ID */
  id: number;
  /** Outcome index this token represents */
  outcome_index: number;
  /** Token mint address */
  token_address: string;
  /** Token name */
  name: string;
  /** Token symbol */
  symbol: string;
  /** Token metadata URI */
  uri?: string;
  /** Display name for UI */
  display_name: string;
  /** Outcome name */
  outcome: string;
  /** Associated deposit symbol */
  deposit_symbol: string;
  /** Short name for display */
  short_name: string;
  /** Token description */
  description?: string;
  /** Icon URL */
  icon_url?: string;
  /** Metadata URI */
  metadata_uri?: string;
  /** Token decimals */
  decimals: number;
  /** Creation timestamp */
  created_at: string;
}

/**
 * Deposit asset information.
 */
export interface DepositAsset {
  /** Display name for the asset */
  display_name: string;
  /** Token symbol */
  token_symbol: string;
  /** Short symbol */
  symbol: string;
  /** Deposit asset mint address */
  deposit_asset: string;
  /** Database ID */
  id: number;
  /** Associated market pubkey */
  market_pubkey: string;
  /** Vault address */
  vault: string;
  /** Number of outcomes */
  num_outcomes: number;
  /** Asset description */
  description?: string;
  /** Icon URL */
  icon_url?: string;
  /** Metadata URI */
  metadata_uri?: string;
  /** Token decimals */
  decimals: number;
  /** Conditional tokens for each outcome */
  conditional_tokens: ConditionalToken[];
  /** Creation timestamp */
  created_at: string;
}

/**
 * Market information.
 */
export interface Market {
  /** Market name */
  market_name: string;
  /** URL-friendly slug */
  slug: string;
  /** Market description */
  description: string;
  /** Market definition/rules */
  definition: string;
  /** Possible outcomes */
  outcomes: Outcome[];
  /** Banner image URL */
  banner_image_url?: string;
  /** Thumbnail URL */
  thumbnail_url?: string;
  /** Market category */
  category?: string;
  /** Tags for filtering */
  tags: string[];
  /** Featured rank (0 = not featured) */
  featured_rank: number;
  /** Market PDA address */
  market_pubkey: string;
  /** Market ID */
  market_id: number;
  /** Oracle address */
  oracle: string;
  /** Question ID */
  question_id: string;
  /** Condition ID */
  condition_id: string;
  /** Current market status */
  market_status: ApiMarketStatus;
  /** Winning outcome index (if settled) */
  winning_outcome: number;
  /** Whether market has a winning outcome */
  has_winning_outcome: boolean;
  /** Creation timestamp */
  created_at: string;
  /** Activation timestamp */
  activated_at?: string;
  /** Settlement timestamp */
  settled_at?: string;
  /** Deposit assets for this market */
  deposit_assets: DepositAsset[];
  /** Orderbooks for this market */
  orderbooks: OrderbookSummary[];
}

/**
 * Response for GET /api/markets.
 */
export interface MarketsResponse {
  /** List of markets */
  markets: Market[];
  /** Total count */
  total: number;
}

/**
 * Response for GET /api/markets/{market_pubkey}.
 */
export interface MarketInfoResponse {
  /** Market details */
  market: Market;
  /** Deposit assets */
  deposit_assets: DepositAsset[];
  /** Count of deposit assets */
  deposit_asset_count: number;
}

/**
 * Response for GET /api/markets/{market_pubkey}/deposit-assets.
 */
export interface DepositAssetsResponse {
  /** Market pubkey */
  market_pubkey: string;
  /** Deposit assets */
  deposit_assets: DepositAsset[];
  /** Total count */
  total: number;
}
