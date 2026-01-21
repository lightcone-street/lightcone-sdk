//! Position-related types for the Lightcone REST API.

use serde::{Deserialize, Serialize};

/// Outcome balance in a position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeBalance {
    /// Outcome index
    pub outcome_index: u32,
    /// Conditional token address
    pub conditional_token: String,
    /// Total balance as decimal string
    pub balance: String,
    /// Idle balance (not on book) as decimal string
    pub balance_idle: String,
    /// Balance on order book as decimal string
    pub balance_on_book: String,
}

/// User position in a market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Database ID
    pub id: i64,
    /// Position PDA address
    pub position_pubkey: String,
    /// Position owner
    pub owner: String,
    /// Market pubkey
    pub market_pubkey: String,
    /// Outcome balances
    pub outcomes: Vec<OutcomeBalance>,
    /// Creation timestamp
    pub created_at: String,
    /// Last update timestamp
    pub updated_at: String,
}

/// Response for GET /api/users/{user_pubkey}/positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionsResponse {
    /// Position owner
    pub owner: String,
    /// Total markets with positions
    pub total_markets: u64,
    /// User positions
    pub positions: Vec<Position>,
}

/// Response for GET /api/users/{user_pubkey}/markets/{market_pubkey}/positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPositionsResponse {
    /// Position owner
    pub owner: String,
    /// Market pubkey
    pub market_pubkey: String,
    /// Positions in this market
    pub positions: Vec<Position>,
}
