/**
 * Position-related types for the Lightcone REST API.
 */

/**
 * Outcome balance in a position.
 */
export interface OutcomeBalance {
  /** Outcome index */
  outcome_index: number;
  /** Conditional token address */
  conditional_token: string;
  /** Total balance as decimal string */
  balance: string;
  /** Idle balance (not on book) as decimal string */
  balance_idle: string;
  /** Balance on order book as decimal string */
  balance_on_book: string;
}

/**
 * User position in a market.
 */
export interface Position {
  /** Database ID */
  id: number;
  /** Position PDA address */
  position_pubkey: string;
  /** Position owner */
  owner: string;
  /** Market pubkey */
  market_pubkey: string;
  /** Outcome balances */
  outcomes: OutcomeBalance[];
  /** Creation timestamp */
  created_at: string;
  /** Last update timestamp */
  updated_at: string;
}

/**
 * Response for GET /api/users/{user_pubkey}/positions.
 */
export interface PositionsResponse {
  /** Position owner */
  owner: string;
  /** Total markets with positions */
  total_markets: number;
  /** User positions */
  positions: Position[];
}

/**
 * Response for GET /api/users/{user_pubkey}/markets/{market_pubkey}/positions.
 */
export interface MarketPositionsResponse {
  /** Position owner */
  owner: string;
  /** Market pubkey */
  market_pubkey: string;
  /** Positions in this market */
  positions: Position[];
}
