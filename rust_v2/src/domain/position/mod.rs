//! Position domain — user positions, portfolio, wallet holdings, token balances.

pub mod client;
pub mod wire;

use crate::shared::{OrderBookId, PubkeyStr};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ─── Portfolio ───────────────────────────────────────────────────────────────

/// Full portfolio for a user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Portfolio {
    pub user_address: PubkeyStr,
    pub wallet_holdings: Vec<WalletHolding>,
    pub positions: Vec<Position>,
    pub total_wallet_value: Decimal,
    pub total_positions_value: Decimal,
}

/// A user's position in a market.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub event_pubkey: PubkeyStr,
    pub event_name: String,
    pub event_img_src: String,
    pub outcomes: Vec<PositionOutcome>,
    pub total_value: Decimal,
    pub created_at: DateTime<Utc>,
}

/// One outcome within a position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionOutcome {
    pub condition_id: u8,
    pub condition_name: String,
    pub token_mint: PubkeyStr,
    pub amount: Decimal,
    pub usd_value: Decimal,
}

/// A wallet holding (non-conditional token balance).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletHolding {
    pub token_mint: PubkeyStr,
    pub symbol: String,
    pub amount: Decimal,
    pub decimals: u64,
    pub usd_value: Decimal,
    pub img_src: String,
}

// ─── TokenBalance ────────────────────────────────────────────────────────────

/// Classification of a token balance's source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenBalanceTokenType {
    DepositAsset,
    ConditionalToken {
        orderbook_id: OrderBookId,
        market_pubkey: PubkeyStr,
        outcome_index: i16,
    },
}

/// A user's balance for a specific token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenBalance {
    pub mint: PubkeyStr,
    pub idle: Decimal,
    pub on_book: Decimal,
    pub token_type: TokenBalanceTokenType,
}

impl Default for TokenBalance {
    fn default() -> Self {
        Self {
            mint: PubkeyStr::default(),
            idle: Decimal::ZERO,
            on_book: Decimal::ZERO,
            token_type: TokenBalanceTokenType::DepositAsset,
        }
    }
}

/// Metadata for a deposit asset fetched from on-chain or DAS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositAssetMetadata {
    pub symbol: String,
    pub name: String,
    pub icon_url: String,
    pub value: Decimal,
}

/// Combined balance + metadata for a deposit token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositTokenBalance {
    pub mint: PubkeyStr,
    pub idle: Decimal,
    pub symbol: String,
    pub name: String,
    pub icon_url: String,
}
