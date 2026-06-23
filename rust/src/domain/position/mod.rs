#![doc = include_str!("README.md")]

pub mod builders;
pub mod client;
pub mod wire;

use std::collections::{hash_map::Entry, HashMap};

pub use builders::{
    DepositBuilder, DepositToGlobalBuilder, ExtendPositionTokensBuilder,
    GlobalToMarketDepositBuilder, InitPositionTokensBuilder, MergeBuilder, RedeemWinningsBuilder,
    WithdrawBuilder, WithdrawFromGlobalBuilder, WithdrawFromPositionBuilder,
};

use crate::{
    prelude::{UserMarketBalance, UserOutcomeBalance},
    shared::{OrderBookId, PubkeyStr},
};
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

impl From<DepositTokenBalance> for TokenBalance {
    fn from(value: DepositTokenBalance) -> Self {
        Self {
            mint: value.mint,
            idle: value.idle,
            on_book: Decimal::ZERO,
            token_type: TokenBalanceTokenType::DepositAsset,
        }
    }
}

impl From<ConditionalBalanceDelta> for TokenBalance {
    fn from(value: ConditionalBalanceDelta) -> Self {
        Self {
            mint: value.conditional_token.clone(),
            idle: value.idle,
            on_book: value.on_book,
            token_type: TokenBalanceTokenType::ConditionalToken {
                orderbook_id: value.orderbook_id.clone().unwrap_or_default(),
                market_pubkey: value.market_pubkey.clone(),
                outcome_index: value.outcome_index,
            },
        }
    }
}

impl TokenBalance {
    pub fn computed_base(&self, conditional_price: Decimal) -> TokenBalanceComputedBase {
        use crate::shared::fmt::decimal;
        let size = self.idle + self.on_book;
        let value = size * conditional_price;

        TokenBalanceComputedBase {
            value: decimal::display(&value),
            size: decimal::display(&size),
            price: decimal::display(&conditional_price),
        }
    }

    pub fn computed_quote(&self) -> String {
        use crate::shared::fmt::decimal;
        let size = self.idle + self.on_book;
        decimal::display(&size)
    }
}

pub struct TokenBalanceComputedBase {
    pub value: String,
    pub size: String,
    pub price: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositAssetMetadata {
    pub symbol: String,
    pub short_symbol: String,
    pub name: String,
    pub deposit_asset: PubkeyStr,
    pub icon_url_low: String,
    pub icon_url_medium: String,
    pub icon_url_high: String,
    pub description: Option<String>,
    pub decimals: u16,
}

/// Combined balance + metadata for a deposit token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositTokenBalance {
    pub mint: PubkeyStr,
    pub idle: Decimal,
    pub symbol: String,
    pub name: String,
    pub icon_url_low: Option<String>,
    pub icon_url_medium: Option<String>,
    pub icon_url_high: Option<String>,
}

impl From<ConditionalBalanceDelta> for UserOutcomeBalance {
    fn from(delta: ConditionalBalanceDelta) -> Self {
        UserOutcomeBalance {
            outcome_index: delta.outcome_index,
            conditional_token: delta.conditional_token.clone(),
            balance: delta.total(),
            balance_idle: delta.idle,
            balance_on_book: delta.on_book,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalBalanceDelta {
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: Option<OrderBookId>,
    pub outcome_index: i16,
    pub conditional_token: PubkeyStr,
    pub idle: Decimal,
    pub on_book: Decimal,
}

impl ConditionalBalanceDelta {
    pub fn total(&self) -> Decimal {
        self.idle + self.on_book
    }

    pub fn is_zero(&self) -> bool {
        !(self.idle > Decimal::ZERO || self.on_book > Decimal::ZERO)
    }
}

pub type ConditionalTokenBalanceIndex = HashMap<PubkeyStr, UserOutcomeBalance>;
pub type DepositAssetBalanceIndex = HashMap<PubkeyStr, ConditionalTokenBalanceIndex>;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserMarketBalanceIndex(pub HashMap<PubkeyStr, DepositAssetBalanceIndex>);

impl UserMarketBalanceIndex {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn entry(
        &mut self,
        market_pubkey: PubkeyStr,
    ) -> Entry<'_, PubkeyStr, DepositAssetBalanceIndex> {
        self.0.entry(market_pubkey)
    }

    pub fn get(&self, market_pubkey: &PubkeyStr) -> Option<&DepositAssetBalanceIndex> {
        self.0.get(market_pubkey)
    }

    pub fn insert(&mut self, market_pubkey: PubkeyStr, market_entry: DepositAssetBalanceIndex) {
        self.0.insert(market_pubkey, market_entry);
    }

    pub fn extend(&mut self, other: Self) {
        for (market_pubkey, market_entry) in other.0 {
            self.entry(market_pubkey).or_default().extend(market_entry);
        }
    }

    pub fn remove(&mut self, market_pubkey: &PubkeyStr) {
        self.0.remove(market_pubkey);
    }

    pub fn get_mut(&mut self, market_pubkey: &PubkeyStr) -> Option<&mut DepositAssetBalanceIndex> {
        self.0.get_mut(market_pubkey)
    }

    pub fn inner(&self) -> &HashMap<PubkeyStr, DepositAssetBalanceIndex> {
        &self.0
    }

    pub fn market_pubkeys(&self) -> Vec<PubkeyStr> {
        let mut pubkeys: Vec<PubkeyStr> = self.0.keys().cloned().collect();
        pubkeys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        pubkeys
    }
}

impl From<UserMarketBalance> for Option<UserMarketBalanceIndex> {
    fn from(market_balance: UserMarketBalance) -> Option<UserMarketBalanceIndex> {
        let market_pubkey = market_balance.market_pubkey;
        let mut market_entry = DepositAssetBalanceIndex::new();

        for deposit_asset_balance in market_balance.deposit_assets {
            let mut outcomes = ConditionalTokenBalanceIndex::new();
            for outcome in deposit_asset_balance.outcomes {
                if !outcome.is_zero() {
                    outcomes.insert(outcome.conditional_token.clone(), outcome);
                }
            }
            if !outcomes.is_empty() {
                market_entry.insert(deposit_asset_balance.deposit_asset, outcomes);
            }
        }

        match market_entry.is_empty() {
            true => None,
            false => {
                let mut index = UserMarketBalanceIndex::new();
                index.entry(market_pubkey).or_default().extend(market_entry);
                Some(index)
            }
        }
    }
}

impl From<Vec<UserMarketBalance>> for UserMarketBalanceIndex {
    fn from(market_balances: Vec<UserMarketBalance>) -> Self {
        let mut index = UserMarketBalanceIndex::new();
        for market_balance in market_balances {
            if let Some(market_index) = market_balance.into() {
                index.extend(market_index);
            }
        }

        index
    }
}
