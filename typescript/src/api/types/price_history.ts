/**
 * Price history types for the Lightcone REST API.
 */

import { Resolution } from "../../shared";

/**
 * Price point data.
 */
export interface PricePoint {
  /** Timestamp (milliseconds) */
  t: number;
  /** Midpoint price as decimal string */
  m: string;
  /** Open price (only with include_ohlcv) as decimal string */
  o?: string;
  /** High price (only with include_ohlcv) as decimal string */
  h?: string;
  /** Low price (only with include_ohlcv) as decimal string */
  l?: string;
  /** Close price (only with include_ohlcv) as decimal string */
  c?: string;
  /** Volume (only with include_ohlcv) as decimal string */
  v?: string;
  /** Best bid (only with include_ohlcv) as decimal string */
  bb?: string;
  /** Best ask (only with include_ohlcv) as decimal string */
  ba?: string;
}

/**
 * Query parameters for GET /api/price-history.
 */
export interface PriceHistoryParams {
  /** Orderbook identifier (required) */
  orderbook_id: string;
  /** Candle resolution */
  resolution?: Resolution;
  /** Start timestamp (milliseconds) */
  from?: number;
  /** End timestamp (milliseconds) */
  to?: number;
  /** Pagination cursor */
  cursor?: number;
  /** Max results (1-1000) */
  limit?: number;
  /** Include full OHLCV data */
  include_ohlcv?: boolean;
}

/**
 * Create new PriceHistoryParams with required orderbook_id.
 */
export function createPriceHistoryParams(orderbook_id: string): PriceHistoryParams {
  return { orderbook_id };
}

/**
 * Response for GET /api/price-history.
 */
export interface PriceHistoryResponse {
  /** Orderbook ID */
  orderbook_id: string;
  /** Resolution used */
  resolution: string;
  /** Whether OHLCV data is included */
  include_ohlcv: boolean;
  /** Price points */
  prices: PricePoint[];
  /** Next pagination cursor */
  next_cursor?: number;
  /** Whether more results exist */
  has_more: boolean;
}
