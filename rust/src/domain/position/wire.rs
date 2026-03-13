//! Wire types for position responses (REST).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Outcome balance within a position.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutcomeBalance {
    pub outcome_index: i16,
    pub conditional_token: String,
    pub balance: String,
    pub balance_idle: String,
    pub balance_on_book: String,
}

/// Vault balance for a deposit asset within a market.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VaultBalance {
    pub deposit_mint: String,
    pub vault: String,
    pub balance: String,
}

/// Global deposit balance attached to positions REST responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalDeposit {
    pub deposit_mint: String,
    pub symbol: String,
    pub balance: String,
}

/// A user's position in a market.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionEntry {
    pub id: i32,
    pub position_pubkey: String,
    pub owner: String,
    pub market_pubkey: String,
    pub outcomes: Vec<OutcomeBalance>,
    #[serde(default)]
    pub vault_balances: Vec<VaultBalance>,
    pub created_at: String,
    pub updated_at: String,
}

/// Response for `GET /api/users/{user_pubkey}/positions`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionsResponse {
    pub owner: String,
    pub total_markets: usize,
    pub positions: Vec<PositionEntry>,
    #[serde(default)]
    pub global_deposits: Vec<GlobalDeposit>,
    pub decimals: HashMap<String, u8>,
}

/// Response for `GET /api/users/{user_pubkey}/markets/{market_pubkey}/positions`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketPositionsResponse {
    pub owner: String,
    pub market_pubkey: String,
    pub positions: Vec<PositionEntry>,
    #[serde(default)]
    pub global_deposits: Vec<GlobalDeposit>,
    pub decimals: HashMap<String, u8>,
}
