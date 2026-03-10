export interface OutcomeBalance {
  outcome_index: number;
  conditional_token: string;
  balance: string;
  balance_idle: string;
  balance_on_book: string;
}

export interface VaultBalance {
  deposit_mint: string;
  vault: string;
  balance: string;
}

export interface PositionEntry {
  id: number;
  position_pubkey: string;
  owner: string;
  market_pubkey: string;
  outcomes: OutcomeBalance[];
  vault_balances: VaultBalance[];
  created_at: string;
  updated_at: string;
}

export interface PositionsResponse {
  owner: string;
  total_markets: number;
  positions: PositionEntry[];
  decimals: Record<string, number>;
}

export interface MarketPositionsResponse {
  owner: string;
  market_pubkey: string;
  positions: PositionEntry[];
  decimals: Record<string, number>;
}
