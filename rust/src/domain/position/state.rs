//! App-owned external-wallet deposit balance state.
//!
//! A complete REST or WebSocket snapshot establishes the wallet-scoped
//! baseline. Later component events apply only to that wallet and carry
//! absolute values, while another complete snapshot may replace the baseline
//! even at a lower cross-component slot. Native SOL remains separate from the
//! sparse mint-keyed SPL map throughout this lifecycle.

use std::collections::HashMap;

use num_bigint::BigUint;
use num_traits::{ToPrimitive, Zero};

use crate::{
    error::SdkError,
    shared::{exact_scaled_integer, PubkeyStr},
};

use super::{DepositTokenBalance, DepositTokenBalancesSnapshot, WalletDepositBalancesEvent};

const SOL_DECIMALS: u8 = 9;
/// Canonical Tokenkeg native mint shared by state lookup and transaction construction.
pub(crate) const WRAPPED_SOL_MINT_ADDRESS: &str = "So11111111111111111111111111111111111111112";

/// Lifecycle disposition of a wallet-deposit balance event.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WalletDepositBalancesApplyResult {
    /// The event passed lifecycle guards and was applied; values may be unchanged.
    Applied,
    /// The event was informational, pre-initialization, or scoped to another wallet.
    Ignored,
}

/// Mutable wallet-scoped balance state owned by the application.
///
/// `Default` is uninitialized. Use [`Self::apply_rest_snapshot`] or a complete
/// WebSocket snapshot before applying component updates or signing a conversion.
/// The public fields make rendering straightforward, but callers should use the
/// application methods to preserve replacement and wallet-matching invariants.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct WalletDepositBalancesState {
    /// Wallet that owns the current baseline, or `None` before initialization.
    pub wallet_address: Option<PubkeyStr>,
    /// Slot from the last accepted event; complete snapshots may move it lower.
    pub context_slot: Option<u64>,
    /// Sparse SPL balances keyed by mint. Native SOL is never inserted here.
    pub balances: HashMap<PubkeyStr, DepositTokenBalance>,
    /// Exact nine-decimal native SOL value, or `None` before initialization.
    pub native_sol_balance: Option<String>,
}

impl WalletDepositBalancesState {
    /// Initialize or wholesale-replace state from an authenticated REST snapshot.
    ///
    /// The caller supplies the wallet because the REST payload omits it. No slot
    /// comparison is performed: every complete snapshot is authoritative.
    pub fn apply_rest_snapshot(
        &mut self,
        wallet_address: PubkeyStr,
        snapshot: &DepositTokenBalancesSnapshot,
    ) -> WalletDepositBalancesApplyResult {
        self.replace(
            wallet_address,
            snapshot.context_slot,
            snapshot.balances.clone(),
            snapshot.native_sol_balance.clone(),
        );
        WalletDepositBalancesApplyResult::Applied
    }

    /// Apply one typed nested WebSocket event.
    ///
    /// Complete snapshots replace every field regardless of prior slot order.
    /// Matching component updates replace one absolute value, explicit-zero SPL
    /// updates remove their mint, and status or wrong-wallet events are ignored.
    pub fn apply_event(
        &mut self,
        event: &WalletDepositBalancesEvent,
    ) -> WalletDepositBalancesApplyResult {
        match event {
            WalletDepositBalancesEvent::Snapshot {
                wallet_address,
                context_slot,
                balances,
                native_sol_balance,
            } => {
                // Complete snapshots are authoritative even when their lower
                // cross-component slot trails a previously observed update.
                self.replace(
                    wallet_address.clone(),
                    *context_slot,
                    balances.clone(),
                    native_sol_balance.clone(),
                );
                WalletDepositBalancesApplyResult::Applied
            }
            WalletDepositBalancesEvent::BalanceUpdate {
                wallet_address,
                context_slot,
                balance,
            } if self.matches_initialized_wallet(wallet_address) => {
                if balance.idle.is_zero() {
                    self.balances.remove(&balance.mint);
                } else {
                    self.balances.insert(balance.mint.clone(), balance.clone());
                }
                self.context_slot = Some(*context_slot);
                WalletDepositBalancesApplyResult::Applied
            }
            WalletDepositBalancesEvent::NativeSolBalanceUpdate {
                wallet_address,
                context_slot,
                native_sol_balance,
            } if self.matches_initialized_wallet(wallet_address) => {
                self.native_sol_balance = Some(native_sol_balance.clone());
                self.context_slot = Some(*context_slot);
                WalletDepositBalancesApplyResult::Applied
            }
            WalletDepositBalancesEvent::Status { .. }
            | WalletDepositBalancesEvent::BalanceUpdate { .. }
            | WalletDepositBalancesEvent::NativeSolBalanceUpdate { .. } => {
                WalletDepositBalancesApplyResult::Ignored
            }
        }
    }

    /// Return exact native SOL plus canonical WSOL, formatted to nine places.
    ///
    /// Addition uses arbitrary-width integers, so display state is not limited
    /// to Solana's transaction `u64` range. Invalid cached precision or an
    /// uninitialized native balance returns [`SdkError::Validation`] without
    /// changing the separately stored components.
    pub fn combined_sol_balance(&self) -> Result<String, SdkError> {
        let native = exact_lamports(self.native_sol_balance.as_deref().ok_or_else(|| {
            SdkError::Validation("wallet balance state is not initialized".into())
        })?)?;
        let wrapped = match self
            .balances
            .get(&PubkeyStr::from(WRAPPED_SOL_MINT_ADDRESS))
        {
            Some(balance) => exact_lamports(&balance.idle.to_string())?,
            None => BigUint::zero(),
        };
        Ok(format_lamports(&(native + wrapped)))
    }

    /// Scale cached native SOL exactly at the transaction's `u64` boundary.
    pub(crate) fn native_sol_lamports(&self) -> Result<u64, SdkError> {
        let value = self.native_sol_balance.as_deref().ok_or_else(|| {
            SdkError::Validation("wallet balance state is not initialized".into())
        })?;
        scaled_u64(value)
    }

    /// Validate and test the canonical WSOL idle balance used by unwrap preflight.
    pub(crate) fn has_positive_wsol(&self) -> Result<bool, SdkError> {
        match self
            .balances
            .get(&PubkeyStr::from(WRAPPED_SOL_MINT_ADDRESS))
        {
            Some(balance) => Ok(!exact_lamports(&balance.idle.to_string())?.is_zero()),
            None => Ok(false),
        }
    }

    /// Prevent component events from crossing wallet or incomplete-baseline boundaries.
    fn matches_initialized_wallet(&self, wallet_address: &PubkeyStr) -> bool {
        self.wallet_address.as_ref() == Some(wallet_address)
            && self.context_slot.is_some()
            && self.native_sol_balance.is_some()
    }

    fn replace(
        &mut self,
        wallet_address: PubkeyStr,
        context_slot: u64,
        balances: HashMap<PubkeyStr, DepositTokenBalance>,
        native_sol_balance: String,
    ) {
        self.wallet_address = Some(wallet_address);
        self.context_slot = Some(context_slot);
        self.balances = balances;
        self.native_sol_balance = Some(native_sol_balance);
    }
}

/// Parse a SOL amount without rounding and enforce Solana's transaction range.
pub(crate) fn sol_amount_to_lamports(value: &str) -> Result<u64, SdkError> {
    scaled_u64(value)
}

fn scaled_u64(value: &str) -> Result<u64, SdkError> {
    exact_lamports(value)?
        .to_u64()
        .ok_or_else(|| SdkError::Validation("SOL amount exceeds the transaction u64 range".into()))
}

fn exact_lamports(value: &str) -> Result<BigUint, SdkError> {
    // Keep state arithmetic arbitrary-width; only transaction construction
    // narrows through `scaled_u64` after exact nine-decimal validation.
    exact_scaled_integer(value, SOL_DECIMALS)
        .map_err(|error| SdkError::Validation(format!("invalid SOL amount: {error}")))
}

fn format_lamports(value: &BigUint) -> String {
    // Integer division restores canonical SOL text without floating-point loss.
    let scale = BigUint::from(1_000_000_000u64);
    let whole = value / &scale;
    let fraction = value % &scale;
    format!(
        "{}.{:0>9}",
        whole.to_str_radix(10),
        fraction.to_str_radix(10)
    )
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use super::*;
    use crate::domain::position::WalletDepositBalanceStatus;

    fn balance(mint: &str, idle: &str) -> DepositTokenBalance {
        DepositTokenBalance {
            mint: PubkeyStr::from(mint),
            idle: idle.parse::<Decimal>().unwrap(),
            symbol: "WSOL".into(),
            name: "Wrapped SOL".into(),
            icon_url_low: None,
            icon_url_medium: None,
            icon_url_high: None,
        }
    }

    fn snapshot(slot: u64, native: &str) -> DepositTokenBalancesSnapshot {
        DepositTokenBalancesSnapshot {
            context_slot: slot,
            balances: HashMap::new(),
            native_sol_balance: native.into(),
        }
    }

    #[test]
    fn rest_and_complete_ws_snapshots_replace_wholesale_despite_slot_order() {
        let wallet = PubkeyStr::from("WalletA");
        let mut state = WalletDepositBalancesState::default();
        state.apply_rest_snapshot(wallet.clone(), &snapshot(200, "1.000000000"));
        state.apply_event(&WalletDepositBalancesEvent::Snapshot {
            wallet_address: wallet,
            context_slot: 100,
            balances: [(PubkeyStr::from("MintA"), balance("MintA", "2"))]
                .into_iter()
                .collect(),
            native_sol_balance: "3.000000000".into(),
        });

        assert_eq!(state.context_slot, Some(100));
        assert_eq!(state.native_sol_balance.as_deref(), Some("3.000000000"));
        assert_eq!(state.balances.len(), 1);
    }

    #[test]
    fn component_updates_require_initialized_matching_wallet_and_remove_zero() {
        let mut state = WalletDepositBalancesState::default();
        let update = WalletDepositBalancesEvent::BalanceUpdate {
            wallet_address: PubkeyStr::from("WalletA"),
            context_slot: 2,
            balance: balance("MintA", "1"),
        };
        assert_eq!(
            state.apply_event(&update),
            WalletDepositBalancesApplyResult::Ignored
        );

        state.apply_rest_snapshot(PubkeyStr::from("WalletA"), &snapshot(1, "1.000000000"));
        assert_eq!(
            state.apply_event(&update),
            WalletDepositBalancesApplyResult::Applied
        );
        assert!(state.balances.contains_key(&PubkeyStr::from("MintA")));

        let zero = WalletDepositBalancesEvent::BalanceUpdate {
            wallet_address: PubkeyStr::from("WalletA"),
            context_slot: 3,
            balance: balance("MintA", "0"),
        };
        state.apply_event(&zero);
        assert!(!state.balances.contains_key(&PubkeyStr::from("MintA")));

        assert_eq!(
            state.apply_event(&WalletDepositBalancesEvent::NativeSolBalanceUpdate {
                wallet_address: PubkeyStr::from("WalletA"),
                context_slot: 4,
                native_sol_balance: "2.000000001".into(),
            }),
            WalletDepositBalancesApplyResult::Applied
        );
        assert_eq!(state.context_slot, Some(4));
        assert_eq!(state.native_sol_balance.as_deref(), Some("2.000000001"));
    }

    #[test]
    fn status_and_wrong_wallet_updates_do_not_mutate_state() {
        let mut state = WalletDepositBalancesState::default();
        state.apply_rest_snapshot(PubkeyStr::from("WalletA"), &snapshot(1, "1.000000000"));
        let before = state.clone();

        assert_eq!(
            state.apply_event(&WalletDepositBalancesEvent::NativeSolBalanceUpdate {
                wallet_address: PubkeyStr::from("WalletB"),
                context_slot: 2,
                native_sol_balance: "2.000000000".into(),
            }),
            WalletDepositBalancesApplyResult::Ignored
        );
        assert_eq!(
            state.apply_event(&WalletDepositBalancesEvent::Status {
                wallet_address: PubkeyStr::from("WalletA"),
                status: WalletDepositBalanceStatus::Reconnecting,
                code: "SOLANA_WALLET_BALANCE_STREAM_RECONNECTING".into(),
            }),
            WalletDepositBalancesApplyResult::Ignored
        );
        assert_eq!(state, before);
    }

    #[test]
    fn combined_balance_is_exact_and_keeps_assets_separate() {
        let mut state = WalletDepositBalancesState::default();
        let mut rest = snapshot(1, "1.999999999");
        rest.balances.insert(
            PubkeyStr::from(WRAPPED_SOL_MINT_ADDRESS),
            balance(WRAPPED_SOL_MINT_ADDRESS, "0.000000001"),
        );
        state.apply_rest_snapshot(PubkeyStr::from("WalletA"), &rest);

        assert_eq!(state.combined_sol_balance().unwrap(), "2.000000000");
        assert_eq!(state.native_sol_balance.as_deref(), Some("1.999999999"));
        assert!(state
            .balances
            .contains_key(&PubkeyStr::from(WRAPPED_SOL_MINT_ADDRESS)));
    }

    #[test]
    fn combined_balance_supports_values_larger_than_u64() {
        let mut state = WalletDepositBalancesState::default();
        state.apply_rest_snapshot(
            PubkeyStr::from("WalletA"),
            &snapshot(1, "18446744073.709551616"),
        );
        assert_eq!(
            state.combined_sol_balance().unwrap(),
            "18446744073.709551616"
        );
    }

    #[test]
    fn conversion_amount_scaling_rejects_precision_and_u64_overflow() {
        assert_eq!(sol_amount_to_lamports("0.123456789").unwrap(), 123_456_789);
        assert!(sol_amount_to_lamports("0.0000000001").is_err());
        assert!(sol_amount_to_lamports("18446744073.709551616").is_err());
    }
}
