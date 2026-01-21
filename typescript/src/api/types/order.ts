/**
 * Order-related types for the Lightcone REST API.
 */

/**
 * Order side enum.
 */
export type ApiOrderSide = 0 | 1;

/**
 * Order status values.
 */
export type OrderStatusValue = "accepted" | "partial_fill" | "filled" | "rejected";

/**
 * Fill information from order matching.
 */
export interface Fill {
  /** Counterparty address */
  counterparty: string;
  /** Counterparty's order hash */
  counterparty_order_hash: string;
  /** Amount filled as decimal string */
  fill_amount: string;
  /** Fill price as decimal string */
  price: string;
  /** Whether this order was the maker */
  is_maker: boolean;
}

/**
 * Request for POST /api/orders/submit.
 */
export interface SubmitOrderRequest {
  /** Order creator's pubkey (Base58) */
  maker: string;
  /** User's nonce for uniqueness */
  nonce: number;
  /** Market address (Base58) */
  market_pubkey: string;
  /** Token being bought/sold (Base58) */
  base_token: string;
  /** Token used for payment (Base58) */
  quote_token: string;
  /** Order side (0=BID, 1=ASK) */
  side: number;
  /** Amount maker gives */
  maker_amount: number;
  /** Amount maker wants to receive */
  taker_amount: number;
  /** Unix timestamp, 0=no expiration */
  expiration?: number;
  /** Ed25519 signature (hex, 128 chars) */
  signature: string;
  /** Target orderbook */
  orderbook_id: string;
}

/**
 * Response for POST /api/orders/submit.
 */
export interface OrderResponse {
  /** Order hash (hex) */
  order_hash: string;
  /** Order status */
  status: string;
  /** Remaining amount as decimal string */
  remaining: string;
  /** Filled amount as decimal string */
  filled: string;
  /** Fill details */
  fills: Fill[];
}

/**
 * Request for POST /api/orders/cancel.
 */
export interface CancelOrderRequest {
  /** Hash of order to cancel (hex) */
  order_hash: string;
  /** Must match order creator (Base58) */
  maker: string;
}

/**
 * Response for POST /api/orders/cancel.
 */
export interface CancelResponse {
  /** Cancellation status */
  status: string;
  /** Order hash */
  order_hash: string;
  /** Remaining amount that was cancelled as decimal string */
  remaining: string;
}

/**
 * Request for POST /api/orders/cancel-all.
 */
export interface CancelAllOrdersRequest {
  /** User's public key (Base58) */
  user_pubkey: string;
  /** Limit to specific market (empty = all) */
  market_pubkey?: string;
}

/**
 * Response for POST /api/orders/cancel-all.
 */
export interface CancelAllResponse {
  /** Status (success) */
  status: string;
  /** User pubkey */
  user_pubkey: string;
  /** Market pubkey if specified */
  market_pubkey?: string;
  /** List of cancelled order hashes */
  cancelled_order_hashes: string[];
  /** Count of cancelled orders */
  count: number;
  /** Human-readable message */
  message: string;
}

/**
 * User order from GET /api/users/orders.
 */
export interface UserOrder {
  /** Order hash */
  order_hash: string;
  /** Market pubkey */
  market_pubkey: string;
  /** Orderbook ID */
  orderbook_id: string;
  /** Order side (0=BID, 1=ASK) */
  side: number;
  /** Maker amount as decimal string */
  maker_amount: string;
  /** Taker amount as decimal string */
  taker_amount: string;
  /** Remaining amount as decimal string */
  remaining: string;
  /** Filled amount as decimal string */
  filled: string;
  /** Order price as decimal string */
  price: string;
  /** Creation timestamp */
  created_at: string;
  /** Expiration timestamp */
  expiration: number;
}

/**
 * Request for POST /api/users/orders.
 */
export interface GetUserOrdersRequest {
  /** User's public key (Base58) */
  user_pubkey: string;
}

/**
 * Outcome balance in user orders response.
 */
export interface UserOrderOutcomeBalance {
  /** Outcome index */
  outcome_index: number;
  /** Conditional token address */
  conditional_token: string;
  /** Idle balance as decimal string */
  idle: string;
  /** Balance on order book as decimal string */
  on_book: string;
}

/**
 * User balance from GET /api/users/orders.
 */
export interface UserBalance {
  /** Market pubkey */
  market_pubkey: string;
  /** Deposit asset */
  deposit_asset: string;
  /** Outcome balances */
  outcomes: UserOrderOutcomeBalance[];
}

/**
 * Response for POST /api/users/orders.
 */
export interface UserOrdersResponse {
  /** User pubkey */
  user_pubkey: string;
  /** Open orders */
  orders: UserOrder[];
  /** User balances */
  balances: UserBalance[];
}
