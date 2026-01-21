/**
 * Trade-related types for the Lightcone REST API.
 */

/**
 * Executed trade information.
 */
export interface Trade {
  /** Trade ID */
  id: number;
  /** Orderbook ID */
  orderbook_id: string;
  /** Taker's pubkey */
  taker_pubkey: string;
  /** Maker's pubkey */
  maker_pubkey: string;
  /** Trade side ("BID" or "ASK") */
  side: string;
  /** Trade size as decimal string */
  size: string;
  /** Trade price as decimal string */
  price: string;
  /** Taker fee as decimal string */
  taker_fee: string;
  /** Maker fee as decimal string */
  maker_fee: string;
  /** Execution timestamp (milliseconds since epoch) */
  executed_at: number;
}

/**
 * Query parameters for GET /api/trades.
 */
export interface TradesParams {
  /** Orderbook identifier (required) */
  orderbook_id: string;
  /** Filter by user pubkey */
  user_pubkey?: string;
  /** Start timestamp (milliseconds) */
  from?: number;
  /** End timestamp (milliseconds) */
  to?: number;
  /** Pagination cursor (trade ID) */
  cursor?: number;
  /** Max results (1-500) */
  limit?: number;
}

/**
 * Create new TradesParams with required orderbook_id.
 */
export function createTradesParams(orderbook_id: string): TradesParams {
  return { orderbook_id };
}

/**
 * Response for GET /api/trades.
 */
export interface TradesResponse {
  /** Orderbook ID */
  orderbook_id: string;
  /** Trade list */
  trades: Trade[];
  /** Next pagination cursor */
  next_cursor?: number;
  /** Whether more results exist */
  has_more: boolean;
}
