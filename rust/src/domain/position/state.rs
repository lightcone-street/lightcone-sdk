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
use rust_decimal::Decimal;

use crate::{
    error::SdkError,
    shared::{exact_scaled_integer, PubkeyStr},
};

use super::{DepositTokenBalance, DepositTokenBalancesSnapshot, WalletDepositBalancesEvent};

const SOL_DECIMALS: u8 = 9;
/// Canonical WSOL mint under Solana's legacy SPL Token Program (“Tokenkeg”).
///
/// State lookup and ATA derivation are deliberately pinned to Tokenkeg rather
/// than Token-2022 so every SDK addresses the protocol's one canonical account.
pub const WRAPPED_SOL_MINT_ADDRESS: &str = "So11111111111111111111111111111111111111112";

/// Unsponsored native reserve floor, in lamports, when canonical ATA creation is required.
pub const SOL_RESERVE_WITH_ACCOUNT_CREATION_LAMPORTS: u64 = 3_500_000;
/// Unsponsored native reserve floor, in lamports, when canonical WSOL already exists.
pub const SOL_RESERVE_WITH_EXISTING_ACCOUNT_LAMPORTS: u64 = 1_000_000;

/// Stores exact live facts for the Trading Wallet's canonical Tokenkeg WSOL account.
///
/// All fields are integer lamports from one confirmed `getAccountInfo` response.
/// `account_lamports` includes the native reserve and any unsynchronized donated
/// lamports. `token_amount_lamports` and `native_reserve_lamports` come from the
/// decoded SPL token state. Direct construction performs no validation. The RPC
/// inspection method validates the canonical address, legacy Token Program owner,
/// native mint, wallet and close authorities, initialized state, native reserve,
/// and account-lamport consistency before returning this value.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CanonicalWsolAccountInfo {
    /// All lamports held by the token account, including its rent reserve.
    pub account_lamports: u64,
    /// Spendable WSOL recorded in the token account's amount field.
    pub token_amount_lamports: u64,
    /// Rent-exempt reserve recorded by the native token account.
    pub native_reserve_lamports: u64,
}

/// Exact transaction-range components behind the single user-facing SOL balance.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SolBalanceComponents {
    /// Lamports held by the Trading Wallet's system account.
    pub native_lamports: u64,
    /// Token amount in the Trading Wallet's canonical WSOL ATA, in lamports.
    pub canonical_wsol_lamports: u64,
}

impl SolBalanceComponents {
    /// Native plus canonical WSOL in lamports, saturated to Solana's `u64` range.
    pub fn lamports(self) -> u64 {
        self.native_lamports
            .saturating_add(self.canonical_wsol_lamports)
    }

    /// Native plus canonical WSOL as an exact nine-decimal SOL amount.
    pub fn sol(self) -> Decimal {
        Decimal::from_i128_with_scale(i128::from(self.lamports()), u32::from(SOL_DECIMALS))
    }
}

/// Live chain costs and sponsorship inputs used to derive action availability.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SolActionCosts {
    /// Fee returned by `getFeeForMessage`, in lamports.
    pub fee_lamports: u64,
    /// Rent that must be funded before the transaction can execute, even if refunded later.
    pub upfront_rent_lamports: u64,
    /// Whether this transaction creates the canonical WSOL ATA.
    pub creates_canonical_wsol_account: bool,
    /// Exact public sponsorship capability for the current build.
    pub sponsored: bool,
}

impl SolActionCosts {
    /// Reserve enough native SOL for live costs and the configured safety floor.
    pub fn reserve_lamports(self) -> Result<u64, SdkError> {
        let live_costs = self
            .fee_lamports
            .checked_add(self.upfront_rent_lamports)
            .ok_or_else(|| SdkError::Validation("SOL transaction costs overflow u64".into()))?;
        if self.sponsored {
            return Ok(0);
        }
        let floor = if self.creates_canonical_wsol_account {
            SOL_RESERVE_WITH_ACCOUNT_CREATION_LAMPORTS
        } else {
            SOL_RESERVE_WITH_EXISTING_ACCOUNT_LAMPORTS
        };
        Ok(live_costs.max(floor))
    }
}

/// Action-specific SOL totals after reserving native transaction funds.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SolBalanceAvailability {
    /// Separately authoritative native and canonical WSOL balances.
    pub components: SolBalanceComponents,
    /// Sum of both components in lamports, before action reserve.
    pub displayed_lamports: u64,
    /// Action-specific native lamports withheld; ordinary actions include a safety floor.
    pub reserve_lamports: u64,
    /// Displayed lamports available to the planned action after reserve.
    pub spendable_lamports: u64,
}

impl SolBalanceAvailability {
    /// Derive ordinary availability, failing on component/cost overflow or insufficient native reserve.
    pub fn from_costs(
        components: SolBalanceComponents,
        costs: SolActionCosts,
    ) -> Result<Self, SdkError> {
        let displayed_lamports = components
            .native_lamports
            .checked_add(components.canonical_wsol_lamports)
            .ok_or_else(|| SdkError::Validation("SOL balance components overflow u64".into()))?;
        let reserve_lamports = costs.reserve_lamports()?;
        if components.native_lamports < reserve_lamports {
            return Err(SdkError::Validation(format!(
                "native SOL balance cannot fund the required {reserve_lamports} lamport transaction reserve"
            )));
        }
        Ok(Self {
            components,
            displayed_lamports,
            reserve_lamports,
            spendable_lamports: displayed_lamports.saturating_sub(reserve_lamports),
        })
    }

    /// Return unwrap-all availability with the live fee as its entire reserve.
    ///
    /// This method rejects `SolActionCosts` that include sponsorship, account
    /// creation, or upfront rent. It checks the component sum and fee subtraction.
    /// Native SOL must fund the fee without relying on lamports that a later
    /// `CloseAccount` instruction may transfer. The ordinary safety floor does not
    /// apply because unwrap-all removes the persistent canonical account.
    pub fn from_unwrap_all_costs(
        components: SolBalanceComponents,
        costs: SolActionCosts,
    ) -> Result<Self, SdkError> {
        if costs.upfront_rent_lamports != 0
            || costs.creates_canonical_wsol_account
            || costs.sponsored
        {
            return Err(SdkError::Validation(
                "unwrap-all costs must be unsponsored with no upfront rent or account creation"
                    .into(),
            ));
        }
        if components.native_lamports < costs.fee_lamports {
            return Err(SdkError::Validation(format!(
                "native SOL balance cannot fund the required {} lamport unwrap-all fee",
                costs.fee_lamports
            )));
        }
        let displayed_lamports = components
            .native_lamports
            .checked_add(components.canonical_wsol_lamports)
            .ok_or_else(|| {
                SdkError::Validation("unwrap-all displayed SOL balance overflows u64".into())
            })?;
        let spendable_lamports = displayed_lamports
            .checked_sub(costs.fee_lamports)
            .ok_or_else(|| {
                SdkError::Validation("unwrap-all fee exceeds displayed SOL balance".into())
            })?;
        Ok(Self {
            components,
            displayed_lamports,
            reserve_lamports: costs.fee_lamports,
            spendable_lamports,
        })
    }
}

/// Lifecycle disposition of a wallet-deposit balance event.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WalletDepositBalancesApplyResult {
    /// The event passed lifecycle guards and was applied; values may be unchanged.
    Applied,
    /// The event was informational, pre-initialization, or scoped to another wallet.
    Ignored,
    /// The event targeted this state but carried an invalid balance.
    Rejected,
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
                if balance.idle.is_sign_negative() {
                    return WalletDepositBalancesApplyResult::Rejected;
                } else if balance.idle.is_zero() {
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

    /// Return the exact transaction-range native and canonical WSOL components.
    ///
    /// Display arithmetic may exceed `u64`, but a fund-moving plan cannot. This
    /// boundary therefore rejects either component when it cannot enter a Solana
    /// instruction without rounding or truncation.
    pub fn sol_components(&self) -> Result<SolBalanceComponents, SdkError> {
        Ok(SolBalanceComponents {
            native_lamports: self.native_sol_lamports()?,
            canonical_wsol_lamports: self.canonical_wsol_lamports()?,
        })
    }

    /// Scale cached native SOL exactly at the transaction's `u64` boundary.
    pub(crate) fn native_sol_lamports(&self) -> Result<u64, SdkError> {
        let value = self.native_sol_balance.as_deref().ok_or_else(|| {
            SdkError::Validation("wallet balance state is not initialized".into())
        })?;
        scaled_u64(value)
    }

    /// Scale the canonical WSOL idle balance exactly to lamports.
    pub fn canonical_wsol_lamports(&self) -> Result<u64, SdkError> {
        match self
            .balances
            .get(&PubkeyStr::from(WRAPPED_SOL_MINT_ADDRESS))
        {
            Some(balance) => scaled_u64(&balance.idle.to_string()),
            None => Ok(0),
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

        let before = state.clone();
        assert_eq!(
            state.apply_event(&WalletDepositBalancesEvent::BalanceUpdate {
                wallet_address: PubkeyStr::from("WalletA"),
                context_slot: 4,
                balance: balance("MintA", "-1"),
            }),
            WalletDepositBalancesApplyResult::Rejected
        );
        assert_eq!(state, before);

        assert_eq!(
            state.apply_event(&WalletDepositBalancesEvent::NativeSolBalanceUpdate {
                wallet_address: PubkeyStr::from("WalletA"),
                context_slot: 5,
                native_sol_balance: "2.000000001".into(),
            }),
            WalletDepositBalancesApplyResult::Applied
        );
        assert_eq!(state.context_slot, Some(5));
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
    /// Exposes saturating lamports and exact decimal SOL without a fallible display conversion.
    fn sol_balance_components_report_lamports_and_sol() {
        let components = SolBalanceComponents {
            native_lamports: 1_000_000_000,
            canonical_wsol_lamports: 500_000_000,
        };
        assert_eq!(components.lamports(), 1_500_000_000);
        assert_eq!(components.sol(), Decimal::new(15, 1));

        assert_eq!(
            SolBalanceComponents {
                native_lamports: u64::MAX,
                canonical_wsol_lamports: 1,
            }
            .lamports(),
            u64::MAX
        );
    }

    #[test]
    /// Keeps display arithmetic broad while rejecting transaction-range overflow.
    fn transaction_components_reject_u64_overflow() {
        let mut state = WalletDepositBalancesState::default();
        state.apply_rest_snapshot(
            PubkeyStr::from("WalletA"),
            &snapshot(1, "18446744073.709551616"),
        );
        assert!(state.sol_components().is_err());
    }

    #[test]
    /// Uses live costs above a floor and only honors explicit sponsorship.
    fn availability_uses_live_costs_or_the_matching_unsponsored_floor() {
        let components = SolBalanceComponents {
            native_lamports: 10_000_000,
            canonical_wsol_lamports: 5_000_000,
        };
        let existing = SolBalanceAvailability::from_costs(
            components,
            SolActionCosts {
                fee_lamports: 5_000,
                upfront_rent_lamports: 0,
                creates_canonical_wsol_account: false,
                sponsored: false,
            },
        )
        .unwrap();
        assert_eq!(existing.reserve_lamports, 1_000_000);
        assert_eq!(existing.spendable_lamports, 14_000_000);

        let creates_account = SolBalanceAvailability::from_costs(
            components,
            SolActionCosts {
                fee_lamports: 1_000_000,
                upfront_rent_lamports: 3_000_000,
                creates_canonical_wsol_account: true,
                sponsored: false,
            },
        )
        .unwrap();
        assert_eq!(creates_account.reserve_lamports, 4_000_000);

        let sponsored = SolBalanceAvailability::from_costs(
            components,
            SolActionCosts {
                fee_lamports: 20_000_000,
                upfront_rent_lamports: 20_000_000,
                creates_canonical_wsol_account: true,
                sponsored: true,
            },
        )
        .unwrap();
        assert_eq!(sponsored.reserve_lamports, 0);
        assert_eq!(sponsored.spendable_lamports, 15_000_000);
        assert!(SolBalanceAvailability::from_costs(
            components,
            SolActionCosts {
                fee_lamports: u64::MAX,
                upfront_rent_lamports: 1,
                creates_canonical_wsol_account: true,
                sponsored: true,
            },
        )
        .is_err());
    }

    #[test]
    /// Requires the reserve in native SOL even when aggregate WSOL is sufficient.
    fn availability_requires_native_reserve_even_when_wrapped_sol_is_sufficient() {
        let error = SolBalanceAvailability::from_costs(
            SolBalanceComponents {
                native_lamports: 999_999,
                canonical_wsol_lamports: 10_000_000,
            },
            SolActionCosts {
                fee_lamports: 5_000,
                upfront_rent_lamports: 0,
                creates_canonical_wsol_account: false,
                sponsored: false,
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("transaction reserve"));
    }

    #[test]
    fn ordinary_availability_rejects_component_overflow_without_changing_display_saturation() {
        let components = SolBalanceComponents {
            native_lamports: u64::MAX,
            canonical_wsol_lamports: 1,
        };
        assert_eq!(components.lamports(), u64::MAX);

        let error = SolBalanceAvailability::from_costs(
            components,
            SolActionCosts {
                fee_lamports: 0,
                upfront_rent_lamports: 0,
                creates_canonical_wsol_account: false,
                sponsored: true,
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("components overflow u64"));
    }

    #[test]
    fn unwrap_all_availability_uses_fee_only_and_preserves_exact_components(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let components = SolBalanceComponents {
            native_lamports: 10_000,
            canonical_wsol_lamports: 2_000_000,
        };
        let availability = SolBalanceAvailability::from_unwrap_all_costs(
            components,
            SolActionCosts {
                fee_lamports: 5_000,
                upfront_rent_lamports: 0,
                creates_canonical_wsol_account: false,
                sponsored: false,
            },
        )?;

        assert_eq!(availability.components, components);
        assert_eq!(availability.displayed_lamports, 2_010_000);
        assert_eq!(availability.reserve_lamports, 5_000);
        assert_eq!(availability.spendable_lamports, 2_005_000);
        Ok(())
    }

    #[test]
    fn unwrap_all_availability_rejects_fee_shortfall_overflow_and_ordinary_costs() {
        let costs = SolActionCosts {
            fee_lamports: 5_000,
            upfront_rent_lamports: 0,
            creates_canonical_wsol_account: false,
            sponsored: false,
        };
        let fee_error = SolBalanceAvailability::from_unwrap_all_costs(
            SolBalanceComponents {
                native_lamports: 4_999,
                canonical_wsol_lamports: 10_000,
            },
            costs,
        )
        .unwrap_err();
        assert!(fee_error.to_string().contains("unwrap-all fee"));

        let overflow = SolBalanceAvailability::from_unwrap_all_costs(
            SolBalanceComponents {
                native_lamports: u64::MAX,
                canonical_wsol_lamports: 1,
            },
            SolActionCosts {
                fee_lamports: 0,
                ..costs
            },
        )
        .unwrap_err();
        assert!(overflow.to_string().contains("overflows u64"));

        let ordinary_costs = SolBalanceAvailability::from_unwrap_all_costs(
            SolBalanceComponents {
                native_lamports: 10_000,
                canonical_wsol_lamports: 1,
            },
            SolActionCosts {
                upfront_rent_lamports: 1,
                ..costs
            },
        )
        .unwrap_err();
        assert!(ordinary_costs.to_string().contains("no upfront rent"));
    }
}
