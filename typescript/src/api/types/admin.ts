/**
 * Admin-related types for the Lightcone REST API.
 */

/**
 * Response for GET /api/admin/test.
 */
export interface AdminResponse {
  /** Status (usually "success") */
  status: string;
  /** Human-readable message */
  message: string;
}

/**
 * Request for POST /api/admin/create-orderbook.
 */
export interface CreateOrderbookRequest {
  /** Market address (Base58) */
  market_pubkey: string;
  /** Base conditional token (Base58) */
  base_token: string;
  /** Quote conditional token (Base58) */
  quote_token: string;
  /** Price granularity (default: 1000) */
  tick_size?: number;
}

/**
 * Create a new CreateOrderbookRequest with required fields.
 */
export function createOrderbookRequest(
  market_pubkey: string,
  base_token: string,
  quote_token: string
): CreateOrderbookRequest {
  return { market_pubkey, base_token, quote_token };
}

/**
 * Response for POST /api/admin/create-orderbook.
 */
export interface CreateOrderbookResponse {
  /** Status (usually "success") */
  status: string;
  /** Created orderbook ID */
  orderbook_id: string;
  /** Market pubkey */
  market_pubkey: string;
  /** Base token address */
  base_token: string;
  /** Quote token address */
  quote_token: string;
  /** Tick size */
  tick_size: number;
  /** Human-readable message */
  message: string;
}
