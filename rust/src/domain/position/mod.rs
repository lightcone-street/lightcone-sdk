#![doc = include_str!("README.md")]

pub mod builders;
pub mod client;
pub mod state;
pub mod wire;

use std::collections::{hash_map::Entry, HashMap};

pub use builders::{
    DepositBuilder, DepositToGlobalBuilder, ExtendPositionTokensBuilder,
    GlobalToMarketDepositBuilder, InitPositionTokensBuilder, MergeBuilder, RedeemWinningsBuilder,
    SolActionKind, SolActionPlan, SolBalanceDelta, WithdrawBuilder, WithdrawFromGlobalBuilder,
    WithdrawFromPositionBuilder,
};
pub use state::{
    CanonicalWsolAccountInfo, SolActionCosts, SolBalanceAvailability, SolBalanceBreakdown,
    WalletDepositBalancesApplyResult, WalletDepositBalancesState, WRAPPED_SOL_MINT_ADDRESS,
};

use crate::{
    prelude::{UserMarketBalance, UserOutcomeBalance},
    shared::{OrderBookId, PubkeyStr},
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

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

/// Exact mint-denominated balance plus optional display metadata for one SPL mint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositTokenBalance {
    /// SPL mint identifying this map entry.
    pub mint: PubkeyStr,
    /// Exact token amount using the mint's own decimal precision.
    pub idle: Decimal,
    /// Display symbol supplied by metadata.
    pub symbol: String,
    /// Display name supplied by metadata.
    pub name: String,
    /// Low-resolution icon, or `None` when metadata is unavailable.
    pub icon_url_low: Option<String>,
    /// Medium-resolution icon, or `None` when metadata is unavailable.
    pub icon_url_medium: Option<String>,
    /// High-resolution icon, or `None` when metadata is unavailable.
    pub icon_url_high: Option<String>,
}

/// Complete authenticated external-wallet balance snapshot.
///
/// Native SOL is required as canonical nine-decimal text and intentionally
/// remains outside the mint-keyed SPL map. The type has no empty default because
/// omitting either native or SPL balances would manufacture a partial snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositTokenBalancesSnapshot {
    /// Lower confirmed slot valid for both independently observed balance sources.
    pub context_slot: u64,
    /// Complete SPL balance map keyed by mint; it does not contain native SOL.
    pub balances: HashMap<PubkeyStr, DepositTokenBalance>,
    /// Exact non-negative native SOL with exactly nine fractional digits.
    #[serde(deserialize_with = "deserialize_native_sol_balance")]
    pub native_sol_balance: String,
}

/// Recoverable availability states emitted by the wallet-balance stream.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WalletDepositBalanceStatus {
    /// The backend is restoring its wallet watcher and will publish a replacement.
    Reconnecting,
    /// Balances remain usable, but token metadata could not be refreshed.
    MetadataUnavailable,
}

/// Nested payload carried by the authenticated `wallet_deposit_balances` channel.
///
/// [`WalletDepositBalancesState`] centralizes the replacement, wallet-matching,
/// absolute-update, and non-mutating-status rules for these variants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum WalletDepositBalancesEvent {
    /// Complete wallet baseline that replaces all prior SPL and native state.
    #[serde(rename = "wallet_deposit_balance_snapshot")]
    Snapshot {
        /// External wallet observed by this stream.
        wallet_address: PubkeyStr,
        /// Lower slot valid across the snapshot's SPL and native balances.
        context_slot: u64,
        /// Complete mint-keyed SPL map.
        balances: HashMap<PubkeyStr, DepositTokenBalance>,
        /// Exact non-negative native SOL with nine fractional digits.
        #[serde(deserialize_with = "deserialize_native_sol_balance")]
        native_sol_balance: String,
    },
    /// Absolute replacement for one SPL mint; zero removes it from app state.
    #[serde(rename = "wallet_deposit_balance_update")]
    BalanceUpdate {
        /// Wallet whose initialized baseline may accept the update.
        wallet_address: PubkeyStr,
        /// Slot of this mint balance observation, not a global stream sequence.
        context_slot: u64,
        /// Complete current balance for the affected mint, not a delta.
        balance: DepositTokenBalance,
    },
    /// Absolute native SOL replacement, never a delta.
    #[serde(rename = "wallet_native_sol_balance_update")]
    NativeSolBalanceUpdate {
        /// Wallet whose initialized baseline may accept the update.
        wallet_address: PubkeyStr,
        /// Slot of this native balance observation.
        context_slot: u64,
        /// Exact non-negative native SOL with nine fractional digits.
        #[serde(deserialize_with = "deserialize_native_sol_balance")]
        native_sol_balance: String,
    },
    /// Wallet-scoped diagnostic that leaves balances and slots unchanged.
    #[serde(rename = "wallet_deposit_balance_status")]
    Status {
        /// Wallet affected by the stream condition.
        wallet_address: PubkeyStr,
        /// Typed recoverable condition.
        status: WalletDepositBalanceStatus,
        /// Stable backend diagnostic code for logging or UX.
        code: String,
    },
}

/// Preserve the backend's canonical lamport representation as text.
///
/// General decimal syntax, exponents, signs, leading zeroes, and rounding are
/// rejected so every accepted value maps directly to one exact lamport count.
fn deserialize_native_sol_balance<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    let Some((whole, fraction)) = value.split_once('.') else {
        return Err(serde::de::Error::custom(
            "native_sol_balance must have exactly nine decimal places",
        ));
    };
    if whole.is_empty()
        || (whole.len() > 1 && whole.starts_with('0'))
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.len() != 9
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(serde::de::Error::custom(
            "native_sol_balance must have exactly nine decimal places",
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod deposit_token_balance_tests {
    use super::*;

    #[test]
    fn snapshot_deserializes_context_slot_and_balances() {
        let snapshot: DepositTokenBalancesSnapshot = serde_json::from_value(serde_json::json!({
            "context_slot": 1234,
            "native_sol_balance": "1.234567890",
            "balances": {
                "MintA": {
                    "mint": "MintA",
                    "idle": "1.25",
                    "symbol": "USDC",
                    "name": "USD Coin"
                }
            }
        }))
        .unwrap();

        assert_eq!(snapshot.context_slot, 1234);
        assert_eq!(snapshot.native_sol_balance, "1.234567890");
        assert_eq!(
            snapshot.balances[&PubkeyStr::from("MintA")].idle,
            Decimal::new(125, 2)
        );
    }

    #[test]
    fn snapshot_requires_native_sol_balance() {
        assert!(
            serde_json::from_value::<DepositTokenBalancesSnapshot>(serde_json::json!({
                "context_slot": 1234,
                "balances": {}
            }))
            .is_err()
        );
    }

    #[test]
    fn snapshot_preserves_exact_zero_native_sol_balance() {
        let snapshot: DepositTokenBalancesSnapshot = serde_json::from_value(serde_json::json!({
            "context_slot": 1234,
            "balances": {},
            "native_sol_balance": "0.000000000"
        }))
        .unwrap();

        assert_eq!(snapshot.native_sol_balance, "0.000000000");
    }

    #[test]
    fn snapshot_requires_exact_nine_decimal_native_sol_balance() {
        for native_sol_balance in ["1", "1.0", "01.000000000", "-1.000000000"] {
            assert!(
                serde_json::from_value::<DepositTokenBalancesSnapshot>(serde_json::json!({
                    "context_slot": 1234,
                    "balances": {},
                    "native_sol_balance": native_sol_balance
                }))
                .is_err()
            );
        }
    }
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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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
