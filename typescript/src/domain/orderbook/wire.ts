import type { OrderBookId, Side } from "../../shared";

export interface OrderbookResponse {
  id: number;
  market_pubkey: string;
  orderbook_id: string;
  base_token: string;
  quote_token: string;
  outcome_index?: number;
  tick_size: number;
  total_bids: number;
  total_asks: number;
  last_trade_price?: string;
  last_trade_time?: string;
  active: boolean;
  created_at: string;
  updated_at: string;
}

export interface OrderbooksResponse {
  orderbooks: OrderbookResponse[];
  total: number;
}

export interface RestBookLevel {
  price: string;
  size: string;
  orders?: number;
}

/**
 * REST depth response. Depth is capped server-side at 20 levels per side.
 */
export interface OrderbookDepthResponse {
  orderbook_id: OrderBookId;
  market_pubkey?: string;
  best_bid?: string;
  best_ask?: string;
  spread?: string;
  tick_size?: string;
  bids: RestBookLevel[];
  asks: RestBookLevel[];
  /**
   * Display decimals for prices and sizes. Always sent by current backends;
   * optional for tolerance of older payloads.
   */
  decimals?: OrderbookDepthDecimals;
}

/**
 * Price/size display decimals from the depth endpoint. Distinct from
 * `DecimalsResponse` (the `/decimals` endpoint).
 */
export interface OrderbookDepthDecimals {
  price: number;
  size: number;
}

export interface DecimalsResponse {
  orderbook_id: string;
  base_decimals: number;
  quote_decimals: number;
  price_decimals: number;
}

export interface WsBookLevel {
  side: Side;
  price: string;
  size: string;
}

/**
 * WS orderbook snapshot frame.
 *
 * The stream is snapshot-only: every data frame carries the full top-20
 * levels per side and replaces the previous book wholesale (last-write-wins).
 * `seq` is strictly increasing but non-contiguous, and the initial snapshot
 * after every (re)subscribe is `seq: 0` — informational only, never a gate.
 */
export interface OrderBook {
  orderbook_id: OrderBookId;
  is_snapshot?: boolean;
  seq?: number;
  resync?: boolean;
  timestamp?: string;
  bids: WsBookLevel[];
  asks: WsBookLevel[];
  /**
   * Aggregation tags echoed by the backend (omitted = full precision).
   * Always normalized server-side ((5, none) arrives as (5, 1)). Use
   * `aggregationFromFrame(book.n_sig_figs, book.mantissa)` to key
   * per-`(orderbook, aggregation)` book state.
   */
  n_sig_figs?: number;
  mantissa?: number;
}

export interface WsTickerData {
  orderbook_id: OrderBookId;
  best_bid?: string;
  best_ask?: string;
  mid?: string;
  timestamp?: string;
}
