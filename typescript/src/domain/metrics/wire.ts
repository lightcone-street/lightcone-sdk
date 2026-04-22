import type { OrderBookId, PubkeyStr } from "../../shared";

// Decimal-bearing fields arrive as strings from the backend.

// ─── Platform ───────────────────────────────────────────────────────────────

/** Response of `GET /api/metrics/platform`. */
export interface PlatformMetrics {
  volume_24h_usd: string;
  volume_7d_usd: string;
  volume_30d_usd: string;
  volume_total_usd: string;
  taker_bid_volume_24h_usd: string;
  taker_bid_volume_7d_usd: string;
  taker_bid_volume_30d_usd: string;
  taker_bid_volume_total_usd: string;
  taker_ask_volume_24h_usd: string;
  taker_ask_volume_7d_usd: string;
  taker_ask_volume_30d_usd: string;
  taker_ask_volume_total_usd: string;
  taker_bid_ask_imbalance_24h_pct: string;
  taker_bid_ask_imbalance_7d_pct: string;
  taker_bid_ask_imbalance_30d_pct: string;
  taker_bid_ask_imbalance_total_pct: string;
  unique_traders_24h: number;
  unique_traders_7d: number;
  unique_traders_30d: number;
  active_markets: number;
  active_orderbooks: number;
  deposit_token_volumes: DepositTokenVolumeMetrics[];
  updated_at?: string;
}

// ─── Market (listing + detail) ──────────────────────────────────────────────

/** Entry in `GET /api/metrics/markets`. */
export interface MarketVolumeMetrics {
  market_pubkey: PubkeyStr;
  slug?: string;
  market_name?: string;
  category?: string;
  volume_24h_usd: string;
  volume_7d_usd: string;
  volume_30d_usd: string;
  volume_total_usd: string;
  taker_bid_volume_24h_usd: string;
  taker_bid_volume_7d_usd: string;
  taker_bid_volume_30d_usd: string;
  taker_bid_volume_total_usd: string;
  taker_ask_volume_24h_usd: string;
  taker_ask_volume_7d_usd: string;
  taker_ask_volume_30d_usd: string;
  taker_ask_volume_total_usd: string;
  taker_bid_ask_imbalance_24h_pct: string;
  taker_bid_ask_imbalance_7d_pct: string;
  taker_bid_ask_imbalance_30d_pct: string;
  taker_bid_ask_imbalance_total_pct: string;
  unique_traders_24h: number;
  unique_traders_7d: number;
  unique_traders_30d: number;
  category_volume_share_24h_pct: string;
  platform_volume_share_24h_pct: string;
}

/** `GET /api/metrics/markets` envelope. */
export interface MarketsMetrics {
  markets: MarketVolumeMetrics[];
  total: number;
}

export interface OutcomeVolumeMetrics {
  outcome_index: number | null;
  outcome_name?: string;
  volume_24h_usd: string;
  volume_7d_usd: string;
  volume_30d_usd: string;
  volume_total_usd: string;
  taker_bid_volume_24h_usd: string;
  taker_bid_volume_7d_usd: string;
  taker_bid_volume_30d_usd: string;
  taker_bid_volume_total_usd: string;
  taker_ask_volume_24h_usd: string;
  taker_ask_volume_7d_usd: string;
  taker_ask_volume_30d_usd: string;
  taker_ask_volume_total_usd: string;
  taker_bid_ask_imbalance_24h_pct: string;
  taker_bid_ask_imbalance_7d_pct: string;
  taker_bid_ask_imbalance_30d_pct: string;
  taker_bid_ask_imbalance_total_pct: string;
  unique_traders_24h: number;
  unique_traders_7d: number;
  unique_traders_30d: number;
  volume_share_24h_pct: string;
}

export interface MarketOrderbookVolumeMetrics {
  orderbook_id: OrderBookId;
  outcome_index: number | null;
  outcome_name?: string;
  base_deposit_asset: PubkeyStr;
  base_deposit_symbol?: string;
  quote_deposit_asset: PubkeyStr;
  quote_deposit_symbol?: string;
  volume_24h_usd: string;
  volume_7d_usd: string;
  volume_30d_usd: string;
  volume_total_usd: string;
  volume_24h_base: string;
  volume_7d_base: string;
  volume_30d_base: string;
  volume_total_base: string;
  volume_24h_quote: string;
  volume_7d_quote: string;
  volume_30d_quote: string;
  volume_total_quote: string;
  taker_bid_volume_24h_usd: string;
  taker_bid_volume_7d_usd: string;
  taker_bid_volume_30d_usd: string;
  taker_bid_volume_total_usd: string;
  taker_bid_volume_24h_base: string;
  taker_bid_volume_7d_base: string;
  taker_bid_volume_30d_base: string;
  taker_bid_volume_total_base: string;
  taker_bid_volume_24h_quote: string;
  taker_bid_volume_7d_quote: string;
  taker_bid_volume_30d_quote: string;
  taker_bid_volume_total_quote: string;
  taker_ask_volume_24h_usd: string;
  taker_ask_volume_7d_usd: string;
  taker_ask_volume_30d_usd: string;
  taker_ask_volume_total_usd: string;
  taker_ask_volume_24h_base: string;
  taker_ask_volume_7d_base: string;
  taker_ask_volume_30d_base: string;
  taker_ask_volume_total_base: string;
  taker_ask_volume_24h_quote: string;
  taker_ask_volume_7d_quote: string;
  taker_ask_volume_30d_quote: string;
  taker_ask_volume_total_quote: string;
  taker_bid_ask_imbalance_24h_pct: string;
  taker_bid_ask_imbalance_7d_pct: string;
  taker_bid_ask_imbalance_30d_pct: string;
  taker_bid_ask_imbalance_total_pct: string;
  volume_share_24h_pct: string;
}

/** Response of `GET /api/metrics/markets/{market_pubkey}`. */
export interface MarketDetailMetrics {
  market_pubkey: PubkeyStr;
  slug?: string;
  market_name?: string;
  category?: string;
  volume_24h_usd: string;
  volume_7d_usd: string;
  volume_30d_usd: string;
  volume_total_usd: string;
  taker_bid_volume_24h_usd: string;
  taker_bid_volume_7d_usd: string;
  taker_bid_volume_30d_usd: string;
  taker_bid_volume_total_usd: string;
  taker_ask_volume_24h_usd: string;
  taker_ask_volume_7d_usd: string;
  taker_ask_volume_30d_usd: string;
  taker_ask_volume_total_usd: string;
  taker_bid_ask_imbalance_24h_pct: string;
  taker_bid_ask_imbalance_7d_pct: string;
  taker_bid_ask_imbalance_30d_pct: string;
  taker_bid_ask_imbalance_total_pct: string;
  unique_traders_24h: number;
  unique_traders_7d: number;
  unique_traders_30d: number;
  category_volume_share_24h_pct: string;
  platform_volume_share_24h_pct: string;
  outcome_volumes: OutcomeVolumeMetrics[];
  orderbook_volumes: MarketOrderbookVolumeMetrics[];
  deposit_token_volumes: DepositTokenVolumeMetrics[];
}

// ─── Orderbook ──────────────────────────────────────────────────────────────

/** Response of `GET /api/metrics/orderbooks/{orderbook_id}`. */
export interface OrderbookVolumeMetrics {
  orderbook_id: OrderBookId;
  market_pubkey: PubkeyStr;
  outcome_index: number | null;
  outcome_name?: string;
  base_deposit_asset: PubkeyStr;
  base_deposit_symbol?: string;
  quote_deposit_asset: PubkeyStr;
  quote_deposit_symbol?: string;
  volume_24h_usd: string;
  volume_7d_usd: string;
  volume_30d_usd: string;
  volume_total_usd: string;
  volume_24h_base: string;
  volume_7d_base: string;
  volume_30d_base: string;
  volume_total_base: string;
  volume_24h_quote: string;
  volume_7d_quote: string;
  volume_30d_quote: string;
  volume_total_quote: string;
  taker_bid_volume_24h_usd: string;
  taker_bid_volume_7d_usd: string;
  taker_bid_volume_30d_usd: string;
  taker_bid_volume_total_usd: string;
  taker_bid_volume_24h_base: string;
  taker_bid_volume_7d_base: string;
  taker_bid_volume_30d_base: string;
  taker_bid_volume_total_base: string;
  taker_bid_volume_24h_quote: string;
  taker_bid_volume_7d_quote: string;
  taker_bid_volume_30d_quote: string;
  taker_bid_volume_total_quote: string;
  taker_ask_volume_24h_usd: string;
  taker_ask_volume_7d_usd: string;
  taker_ask_volume_30d_usd: string;
  taker_ask_volume_total_usd: string;
  taker_ask_volume_24h_base: string;
  taker_ask_volume_7d_base: string;
  taker_ask_volume_30d_base: string;
  taker_ask_volume_total_base: string;
  taker_ask_volume_24h_quote: string;
  taker_ask_volume_7d_quote: string;
  taker_ask_volume_30d_quote: string;
  taker_ask_volume_total_quote: string;
  taker_bid_ask_imbalance_24h_pct: string;
  taker_bid_ask_imbalance_7d_pct: string;
  taker_bid_ask_imbalance_30d_pct: string;
  taker_bid_ask_imbalance_total_pct: string;
  unique_traders_24h: number;
  unique_traders_7d: number;
  unique_traders_30d: number;
  market_volume_share_24h_pct: string;
}

// ─── Category ───────────────────────────────────────────────────────────────

/**
 * Entry in `GET /api/metrics/categories` and the single response of
 * `GET /api/metrics/categories/{category}`.
 */
export interface CategoryVolumeMetrics {
  category: string;
  volume_24h_usd: string;
  volume_7d_usd: string;
  volume_30d_usd: string;
  volume_total_usd: string;
  taker_bid_volume_24h_usd: string;
  taker_bid_volume_7d_usd: string;
  taker_bid_volume_30d_usd: string;
  taker_bid_volume_total_usd: string;
  taker_ask_volume_24h_usd: string;
  taker_ask_volume_7d_usd: string;
  taker_ask_volume_30d_usd: string;
  taker_ask_volume_total_usd: string;
  taker_bid_ask_imbalance_24h_pct: string;
  taker_bid_ask_imbalance_7d_pct: string;
  taker_bid_ask_imbalance_30d_pct: string;
  taker_bid_ask_imbalance_total_pct: string;
  unique_traders_24h: number;
  unique_traders_7d: number;
  unique_traders_30d: number;
  platform_volume_share_24h_pct: string;
  deposit_token_volumes: DepositTokenVolumeMetrics[];
}

export interface CategoriesMetrics {
  categories: CategoryVolumeMetrics[];
}

// ─── Deposit tokens ─────────────────────────────────────────────────────────

export interface DepositTokenVolumeMetrics {
  deposit_asset: PubkeyStr;
  symbol?: string;
  volume_24h_usd: string;
  volume_7d_usd: string;
  volume_30d_usd: string;
  volume_total_usd: string;
  taker_bid_volume_24h_usd: string;
  taker_bid_volume_7d_usd: string;
  taker_bid_volume_30d_usd: string;
  taker_bid_volume_total_usd: string;
  taker_ask_volume_24h_usd: string;
  taker_ask_volume_7d_usd: string;
  taker_ask_volume_30d_usd: string;
  taker_ask_volume_total_usd: string;
  taker_bid_ask_imbalance_24h_pct: string;
  taker_bid_ask_imbalance_7d_pct: string;
  taker_bid_ask_imbalance_30d_pct: string;
  taker_bid_ask_imbalance_total_pct: string;
  volume_share_24h_pct: string;
}

export interface DepositTokensMetrics {
  deposit_tokens: DepositTokenVolumeMetrics[];
}

// ─── Leaderboard ────────────────────────────────────────────────────────────

export interface LeaderboardEntry {
  rank: number;
  market_pubkey: PubkeyStr;
  slug?: string;
  market_name?: string;
  category?: string;
  volume_24h_usd: string;
  category_volume_share_24h_pct: string;
  platform_volume_share_24h_pct: string;
}

export interface Leaderboard {
  entries: LeaderboardEntry[];
  period: string;
}

// ─── History ────────────────────────────────────────────────────────────────

export interface HistoryPoint {
  /** Bucket start, Unix epoch milliseconds. */
  bucket_start: number;
  volume_usd: string;
}

export interface MetricsHistory {
  scope: string;
  scope_key: string;
  resolution: string;
  points: HistoryPoint[];
}

/** Query for `GET /api/metrics/history/{scope}/{scope_key}`. */
export interface MetricsHistoryQuery {
  /** Defaults to `"1h"` on the backend. */
  resolution?: string;
  from?: number;
  to?: number;
  limit?: number;
}
