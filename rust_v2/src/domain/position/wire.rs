//! Wire types for position responses (REST).

use crate::shared::PubkeyStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// REST response for user positions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionsResponse {
    pub positions: Vec<PositionResponse>,
}

/// A single position from the REST API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PositionResponse {
    pub market_pubkey: PubkeyStr,
    pub user_pubkey: PubkeyStr,
    pub outcome_index: i16,
    pub token_mint: PubkeyStr,
    pub balance: Decimal,
}
