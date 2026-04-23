import type { OrderBookId, PubkeyStr } from "../../shared";
import type { Status } from "./index";

export interface OutcomeResponse {
  index: number;
  name: string;
  icon_url?: string;
}

export interface ConditionalTokenResponse {
  id: number;
  outcome_index: number;
  token_address: string;
  symbol?: string;
  uri?: string;
  outcome?: string;
  deposit_symbol?: string;
  short_symbol?: string;
  description?: string;
  icon_url?: string;
  metadata_uri?: string;
  decimals?: number;
  created_at: string;
}

export interface DepositAssetResponse {
  display_name?: string;
  token_symbol?: string;
  symbol?: string;
  deposit_asset: string;
  id: number;
  market_pubkey: string;
  vault: string;
  num_outcomes: number;
  description?: string;
  icon_url?: string;
  metadata_uri?: string;
  decimals?: number;
  conditional_mints: ConditionalTokenResponse[];
  created_at: string;
}

export interface MarketResponse {
  market_name?: string;
  slug?: string;
  description?: string;
  definition?: string;
  outcomes: OutcomeResponse[];
  banner_image_url?: string;
  icon_url?: string;
  category?: string;
  tags?: string[];
  featured_rank?: number;
  market_pubkey: string;
  market_id: number;
  oracle: string;
  question_id: string;
  condition_id: string;
  market_status: string;
  winning_outcome?: number;
  has_winning_outcome: boolean;
  created_at: string;
  activated_at?: string;
  settled_at?: string;
  deposit_assets: DepositAssetResponse[];
  orderbooks: import("../orderbook/wire").OrderbookResponse[];
}

export interface MarketsResponse {
  markets: MarketResponse[];
  next_cursor?: number;
  has_more?: boolean;
}

export interface SingleMarketResponse {
  market: MarketResponse;
}

export interface SearchOrderbook {
  orderbook_id: OrderBookId;
  outcome_name: string;
  outcome_index: number;
  deposit_base_asset: PubkeyStr;
  deposit_quote_asset: PubkeyStr;
  deposit_base_symbol: string;
  deposit_quote_symbol: string;
  base_icon_url: string;
  quote_icon_url: string;
  conditional_base_mint: PubkeyStr;
  conditional_quote_mint: PubkeyStr;
  latest_mid_price?: string;
}

export interface MarketSearchResult {
  slug: string;
  market_name: string;
  market_status: Status;
  category?: string;
  tags: string[];
  featured_rank: number;
  description?: string;
  icon_url?: string;
  orderbooks: SearchOrderbook[];
}

export type MarketEvent =
  | { event_type: "settled"; market_pubkey: string }
  | { event_type: "created"; market_pubkey: string }
  | { event_type: "opened"; market_pubkey: string }
  | { event_type: "paused"; market_pubkey: string }
  | { event_type: "orderbook_created"; market_pubkey: string; orderbook_id: string };

export interface DepositMintsResponse {
  market_pubkey: string;
  deposit_assets: DepositAssetResponse[];
  total: number;
}

export interface GlobalDepositAssetResponse {
  id: number;
  mint: string;
  display_name?: string;
  symbol?: string;
  description?: string;
  icon_url?: string;
  decimals: number | null;
  whitelist_index: number;
  active: boolean;
}

export interface GlobalDepositAssetsListResponse {
  assets: GlobalDepositAssetResponse[];
  total: number;
}
