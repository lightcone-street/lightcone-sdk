/**
 * Orderbook-related types for the Lightcone REST API.
 */

/**
 * Price level in the orderbook.
 */
export interface PriceLevel {
  /** Price as decimal string (e.g., "0.500000") */
  price: string;
  /** Total size at this price level as decimal string */
  size: string;
  /** Number of orders at this level */
  orders: number;
}

/**
 * Response for GET /api/orderbook/{orderbook_id}.
 */
export interface OrderbookResponse {
  /** Market pubkey */
  market_pubkey: string;
  /** Orderbook identifier */
  orderbook_id: string;
  /** Bid levels (buy orders), sorted by price descending */
  bids: PriceLevel[];
  /** Ask levels (sell orders), sorted by price ascending */
  asks: PriceLevel[];
  /** Best bid price as decimal string */
  best_bid?: string;
  /** Best ask price as decimal string */
  best_ask?: string;
  /** Spread (best_ask - best_bid) as decimal string */
  spread?: string;
  /** Tick size for this orderbook as decimal string */
  tick_size: string;
}
