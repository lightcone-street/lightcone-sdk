//! Positions sub-client for portfolio queries and fee-prepared position operations.
//!
//! Explicit WSOL conversion follows this lifecycle:
//! complete matching wallet state and native keypair -> live canonical-account and
//! cost reads -> signer, account, reserve, and amount guards -> fee-prepared plan ->
//! unchanged prepared submission -> complete snapshot covering the confirmed slot.
//! An uncertain submission returns control to the caller, which refreshes
//! authoritative state before planning another action.

use crate::auth::AuthCredentials;
use crate::client::LightconeClient;
use crate::domain::market::Market;
use crate::domain::position::builders::{
    build_direct_native_withdraw_transaction, build_sol_merge_transaction,
    build_sol_redeem_transaction, build_sol_split_transaction,
    build_temporary_native_withdraw_transaction, native_withdraw_seed, temporary_wsol_account,
    wrapped_sol_accounts, DepositBuilder, DepositToGlobalBuilder, ExtendPositionTokensBuilder,
    GlobalToMarketDepositBuilder, InitPositionTokensBuilder, MergeBuilder, RedeemWinningsBuilder,
    SolActionKind, SolActionPlan, SolComponentDelta, WithdrawBuilder, WithdrawFromGlobalBuilder,
    WithdrawFromPositionBuilder, TOKEN_ACCOUNT_SPACE,
};
#[cfg(feature = "native-auth")]
use crate::domain::position::builders::{
    build_unwrap_wsol_all_transaction, build_wrap_sol_transaction,
};
use crate::domain::position::wire::{MarketPositionsResponse, PositionsResponse};
use crate::domain::position::{
    DepositTokenBalancesSnapshot, SolActionCosts, SolBalanceAvailability,
    WalletDepositBalancesState,
};
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::program::instructions;
use crate::program::types::{
    ClosePositionAltParams, ClosePositionTokenAccountsParams, DepositToGlobalAltContext,
    DepositToGlobalParams, ExtendPositionTokensParams, GlobalToMarketDepositParams,
    InitPositionTokensParams, RedeemWinningsParams, WithdrawConditionalFromPositionParams,
    WithdrawFromGlobalParams, WithdrawFromPositionParams,
};
use crate::shared::signing::SigningStrategy;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;

fn deposit_token_balances_query(min_context_slot: Option<u64>) -> Vec<(&'static str, String)> {
    min_context_slot
        .map(|slot| vec![("min_context_slot", slot.to_string())])
        .unwrap_or_default()
}

fn validated_conversion_wallet(
    credentials: Option<&AuthCredentials>,
    state: &WalletDepositBalancesState,
) -> Result<Pubkey, SdkError> {
    // Cached identity is a signing trust boundary, not just display metadata.
    // Require an unexpired session and a complete baseline for the same wallet.
    let credentials = credentials
        .ok_or_else(|| SdkError::Validation("authenticated credentials are required".into()))?;
    if !credentials.is_authenticated() {
        return Err(SdkError::Validation(
            "authenticated credentials have expired".into(),
        ));
    }
    let state_wallet = state
        .wallet_address
        .as_ref()
        .ok_or_else(|| SdkError::Validation("wallet balance state is not initialized".into()))?;
    if state_wallet != &credentials.wallet_address {
        return Err(SdkError::Validation(
            "authenticated wallet does not match wallet balance state".into(),
        ));
    }
    if state.context_slot.is_none() || state.native_sol_balance.is_none() {
        return Err(SdkError::Validation(
            "wallet balance state is not initialized".into(),
        ));
    }
    credentials
        .wallet_address
        .to_pubkey()
        .map_err(SdkError::Validation)
}

fn validate_signing_wallet(strategy: &SigningStrategy, wallet: Pubkey) -> Result<(), SdkError> {
    let signing_wallet = strategy.wallet_address().ok_or_else(|| {
        SdkError::Validation("signing strategy wallet identity is required".into())
    })?;
    if signing_wallet != wallet {
        return Err(SdkError::Validation(
            "signing strategy does not control authenticated wallet".into(),
        ));
    }
    Ok(())
}

/// Validate that a local keypair controls the authenticated Trading Wallet.
///
/// This check returns an error for external signing strategies or a different
/// keypair address. It runs before conversion RPC reads. Ordinary planners keep
/// using [`validate_signing_wallet`] and therefore retain browser-wallet support.
#[cfg(feature = "native-auth")]
fn validate_native_conversion_signing_wallet(
    strategy: &SigningStrategy,
    wallet: Pubkey,
) -> Result<(), SdkError> {
    let signing_wallet = strategy.native_conversion_wallet().ok_or_else(|| {
        SdkError::Validation(
            "native keypair signing is required for explicit WSOL conversion".into(),
        )
    })?;
    if signing_wallet != wallet {
        return Err(SdkError::Validation(
            "native signing keypair does not control authenticated wallet".into(),
        ));
    }
    Ok(())
}

fn require_unsponsored_plan(sponsored: bool) -> Result<(), SdkError> {
    if sponsored {
        return Err(SdkError::Validation(
            "sponsored SOL action planning is not supported".into(),
        ));
    }
    Ok(())
}

pub struct Positions<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Positions<'a> {
    // ── PDA helpers ──────────────────────────────────────────────────────

    /// Get the Position PDA.
    pub fn pda(&self, owner: &Pubkey, market: &Pubkey) -> Pubkey {
        crate::program::pda::get_position_pda(owner, market, &self.client.program_id).0
    }

    // ── HTTP methods ─────────────────────────────────────────────────────

    /// Get all positions for a user across all markets.
    pub async fn get(&self, user_pubkey: &str) -> Result<PositionsResponse, SdkError> {
        let url = format!(
            "{}/api/users/{}/positions",
            self.client.http.base_url(),
            user_pubkey
        );
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Get positions for a user in a specific market.
    pub async fn get_for_market(
        &self,
        user_pubkey: &str,
        market_pubkey: &str,
    ) -> Result<MarketPositionsResponse, SdkError> {
        let url = format!(
            "{}/api/users/{}/markets/{}/positions",
            self.client.http.base_url(),
            user_pubkey,
            market_pubkey
        );
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Get all conditional-token positions for the authenticated user across
    /// every market. The wallet is resolved server-side from the auth cookie,
    /// so no parameter is required. Same response shape as
    /// [`Positions::get`]; empty `positions` array when the user has none.
    pub async fn positions(&self) -> Result<PositionsResponse, SdkError> {
        let url = format!("{}/api/users/positions", self.client.http.base_url());
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Same as [`Self::positions`], but forwards the supplied raw `Cookie` header (`privy-token` and/or `lightcone-token`) for
    /// this call instead of the SDK's process-wide token store.
    ///
    /// Intended for server-side cookie forwarding (SSR / server functions)
    /// where the per-request browser cookie can't propagate to the shared
    /// client. On WASM this is equivalent to [`Self::positions`] because the
    /// browser is already attaching the cookie via credentials mode.
    pub async fn positions_with_cookies(
        &self,
        cookie_header: &str,
    ) -> Result<PositionsResponse, SdkError> {
        let url = format!("{}/api/users/positions", self.client.http.base_url());
        self.client
            .http
            .get_with_cookies(&url, RetryPolicy::Idempotent, cookie_header)
            .await
    }

    /// Get the authenticated user's positions in a specific market. The
    /// wallet is resolved server-side from the auth cookie.
    pub async fn positions_for_market(
        &self,
        market_pubkey: &str,
    ) -> Result<MarketPositionsResponse, SdkError> {
        let url = format!(
            "{}/api/users/markets/{}/positions",
            self.client.http.base_url(),
            market_pubkey
        );
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Same as [`Self::positions_for_market`], but forwards the supplied raw
    /// `Cookie` header (`privy-token` and/or `lightcone-token`) for this call
    /// instead of the SDK's process-wide token store. For server-side cookie
    /// forwarding (SSR / server functions).
    pub async fn positions_for_market_with_cookies(
        &self,
        market_pubkey: &str,
        cookie_header: &str,
    ) -> Result<MarketPositionsResponse, SdkError> {
        let url = format!(
            "{}/api/users/markets/{}/positions",
            self.client.http.base_url(),
            market_pubkey
        );
        self.client
            .http
            .get_with_cookies(&url, RetryPolicy::Idempotent, cookie_header)
            .await
    }

    /// Fetch a complete authenticated SPL and native-SOL balance snapshot.
    ///
    /// `min_context_slot` lower-bounds the complete cross-component snapshot.
    /// Native SOL is required canonical nine-decimal text and remains separate
    /// from `balances`; apply the result through [`WalletDepositBalancesState`]
    /// when combining it with WebSocket updates.
    pub async fn deposit_token_balances(
        &self,
        min_context_slot: Option<u64>,
    ) -> Result<DepositTokenBalancesSnapshot, SdkError> {
        let url = format!(
            "{}/api/users/deposit-token-balances",
            self.client.http.base_url()
        );
        let query = deposit_token_balances_query(min_context_slot);
        self.client
            .http
            .get_with_query(&url, &query, RetryPolicy::Idempotent)
            .await
    }

    /// Same as [`Self::deposit_token_balances`], but forwards the supplied raw
    /// `Cookie` header (`privy-token` and/or `lightcone-token`) for this call
    /// instead of the SDK's process-wide token store.
    ///
    /// Intended for server-side cookie forwarding (SSR / server functions)
    /// where the per-request browser cookies can't propagate to the shared
    /// client. On WASM this is equivalent to
    /// [`Self::deposit_token_balances`] because the browser is already
    /// attaching cookies via credentials mode.
    pub async fn deposit_token_balances_with_cookies(
        &self,
        min_context_slot: Option<u64>,
        cookie_header: &str,
    ) -> Result<DepositTokenBalancesSnapshot, SdkError> {
        let url = format!(
            "{}/api/users/deposit-token-balances",
            self.client.http.base_url()
        );
        let query = deposit_token_balances_query(min_context_slot);
        self.client
            .http
            .get_with_cookies_and_query(&url, &query, RetryPolicy::Idempotent, cookie_header)
            .await
    }

    /// Return a fee-prepared plan for an exact canonical WSOL wrap.
    ///
    /// The authenticated Trading Wallet must have a local native keypair and a
    /// complete balance snapshot. Live canonical account data must match the
    /// snapshot. An existing account must have account lamports equal to its token
    /// amount plus native reserve. Otherwise a later `SyncNative` instruction would
    /// recalculate the WSOL token amount from account lamports and wrap donated
    /// excess beyond `amount_lamports`. The returned transaction
    /// contains strict ATA creation only when the account is absent. It then
    /// contains the exact transfer and `SyncNative`. Availability uses the ordinary
    /// reserve floor. The projected native delta includes `amount_lamports`, the
    /// live fee, and newly funded account rent.
    ///
    /// Callers rebuild immediately before signing and submit through the prepared
    /// transaction API. Callers retain the returned component projection until a
    /// complete snapshot covers the confirmed slot. A submission or confirmation
    /// error is uncertain, so callers refresh authoritative state before retrying.
    #[cfg(feature = "native-auth")]
    pub async fn plan_wrap_sol(
        &self,
        amount_lamports: u64,
        state: &WalletDepositBalancesState,
    ) -> Result<SolActionPlan, SdkError> {
        if amount_lamports == 0 {
            return Err(SdkError::Validation(
                "wrap amount must be greater than zero".into(),
            ));
        }
        let wallet = self.native_conversion_planning_wallet(state).await?;
        let components = state.sol_components()?;
        let (_, canonical_account) = wrapped_sol_accounts(&wallet)?;
        let account_info = self
            .client
            .canonical_wsol_account_info(&canonical_account, &wallet)
            .await?;
        match account_info {
            Some(info) => {
                if info.token_amount_lamports != components.canonical_wsol_lamports {
                    return Err(SdkError::Validation(
                        "live canonical WSOL amount does not match wallet balance state".into(),
                    ));
                }
                let synchronized_account_lamports = info
                    .token_amount_lamports
                    .checked_add(info.native_reserve_lamports)
                    .ok_or_else(|| {
                        SdkError::Validation(
                            "canonical WSOL token amount plus native reserve overflows u64".into(),
                        )
                    })?;
                if info.account_lamports != synchronized_account_lamports {
                    return Err(SdkError::Validation(
                        "canonical WSOL account has unsynchronized excess lamports".into(),
                    ));
                }
                if info
                    .token_amount_lamports
                    .checked_add(amount_lamports)
                    .is_none()
                    || info.account_lamports.checked_add(amount_lamports).is_none()
                {
                    return Err(SdkError::Validation(
                        "wrap would exceed canonical WSOL token or account u64 range".into(),
                    ));
                }
            }
            None if components.canonical_wsol_lamports > 0 => {
                return Err(SdkError::Validation(
                    "canonical WSOL balance is positive but its account is unavailable".into(),
                ));
            }
            _ => {}
        }
        let creates_canonical_wsol_account = account_info.is_none();
        let upfront_rent_lamports = if creates_canonical_wsol_account {
            self.client
                .minimum_balance_for_rent_exemption(TOKEN_ACCOUNT_SPACE)
                .await?
        } else {
            0
        };
        let mut transaction =
            build_wrap_sol_transaction(wallet, amount_lamports, creates_canonical_wsol_account)?;
        let fee_lamports = self
            .client
            .prepare_and_estimate_transaction_fee(&mut transaction)
            .await?;
        let costs = SolActionCosts {
            fee_lamports,
            upfront_rent_lamports,
            creates_canonical_wsol_account,
            sponsored: false,
        };
        let availability = SolBalanceAvailability::from_costs(components, costs)?;
        let required_native = amount_lamports
            .checked_add(availability.reserve_lamports)
            .ok_or_else(|| SdkError::Validation("wrap native requirement overflows u64".into()))?;
        if components.native_lamports < required_native {
            return Err(SdkError::Validation(
                "native SOL cannot fund the wrap amount and transaction reserve".into(),
            ));
        }
        let wallet_costs = fee_lamports
            .checked_add(upfront_rent_lamports)
            .ok_or_else(|| SdkError::Validation("wrap costs overflow u64".into()))?;
        Ok(SolActionPlan {
            kind: SolActionKind::Wrap,
            transaction,
            costs,
            availability,
            expected_delta: SolComponentDelta {
                native_lamports: -i128::from(amount_lamports) - i128::from(wallet_costs),
                canonical_wsol_lamports: i128::from(amount_lamports),
            },
        })
    }

    /// Return a fee-prepared plan for closing the complete canonical WSOL account.
    ///
    /// The local native keypair supplies the transaction payer, close authority,
    /// and destination. Snapshot canonical WSOL must be positive and equal the live
    /// decoded token amount. The returned transaction contains one `CloseAccount`
    /// instruction for the Trading Wallet's canonical account. If submitted
    /// successfully, that instruction transfers the account's complete lamport
    /// balance, including rent and donated excess, to the Trading Wallet. The
    /// returned `SolActionCosts` has zero upfront rent, no account creation, and no
    /// sponsorship. Availability reserves only the freshly estimated fee, which
    /// must already be available in native SOL.
    ///
    /// Callers rebuild immediately before prepared submission. They retain the
    /// returned component projection until a complete snapshot covers the
    /// confirmed slot. An uncertain outcome requires authoritative refresh before
    /// another plan; it does not authorize automatic resubmission.
    #[cfg(feature = "native-auth")]
    pub async fn plan_unwrap_wsol_all(
        &self,
        state: &WalletDepositBalancesState,
    ) -> Result<SolActionPlan, SdkError> {
        let wallet = self.native_conversion_planning_wallet(state).await?;
        let components = state.sol_components()?;
        if components.canonical_wsol_lamports == 0 {
            return Err(SdkError::Validation(
                "canonical WSOL balance must be greater than zero for unwrap-all".into(),
            ));
        }
        let (_, canonical_account) = wrapped_sol_accounts(&wallet)?;
        let account_info = self
            .client
            .canonical_wsol_account_info(&canonical_account, &wallet)
            .await?
            .ok_or_else(|| {
                SdkError::Validation("canonical WSOL account is required for unwrap-all".into())
            })?;
        if account_info.token_amount_lamports != components.canonical_wsol_lamports {
            return Err(SdkError::Validation(
                "live canonical WSOL amount does not match wallet balance state".into(),
            ));
        }

        let mut transaction = build_unwrap_wsol_all_transaction(wallet)?;
        let fee_lamports = self
            .client
            .prepare_and_estimate_transaction_fee(&mut transaction)
            .await?;
        let costs = SolActionCosts {
            fee_lamports,
            upfront_rent_lamports: 0,
            creates_canonical_wsol_account: false,
            sponsored: false,
        };
        let availability = SolBalanceAvailability::from_unwrap_all_costs(components, costs)?;
        components
            .native_lamports
            .checked_sub(fee_lamports)
            .and_then(|native_after_fee| {
                native_after_fee.checked_add(account_info.account_lamports)
            })
            .ok_or_else(|| {
                SdkError::Validation("unwrap-all projected native balance overflows u64".into())
            })?;
        Ok(SolActionPlan {
            kind: SolActionKind::UnwrapAll,
            transaction,
            costs,
            availability,
            expected_delta: SolComponentDelta {
                native_lamports: i128::from(account_info.account_lamports)
                    - i128::from(fee_lamports),
                canonical_wsol_lamports: -i128::from(account_info.token_amount_lamports),
            },
        })
    }

    /// Plan one atomic SOL-backed split using canonical WSOL before wrapping a shortfall.
    ///
    /// Amounts and costs are lamports. Account checks and live fee/rent reads
    /// fail closed; sponsored planning is rejected until a sponsor owns costs.
    pub async fn plan_sol_split(
        &self,
        market: &Market,
        amount_lamports: u64,
        state: &WalletDepositBalancesState,
        sponsored: bool,
    ) -> Result<SolActionPlan, SdkError> {
        require_unsponsored_plan(sponsored)?;
        if amount_lamports == 0 {
            return Err(SdkError::Validation(
                "split amount must be greater than zero".into(),
            ));
        }
        let wallet = self.planning_wallet(state).await?;
        let components = state.sol_components()?;
        let (_, canonical_account) = wrapped_sol_accounts(&wallet)?;
        let canonical_exists = self
            .client
            .canonical_wsol_account_exists(&canonical_account, &wallet)
            .await?;
        if components.canonical_wsol_lamports > 0 && !canonical_exists {
            return Err(SdkError::Validation(
                "canonical WSOL balance is positive but its account is unavailable".into(),
            ));
        }
        let shortfall = amount_lamports.saturating_sub(components.canonical_wsol_lamports);
        let upfront_rent_lamports = if canonical_exists {
            0
        } else {
            self.client
                .minimum_balance_for_rent_exemption(TOKEN_ACCOUNT_SPACE)
                .await?
        };
        let mut transaction = build_sol_split_transaction(
            &self.client.program_id,
            wallet,
            market,
            amount_lamports,
            shortfall,
            !canonical_exists,
        )?;
        let fee_lamports = self
            .client
            .prepare_and_estimate_transaction_fee(&mut transaction)
            .await?;
        let costs = SolActionCosts {
            fee_lamports,
            upfront_rent_lamports,
            creates_canonical_wsol_account: !canonical_exists,
            sponsored,
        };
        let availability = SolBalanceAvailability::from_costs(components, costs)?;
        if amount_lamports > availability.spendable_lamports {
            return Err(SdkError::Validation(
                "split amount exceeds spendable SOL after transaction reserve".into(),
            ));
        }
        let required_native = shortfall
            .checked_add(availability.reserve_lamports)
            .ok_or_else(|| SdkError::Validation("split native requirement overflows u64".into()))?;
        if required_native > components.native_lamports {
            return Err(SdkError::Validation(
                "native SOL cannot fund the wrap shortfall and transaction reserve".into(),
            ));
        }
        let wallet_costs = if sponsored {
            0
        } else {
            fee_lamports
                .checked_add(upfront_rent_lamports)
                .ok_or_else(|| SdkError::Validation("split costs overflow u64".into()))?
        };
        Ok(SolActionPlan {
            kind: SolActionKind::Split,
            transaction,
            costs,
            availability,
            expected_delta: SolComponentDelta {
                native_lamports: -i128::from(shortfall) - i128::from(wallet_costs),
                canonical_wsol_lamports: i128::from(shortfall) - i128::from(amount_lamports),
            },
        })
    }

    /// Plan a complete-set merge that retains returned WSOL in the canonical ATA.
    ///
    /// The returned unsigned transaction is fee-prepared and does not mutate
    /// cached state; callers refresh authority after confirmed submission.
    pub async fn plan_sol_merge(
        &self,
        market: &Market,
        amount_lamports: u64,
        state: &WalletDepositBalancesState,
        sponsored: bool,
    ) -> Result<SolActionPlan, SdkError> {
        require_unsponsored_plan(sponsored)?;
        if amount_lamports == 0 {
            return Err(SdkError::Validation(
                "merge amount must be greater than zero".into(),
            ));
        }
        let wallet = self.planning_wallet(state).await?;
        let components = state.sol_components()?;
        let (_, canonical_account) = wrapped_sol_accounts(&wallet)?;
        let canonical_exists = self
            .client
            .canonical_wsol_account_exists(&canonical_account, &wallet)
            .await?;
        if components.canonical_wsol_lamports > 0 && !canonical_exists {
            return Err(SdkError::Validation(
                "canonical WSOL balance is positive but its account is unavailable".into(),
            ));
        }
        let upfront_rent_lamports = if canonical_exists {
            0
        } else {
            self.client
                .minimum_balance_for_rent_exemption(TOKEN_ACCOUNT_SPACE)
                .await?
        };
        let mut transaction = build_sol_merge_transaction(
            &self.client.program_id,
            wallet,
            market,
            amount_lamports,
            !canonical_exists,
        )?;
        self.finish_receive_plan(
            SolActionKind::Merge,
            amount_lamports,
            components,
            sponsored,
            !canonical_exists,
            upfront_rent_lamports,
            &mut transaction,
        )
        .await
    }

    /// Plan a winning-token redemption that retains returned WSOL in the canonical ATA.
    ///
    /// `amount_lamports` is exact collateral scale and `outcome_index` remains
    /// governed by the on-chain market instruction.
    pub async fn plan_sol_redeem(
        &self,
        market: Pubkey,
        amount_lamports: u64,
        outcome_index: u8,
        num_outcomes: u8,
        state: &WalletDepositBalancesState,
        sponsored: bool,
    ) -> Result<SolActionPlan, SdkError> {
        require_unsponsored_plan(sponsored)?;
        if amount_lamports == 0 {
            return Err(SdkError::Validation(
                "redeem amount must be greater than zero".into(),
            ));
        }
        crate::program::utils::validate_outcome_count(num_outcomes)?;
        crate::program::utils::validate_outcome_index(outcome_index, num_outcomes)?;
        let wallet = self.planning_wallet(state).await?;
        let components = state.sol_components()?;
        let (_, canonical_account) = wrapped_sol_accounts(&wallet)?;
        let canonical_exists = self
            .client
            .canonical_wsol_account_exists(&canonical_account, &wallet)
            .await?;
        if components.canonical_wsol_lamports > 0 && !canonical_exists {
            return Err(SdkError::Validation(
                "canonical WSOL balance is positive but its account is unavailable".into(),
            ));
        }
        let upfront_rent_lamports = if canonical_exists {
            0
        } else {
            self.client
                .minimum_balance_for_rent_exemption(TOKEN_ACCOUNT_SPACE)
                .await?
        };
        let mut transaction = build_sol_redeem_transaction(
            &self.client.program_id,
            wallet,
            market,
            amount_lamports,
            outcome_index,
            !canonical_exists,
        )?;
        self.finish_receive_plan(
            SolActionKind::Redeem,
            amount_lamports,
            components,
            sponsored,
            !canonical_exists,
            upfront_rent_lamports,
            &mut transaction,
        )
        .await
    }

    /// Plan an exact native-SOL withdrawal to an arbitrary Solana recipient.
    ///
    /// Native funds are sent directly when they cover both amount and reserve.
    /// Otherwise only the required canonical WSOL is moved through a seeded,
    /// short-lived Tokenkeg account; the persistent canonical ATA stays open.
    /// Account presence, rent, and fees are live authority and any unavailable
    /// read fails closed. Seed selection tries at most eight blockhash-scoped
    /// candidates to bound RPC latency while making accidental exhaustion
    /// negligible. The returned transaction's message is already prepared.
    pub async fn plan_native_sol_withdrawal(
        &self,
        recipient: Pubkey,
        amount_lamports: u64,
        state: &WalletDepositBalancesState,
        sponsored: bool,
    ) -> Result<SolActionPlan, SdkError> {
        require_unsponsored_plan(sponsored)?;
        if amount_lamports == 0 {
            return Err(SdkError::Validation(
                "withdraw amount must be greater than zero".into(),
            ));
        }
        let wallet = self.planning_wallet(state).await?;
        let components = state.sol_components()?;

        let mut direct =
            build_direct_native_withdraw_transaction(wallet, recipient, amount_lamports);
        let direct_fee = self
            .client
            .prepare_and_estimate_transaction_fee(&mut direct)
            .await?;
        let direct_costs = SolActionCosts {
            fee_lamports: direct_fee,
            upfront_rent_lamports: 0,
            creates_canonical_wsol_account: false,
            sponsored,
        };
        let direct_availability = SolBalanceAvailability::from_costs(components, direct_costs)?;
        if amount_lamports > direct_availability.spendable_lamports {
            return Err(SdkError::Validation(
                "withdraw amount exceeds spendable SOL after transaction reserve".into(),
            ));
        }
        let direct_required = amount_lamports
            .checked_add(direct_availability.reserve_lamports)
            .ok_or_else(|| {
                SdkError::Validation("withdraw native requirement overflows u64".into())
            })?;
        if components.native_lamports >= direct_required {
            return Ok(SolActionPlan {
                kind: SolActionKind::NativeWithdraw,
                transaction: direct,
                costs: direct_costs,
                availability: direct_availability,
                expected_delta: SolComponentDelta {
                    native_lamports: -i128::from(amount_lamports)
                        - i128::from(if sponsored { 0 } else { direct_fee }),
                    canonical_wsol_lamports: 0,
                },
            });
        }

        let (_, canonical_account) = wrapped_sol_accounts(&wallet)?;
        if !self
            .client
            .canonical_wsol_account_exists(&canonical_account, &wallet)
            .await?
        {
            return Err(SdkError::Validation(
                "canonical WSOL is required for this native withdrawal".into(),
            ));
        }
        let temporary_rent = self
            .client
            .minimum_balance_for_rent_exemption(TOKEN_ACCOUNT_SPACE)
            .await?;
        let seed_blockhash = self.client.get_latest_blockhash().await?;
        let mut selected = None;
        // Bound account-existence RPCs; the blockhash and attempt byte make eight collisions remote.
        for attempt in 0..=7 {
            let seed = native_withdraw_seed(
                &seed_blockhash,
                &wallet,
                &recipient,
                amount_lamports,
                attempt,
            );
            let account = temporary_wsol_account(&wallet, &seed)?;
            if !self.client.account_exists(&account).await? {
                selected = Some((seed, account));
                break;
            }
        }
        let (seed, temporary_account) = selected.ok_or_else(|| {
            SdkError::Validation("temporary WSOL seed attempts are exhausted".into())
        })?;

        let mut transaction = build_temporary_native_withdraw_transaction(
            wallet,
            recipient,
            amount_lamports,
            1,
            temporary_rent,
            &seed,
            temporary_account,
        )?;
        transaction.message.recent_blockhash = seed_blockhash;
        let initial_fee = self
            .client
            .estimate_prepared_transaction_fee(&transaction)
            .await?;
        let initial_costs = SolActionCosts {
            fee_lamports: initial_fee,
            upfront_rent_lamports: temporary_rent,
            creates_canonical_wsol_account: false,
            sponsored,
        };
        let initial_availability = SolBalanceAvailability::from_costs(components, initial_costs)?;
        let initial_needed = amount_lamports
            .checked_add(initial_availability.reserve_lamports)
            .and_then(|required| required.checked_sub(components.native_lamports))
            .ok_or_else(|| {
                SdkError::Validation("invalid temporary withdrawal requirement".into())
            })?;

        transaction = build_temporary_native_withdraw_transaction(
            wallet,
            recipient,
            amount_lamports,
            initial_needed,
            temporary_rent,
            &seed,
            temporary_account,
        )?;
        transaction.message.recent_blockhash = seed_blockhash;
        let final_fee = self
            .client
            .estimate_prepared_transaction_fee(&transaction)
            .await?;
        let costs = SolActionCosts {
            fee_lamports: final_fee,
            upfront_rent_lamports: temporary_rent,
            creates_canonical_wsol_account: false,
            sponsored,
        };
        let availability = SolBalanceAvailability::from_costs(components, costs)?;
        let canonical_transfer = amount_lamports
            .checked_add(availability.reserve_lamports)
            .and_then(|required| required.checked_sub(components.native_lamports))
            .ok_or_else(|| {
                SdkError::Validation("invalid temporary withdrawal requirement".into())
            })?;
        if canonical_transfer > components.canonical_wsol_lamports {
            return Err(SdkError::Validation(
                "canonical WSOL cannot fund the native withdrawal shortfall".into(),
            ));
        }
        if canonical_transfer != initial_needed {
            transaction = build_temporary_native_withdraw_transaction(
                wallet,
                recipient,
                amount_lamports,
                canonical_transfer,
                temporary_rent,
                &seed,
                temporary_account,
            )?;
            transaction.message.recent_blockhash = seed_blockhash;
            let stable_fee = self
                .client
                .estimate_prepared_transaction_fee(&transaction)
                .await?;
            if stable_fee != final_fee {
                return Err(SdkError::Other(
                    "transaction fee changed while rebuilding native withdrawal".into(),
                ));
            }
        }
        Ok(SolActionPlan {
            kind: SolActionKind::NativeWithdraw,
            transaction,
            costs,
            availability,
            expected_delta: SolComponentDelta {
                // Temporary account rent returns on close; only the converted
                // amount and live fee change the wallet's native component.
                native_lamports: i128::from(canonical_transfer)
                    - i128::from(amount_lamports)
                    - i128::from(if sponsored { 0 } else { final_fee }),
                canonical_wsol_lamports: -i128::from(canonical_transfer),
            },
        })
    }

    /// Finish merge/redeem planning with live cost authority and component deltas.
    async fn finish_receive_plan(
        &self,
        kind: SolActionKind,
        amount_lamports: u64,
        components: crate::domain::position::SolBalanceComponents,
        sponsored: bool,
        creates_canonical_wsol_account: bool,
        upfront_rent_lamports: u64,
        transaction: &mut Transaction,
    ) -> Result<SolActionPlan, SdkError> {
        let fee_lamports = self
            .client
            .prepare_and_estimate_transaction_fee(transaction)
            .await?;
        let costs = SolActionCosts {
            fee_lamports,
            upfront_rent_lamports,
            creates_canonical_wsol_account,
            sponsored,
        };
        let availability = SolBalanceAvailability::from_costs(components, costs)?;
        let wallet_costs = if sponsored {
            0
        } else {
            fee_lamports
                .checked_add(upfront_rent_lamports)
                .ok_or_else(|| SdkError::Validation("SOL action costs overflow u64".into()))?
        };
        Ok(SolActionPlan {
            kind,
            transaction: transaction.clone(),
            costs,
            availability,
            expected_delta: SolComponentDelta {
                native_lamports: -i128::from(wallet_costs),
                canonical_wsol_lamports: i128::from(amount_lamports),
            },
        })
    }

    /// Resolve the authenticated wallet only from fresh matching cached authority.
    async fn planning_wallet(
        &self,
        state: &WalletDepositBalancesState,
    ) -> Result<Pubkey, SdkError> {
        // Cached identity is a transaction-planning trust boundary. Signing is
        // checked again by the client's submission path after Web rebuilds the
        // plan at its final account-operation boundary.
        let credentials = self.client.auth().credentials().await;
        let wallet = validated_conversion_wallet(credentials.as_ref(), state)?;
        let strategy = self.client.signing_strategy().await.ok_or_else(|| {
            SdkError::Validation("signing strategy is not set on the client".into())
        })?;
        validate_signing_wallet(&strategy, wallet)?;
        Ok(wallet)
    }

    /// Return the authenticated Trading Wallet after native-keypair validation.
    ///
    /// Complete wallet state is validated first. The configured signing strategy
    /// must then be a local keypair for that same wallet. Either failure occurs
    /// before conversion RPC reads.
    #[cfg(feature = "native-auth")]
    async fn native_conversion_planning_wallet(
        &self,
        state: &WalletDepositBalancesState,
    ) -> Result<Pubkey, SdkError> {
        let credentials = self.client.auth().credentials().await;
        let wallet = validated_conversion_wallet(credentials.as_ref(), state)?;
        let strategy = self.client.signing_strategy().await.ok_or_else(|| {
            SdkError::Validation("signing strategy is not set on the client".into())
        })?;
        validate_native_conversion_signing_wallet(&strategy, wallet)?;
        Ok(wallet)
    }

    // ── On-chain instruction builders ───────────────────────────────────

    /// Build RedeemWinnings instruction.
    pub fn redeem_winnings_ix(
        &self,
        params: &RedeemWinningsParams,
        outcome_index: u8,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_redeem_winnings_ix(params, outcome_index, pid)
    }

    /// Build RedeemWinnings transaction.
    pub fn redeem_winnings_tx(
        &self,
        params: RedeemWinningsParams,
        outcome_index: u8,
    ) -> Result<Transaction, SdkError> {
        let ix = self.redeem_winnings_ix(&params, outcome_index);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build a conditional-token withdrawal from a position instruction.
    pub fn withdraw_conditional_from_position_ix(
        &self,
        params: &WithdrawConditionalFromPositionParams,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_withdraw_conditional_from_position_ix(params, pid)
    }

    /// Build a conditional-token withdrawal from a position transaction.
    pub fn withdraw_conditional_from_position_tx(
        &self,
        params: WithdrawConditionalFromPositionParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.withdraw_conditional_from_position_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build a conditional-token withdrawal from a position instruction.
    ///
    /// Compatibility wrapper for the previous SDK method name.
    pub fn withdraw_from_position_ix(&self, params: &WithdrawFromPositionParams) -> Instruction {
        self.withdraw_conditional_from_position_ix(params)
    }

    /// Build a conditional-token withdrawal from a position transaction.
    ///
    /// Compatibility wrapper for the previous SDK method name.
    pub fn withdraw_from_position_tx(
        &self,
        params: WithdrawFromPositionParams,
    ) -> Result<Transaction, SdkError> {
        self.withdraw_conditional_from_position_tx(params)
    }

    /// Build InitPositionTokens instruction.
    pub fn init_position_tokens_ix(
        &self,
        params: &InitPositionTokensParams,
        num_outcomes: u8,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_init_position_tokens_ix(params, num_outcomes, pid)
    }

    /// Build InitPositionTokens transaction.
    pub fn init_position_tokens_tx(
        &self,
        params: InitPositionTokensParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let ix = self.init_position_tokens_ix(&params, num_outcomes);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.payer)))
    }

    /// Build ExtendPositionTokens instruction.
    pub fn extend_position_tokens_ix(
        &self,
        params: &ExtendPositionTokensParams,
        num_outcomes: u8,
    ) -> Result<Instruction, SdkError> {
        let pid = &self.client.program_id;
        Ok(instructions::build_extend_position_tokens_ix(
            params,
            num_outcomes,
            pid,
        )?)
    }

    /// Build ExtendPositionTokens transaction.
    pub fn extend_position_tokens_tx(
        &self,
        params: ExtendPositionTokensParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let ix = self.extend_position_tokens_ix(&params, num_outcomes)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }

    /// Build ClosePositionAlt instruction.
    pub fn close_position_alt_ix(&self, params: &ClosePositionAltParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_close_position_alt_ix(params, pid)
    }

    /// Build ClosePositionAlt transaction.
    pub fn close_position_alt_tx(
        &self,
        params: ClosePositionAltParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.close_position_alt_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }

    /// Build ClosePositionTokenAccounts instruction.
    pub fn close_position_token_accounts_ix(
        &self,
        params: &ClosePositionTokenAccountsParams,
        num_outcomes: u8,
    ) -> Result<Instruction, SdkError> {
        let pid = &self.client.program_id;
        Ok(instructions::build_close_position_token_accounts_ix(
            params,
            num_outcomes,
            pid,
        )?)
    }

    /// Build ClosePositionTokenAccounts transaction.
    pub fn close_position_token_accounts_tx(
        &self,
        params: ClosePositionTokenAccountsParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let ix = self.close_position_token_accounts_ix(&params, num_outcomes)?;
        Ok(Transaction::new_with_payer(&[ix], Some(&params.operator)))
    }

    /// Build DepositToGlobal instruction.
    pub fn deposit_to_global_ix(&self, params: &DepositToGlobalParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_deposit_to_global_ix(params, pid)
    }

    /// Build DepositToGlobal instruction with user deposit ALT create/extend accounts.
    pub fn deposit_to_global_ix_with_alt(
        &self,
        params: &DepositToGlobalParams,
        alt_context: DepositToGlobalAltContext,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_deposit_to_global_ix_with_alt(params, alt_context, pid)
    }

    /// Build DepositToGlobal transaction.
    pub fn deposit_to_global_tx(
        &self,
        params: DepositToGlobalParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.deposit_to_global_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build DepositToGlobal transaction with user deposit ALT create/extend accounts.
    pub fn deposit_to_global_tx_with_alt(
        &self,
        params: DepositToGlobalParams,
        alt_context: DepositToGlobalAltContext,
    ) -> Result<Transaction, SdkError> {
        let ix = self.deposit_to_global_ix_with_alt(&params, alt_context);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build GlobalToMarketDeposit instruction.
    pub fn global_to_market_deposit_ix(
        &self,
        params: &GlobalToMarketDepositParams,
        num_outcomes: u8,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_global_to_market_deposit_ix(params, num_outcomes, pid)
    }

    /// Build GlobalToMarketDeposit transaction.
    pub fn global_to_market_deposit_tx(
        &self,
        params: GlobalToMarketDepositParams,
        num_outcomes: u8,
    ) -> Result<Transaction, SdkError> {
        let ix = self.global_to_market_deposit_ix(&params, num_outcomes);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    /// Build WithdrawFromGlobal instruction.
    pub fn withdraw_from_global_ix(&self, params: &WithdrawFromGlobalParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_withdraw_from_global_ix(params, pid)
    }

    /// Build WithdrawFromGlobal transaction.
    pub fn withdraw_from_global_tx(
        &self,
        params: WithdrawFromGlobalParams,
    ) -> Result<Transaction, SdkError> {
        let ix = self.withdraw_from_global_ix(&params);
        Ok(Transaction::new_with_payer(&[ix], Some(&params.user)))
    }

    // ── Builder factories ──────────────────────────────────────────────

    /// Create a deposit builder pre-seeded with the client's deposit source.
    ///
    /// Use `.build_ix()` or `.build_tx()` to produce the final instruction/transaction.
    pub async fn deposit(&self) -> DepositBuilder<'a> {
        let deposit_source = self.client.deposit_source().await;
        DepositBuilder::new(self.client, deposit_source)
    }

    /// Create a merge builder.
    ///
    /// Burns a complete set of conditional tokens and releases collateral.
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn merge(&self) -> MergeBuilder<'a> {
        MergeBuilder::new(self.client)
    }

    /// Create a withdraw builder pre-seeded with the client's deposit source.
    ///
    /// Dispatches based on deposit source:
    /// - **Global**: withdraws from global deposit pool
    /// - **Market**: withdraws conditional tokens from a position ATA
    ///
    /// Use `.build_ix()` or `.build_tx()` to produce the final instruction/transaction.
    pub async fn withdraw(&self) -> WithdrawBuilder<'a> {
        let deposit_source = self.client.deposit_source().await;
        WithdrawBuilder::new(self.client, deposit_source)
    }

    /// Create a redeem winnings builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn redeem_winnings(&self) -> RedeemWinningsBuilder<'a> {
        RedeemWinningsBuilder::new(self.client)
    }

    /// Create a conditional-token withdraw-from-position builder.
    /// Set `.num_outcomes(...)` before building because this path only receives a market pubkey.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn withdraw_from_position(&self) -> WithdrawFromPositionBuilder<'a> {
        WithdrawFromPositionBuilder::new(self.client)
    }

    /// Create a conditional-token withdraw-from-position builder.
    pub fn withdraw_conditional_from_position(&self) -> WithdrawFromPositionBuilder<'a> {
        WithdrawFromPositionBuilder::new(self.client)
    }

    /// Create an init-position-tokens builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn init_position_tokens(&self) -> InitPositionTokensBuilder<'a> {
        InitPositionTokensBuilder::new(self.client)
    }

    /// Create an extend-position-tokens builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn extend_position_tokens(&self) -> ExtendPositionTokensBuilder<'a> {
        ExtendPositionTokensBuilder::new(self.client)
    }

    /// Create a deposit-to-global builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn deposit_to_global(&self) -> DepositToGlobalBuilder<'a> {
        DepositToGlobalBuilder::new(self.client)
    }

    /// Create a withdraw-from-global builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn withdraw_from_global(&self) -> WithdrawFromGlobalBuilder<'a> {
        WithdrawFromGlobalBuilder::new(self.client)
    }

    /// Create a global-to-market deposit builder.
    ///
    /// Use `.build_ix()`, `.build_tx()`, or `.sign_and_submit()` to produce the final result.
    pub fn global_to_market_deposit(&self) -> GlobalToMarketDepositBuilder<'a> {
        GlobalToMarketDepositBuilder::new(self.client)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// On-chain account fetchers (require RPC)
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "solana-rpc")]
impl<'a> Positions<'a> {
    /// Fetch a Position account (returns None if not found).
    pub async fn get_onchain(
        &self,
        owner: &Pubkey,
        market: &Pubkey,
    ) -> Result<Option<crate::program::accounts::Position>, SdkError> {
        let rpc = crate::rpc::resolve_solana_rpc(self.client).await?;
        let pda = self.pda(owner, market);
        match rpc.get_account(&pda).await {
            Ok(account) => Ok(Some(crate::program::accounts::Position::deserialize(
                &account.data,
            )?)),
            Err(_) => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{deposit_token_balances_query, validated_conversion_wallet};
    use crate::{auth::AuthCredentials, domain::position::WalletDepositBalancesState};
    use chrono::{Duration, Utc};
    use solana_pubkey::Pubkey;

    #[cfg(feature = "native")]
    use {
        crate::{
            client::LightconeClient,
            domain::market::{Market, Status},
            domain::position::{builders, DepositTokenBalance, WRAPPED_SOL_MINT_ADDRESS},
            shared::PubkeyStr,
        },
        rust_decimal::Decimal,
        solana_keypair::Keypair,
        solana_signer::Signer,
        solana_transaction::Transaction,
        std::{
            collections::{HashMap, VecDeque},
            sync::{Arc, Mutex},
        },
        tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpListener,
        },
    };

    #[cfg(feature = "native")]
    /// Start a deterministic local JSON-RPC stub and retain received requests.
    async fn spawn_rpc_server(
        responses: Vec<serde_json::Value>,
    ) -> (String, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let responses = Arc::new(Mutex::new(VecDeque::from(responses)));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let server_responses = Arc::clone(&responses);
        let server_requests = Arc::clone(&requests);

        tokio::spawn(async move {
            loop {
                let Ok((mut socket, _)) = listener.accept().await else {
                    return;
                };
                let mut buffer = [0_u8; 16_384];
                let Ok(bytes_read) = socket.read(&mut buffer).await else {
                    return;
                };
                server_requests
                    .lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&buffer[..bytes_read]).into_owned());
                let body = server_responses
                    .lock()
                    .unwrap()
                    .pop_front()
                    .expect("unexpected RPC request")
                    .to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                socket.write_all(response.as_bytes()).await.unwrap();
            }
        });

        (format!("http://{address}"), requests)
    }

    #[cfg(feature = "native")]
    /// Build complete wallet authority against a deterministic RPC endpoint.
    fn planning_client(
        rpc_url: &str,
        native_sol_balance: &str,
    ) -> (LightconeClient, WalletDepositBalancesState, Pubkey) {
        planning_client_with_keypair(rpc_url, native_sol_balance, Keypair::new())
    }

    #[cfg(feature = "native")]
    fn planning_client_with_keypair(
        rpc_url: &str,
        native_sol_balance: &str,
        keypair: Keypair,
    ) -> (LightconeClient, WalletDepositBalancesState, Pubkey) {
        let wallet = keypair.pubkey();
        let wallet_address = PubkeyStr::from(wallet.to_string());
        let client = LightconeClient::builder()
            .auth(AuthCredentials {
                user_id: "user-a".into(),
                wallet_address: wallet_address.clone(),
                expires_at: Utc::now() + Duration::minutes(1),
            })
            .native_signer(keypair)
            .rpc_url(rpc_url)
            .build()
            .unwrap();
        let state = WalletDepositBalancesState {
            wallet_address: Some(wallet_address),
            context_slot: Some(1),
            native_sol_balance: Some(native_sol_balance.into()),
            ..Default::default()
        };
        (client, state, wallet)
    }

    #[cfg(feature = "native")]
    fn canonical_account_response(wallet: Pubkey, program_owner: Pubkey) -> serde_json::Value {
        canonical_account_response_with(wallet, program_owner, 0, 2_039_280, 1)
    }

    #[cfg(feature = "native")]
    /// Encode exact token/account lamports and token state in an RPC account fixture.
    fn canonical_account_response_with(
        wallet: Pubkey,
        program_owner: Pubkey,
        token_amount_lamports: u64,
        account_lamports: u64,
        account_state: u8,
    ) -> serde_json::Value {
        canonical_account_response_with_details(
            wallet,
            program_owner,
            token_amount_lamports,
            2_039_280,
            account_lamports,
            account_state,
            None,
        )
    }

    #[cfg(feature = "native")]
    /// Encode native reserve and optional close authority for validation fixtures.
    fn canonical_account_response_with_details(
        wallet: Pubkey,
        program_owner: Pubkey,
        token_amount_lamports: u64,
        native_reserve_lamports: u64,
        account_lamports: u64,
        account_state: u8,
        close_authority: Option<Pubkey>,
    ) -> serde_json::Value {
        canonical_account_response_with_token_fields(
            program_owner,
            spl_token_interface::native_mint::id(),
            wallet,
            token_amount_lamports,
            Some(native_reserve_lamports),
            account_lamports,
            account_state,
            close_authority,
        )
    }

    #[cfg(feature = "native")]
    /// Encode independently variable SPL token fields for negative RPC fixtures.
    fn canonical_account_response_with_token_fields(
        program_owner: Pubkey,
        mint: Pubkey,
        token_authority: Pubkey,
        token_amount_lamports: u64,
        native_reserve_lamports: Option<u64>,
        account_lamports: u64,
        account_state: u8,
        close_authority: Option<Pubkey>,
    ) -> serde_json::Value {
        let mut data = vec![0_u8; super::TOKEN_ACCOUNT_SPACE];
        data[..32].copy_from_slice(mint.as_ref());
        data[32..64].copy_from_slice(token_authority.as_ref());
        data[64..72].copy_from_slice(&token_amount_lamports.to_le_bytes());
        data[108] = account_state;
        if let Some(native_reserve_lamports) = native_reserve_lamports {
            data[109..113].copy_from_slice(&1_u32.to_le_bytes());
            data[113..121].copy_from_slice(&native_reserve_lamports.to_le_bytes());
        }
        if let Some(close_authority) = close_authority {
            data[129..133].copy_from_slice(&1_u32.to_le_bytes());
            data[133..165].copy_from_slice(close_authority.as_ref());
        }
        canonical_account_response_with_data(program_owner, account_lamports, data)
    }

    #[cfg(feature = "native")]
    /// Wrap arbitrary account bytes in the confirmed `getAccountInfo` fixture envelope.
    fn canonical_account_response_with_data(
        program_owner: Pubkey,
        account_lamports: u64,
        data: Vec<u8>,
    ) -> serde_json::Value {
        let space = data.len();
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data);
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "context": {"slot": 1},
                "value": {
                    "data": [encoded, "base64"],
                    "executable": false,
                    "lamports": account_lamports,
                    "owner": program_owner.to_string(),
                    "rentEpoch": 0,
                    "space": space,
                }
            }
        })
    }

    #[cfg(feature = "native")]
    fn missing_account_response() -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"context": {"slot": 1}, "value": null}
        })
    }

    #[cfg(feature = "native")]
    fn blockhash_response() -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "context": {"slot": 1},
                "value": {
                    "blockhash": solana_hash::Hash::new_unique().to_string(),
                    "lastValidBlockHeight": 100
                }
            }
        })
    }

    #[cfg(feature = "native")]
    fn fee_response(fee_lamports: Option<u64>) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"context": {"slot": 1}, "value": fee_lamports}
        })
    }

    #[cfg(feature = "native")]
    fn set_canonical_balance(state: &mut WalletDepositBalancesState, lamports: u64) {
        let mint = PubkeyStr::from(WRAPPED_SOL_MINT_ADDRESS);
        state.balances.insert(
            mint.clone(),
            DepositTokenBalance {
                mint,
                idle: Decimal::from_i128_with_scale(i128::from(lamports), 9),
                symbol: "WSOL".into(),
                name: "Wrapped SOL".into(),
                icon_url_low: None,
                icon_url_medium: None,
                icon_url_high: None,
            },
        );
    }

    #[cfg(feature = "native")]
    /// Return the smallest active market shape accepted by split planning.
    fn market() -> Market {
        Market {
            id: 1,
            pubkey: PubkeyStr::from(Pubkey::new_unique().to_string()),
            name: "Market".into(),
            banner_image_url_low: None,
            banner_image_url_medium: None,
            banner_image_url_high: None,
            icon_url_low: String::new(),
            icon_url_medium: String::new(),
            icon_url_high: String::new(),
            featured_rank: None,
            slug: "market".into(),
            status: Status::Active,
            maker_fee_bps: 0,
            taker_fee_bps: 0,
            created_at: Utc::now(),
            activated_at: None,
            settled_at: None,
            resolution_by: None,
            resolution: None,
            description: None,
            definition: "Test market".into(),
            category: None,
            subcategory: None,
            tags: Vec::new(),
            num_outcomes: 2,
            deposit_assets: Vec::new(),
            deposit_asset_pairs: Vec::new(),
            conditional_tokens: Vec::new(),
            outcomes: Vec::new(),
            orderbook_pairs: Vec::new(),
            orderbook_ids: Vec::new(),
            token_metadata: HashMap::new(),
        }
    }

    #[cfg(feature = "native")]
    struct IdentifiedExternalSigner(Pubkey);

    #[cfg(feature = "native")]
    impl crate::shared::signing::ExternalSigner for IdentifiedExternalSigner {
        fn wallet_address(&self) -> Option<Pubkey> {
            Some(self.0)
        }

        fn sign_message<'a>(
            &'a self,
            _message: &'a [u8],
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + 'a>>
        {
            Box::pin(async { Err("not used by planner test".into()) })
        }

        fn sign_transaction<'a>(
            &'a self,
            _transaction: &'a [u8],
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + 'a>>
        {
            Box::pin(async { Err("not used by planner test".into()) })
        }
    }

    #[test]
    fn deposit_token_balances_query_includes_optional_minimum_context_slot() {
        assert!(deposit_token_balances_query(None).is_empty());
        assert_eq!(
            deposit_token_balances_query(Some(1234)),
            vec![("min_context_slot", "1234".to_string())]
        );
    }

    #[test]
    fn conversion_wallet_requires_unexpired_matching_initialized_state() {
        let wallet = Pubkey::new_unique();
        let wallet_address = crate::shared::PubkeyStr::from(wallet.to_string());
        let credentials = AuthCredentials {
            user_id: "user-a".into(),
            wallet_address: wallet_address.clone(),
            expires_at: Utc::now() + Duration::minutes(1),
        };
        let state = WalletDepositBalancesState {
            wallet_address: Some(wallet_address),
            context_slot: Some(1),
            native_sol_balance: Some("1.000000000".into()),
            ..Default::default()
        };

        assert_eq!(
            validated_conversion_wallet(Some(&credentials), &state).unwrap(),
            wallet
        );
        assert!(validated_conversion_wallet(None, &state).is_err());

        let mut expired = credentials.clone();
        expired.expires_at = Utc::now() - Duration::minutes(1);
        assert!(validated_conversion_wallet(Some(&expired), &state).is_err());

        let mut mismatched = credentials.clone();
        mismatched.wallet_address = Pubkey::new_unique().to_string().into();
        assert!(validated_conversion_wallet(Some(&mismatched), &state).is_err());
        assert!(validated_conversion_wallet(
            Some(&credentials),
            &WalletDepositBalancesState::default()
        )
        .is_err());
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    /// Uses the live direct fee in both availability and expected native delta.
    async fn native_withdraw_plan_uses_live_fee_and_direct_component_delta() {
        let blockhash = solana_hash::Hash::new_unique().to_string();
        let (rpc_url, requests) = spawn_rpc_server(vec![
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "context": {"slot": 1},
                    "value": {"blockhash": blockhash, "lastValidBlockHeight": 100}
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"context": {"slot": 1}, "value": 5000}
            }),
        ])
        .await;
        let (client, state, _) = planning_client(&rpc_url, "1.000000000");

        let plan = client
            .positions()
            .plan_native_sol_withdrawal(Pubkey::new_unique(), 500_000_000, &state, false)
            .await
            .unwrap();

        assert_eq!(plan.kind, super::SolActionKind::NativeWithdraw);
        assert_eq!(plan.transaction.message.instructions.len(), 1);
        assert_eq!(plan.costs.fee_lamports, 5_000);
        assert_eq!(plan.availability.reserve_lamports, 1_000_000);
        assert_eq!(plan.expected_delta.native_lamports, -500_005_000);
        assert_eq!(plan.expected_delta.canonical_wsol_lamports, 0);
        let requests = requests.lock().unwrap();
        assert!(requests[0].contains("getLatestBlockhash"));
        assert!(requests[1].contains("getFeeForMessage"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    /// Binds temporary derivation to the blockhash retained by the final plan.
    async fn temporary_withdraw_seed_uses_the_planned_transaction_blockhash() {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let direct_blockhash = solana_hash::Hash::new_unique();
        let planned_blockhash = solana_hash::Hash::new_unique();
        let latest_blockhash = |blockhash: solana_hash::Hash| {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "context": {"slot": 1},
                    "value": {
                        "blockhash": blockhash.to_string(),
                        "lastValidBlockHeight": 100
                    }
                }
            })
        };
        let fee = || {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"context": {"slot": 1}, "value": 5000}
            })
        };
        let (rpc_url, requests) = spawn_rpc_server(vec![
            latest_blockhash(direct_blockhash),
            fee(),
            canonical_account_response(wallet, spl_token_interface::id()),
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": 2_039_280}),
            latest_blockhash(planned_blockhash),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"context": {"slot": 1}, "value": null}
            }),
            fee(),
            fee(),
        ])
        .await;
        let (client, mut state, wallet) =
            planning_client_with_keypair(&rpc_url, "0.010000000", keypair);
        let sol_mint = PubkeyStr::from(WRAPPED_SOL_MINT_ADDRESS);
        state.balances.insert(
            sol_mint.clone(),
            DepositTokenBalance {
                mint: sol_mint,
                idle: Decimal::ONE,
                symbol: "WSOL".into(),
                name: "Wrapped SOL".into(),
                icon_url_low: None,
                icon_url_medium: None,
                icon_url_high: None,
            },
        );
        let recipient = Pubkey::new_unique();

        let plan = client
            .positions()
            .plan_native_sol_withdrawal(recipient, 500_000_000, &state, false)
            .await
            .unwrap();

        assert_eq!(plan.transaction.message.recent_blockhash, planned_blockhash);
        let seed = builders::native_withdraw_seed(
            &plan.transaction.message.recent_blockhash,
            &wallet,
            &recipient,
            500_000_000,
            0,
        );
        let temporary = builders::temporary_wsol_account(&wallet, &seed).unwrap();
        assert!(plan.transaction.message.account_keys.contains(&temporary));
        assert_eq!(
            requests
                .lock()
                .unwrap()
                .iter()
                .filter(|request| request.contains("getLatestBlockhash"))
                .count(),
            2
        );
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    /// Includes missing-ATA rent and creation in one atomic split plan.
    async fn split_plan_creates_missing_canonical_account_and_reserves_rent() {
        let blockhash = solana_hash::Hash::new_unique().to_string();
        let (rpc_url, requests) = spawn_rpc_server(vec![
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"context": {"slot": 1}, "value": null}
            }),
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": 2_039_280}),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "context": {"slot": 1},
                    "value": {"blockhash": blockhash, "lastValidBlockHeight": 100}
                }
            }),
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {"context": {"slot": 1}, "value": 5000}
            }),
        ])
        .await;
        let (client, state, _) = planning_client(&rpc_url, "1.000000000");

        let plan = client
            .positions()
            .plan_sol_split(&market(), 500_000_000, &state, false)
            .await
            .unwrap();

        assert_eq!(plan.kind, super::SolActionKind::Split);
        assert_eq!(plan.transaction.message.instructions.len(), 4);
        assert_eq!(plan.costs.upfront_rent_lamports, 2_039_280);
        assert_eq!(plan.availability.reserve_lamports, 3_500_000);
        assert_eq!(plan.expected_delta.native_lamports, -502_044_280);
        assert_eq!(plan.expected_delta.canonical_wsol_lamports, 0);
        let requests = requests.lock().unwrap();
        assert!(requests[0].contains("getAccountInfo"));
        assert!(requests[1].contains("getMinimumBalanceForRentExemption"));
        assert!(requests[2].contains("getLatestBlockhash"));
        assert!(requests[3].contains("getFeeForMessage"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn exact_account_inspection_accepts_safe_close_authorities_and_preserves_boolean_api(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let no_close_authority = canonical_account_response_with(
            wallet,
            spl_token_interface::id(),
            50_000_000,
            52_049_280,
            1,
        );
        let wallet_close_authority = canonical_account_response_with_details(
            wallet,
            spl_token_interface::id(),
            50_000_000,
            2_039_280,
            52_049_280,
            1,
            Some(wallet),
        );
        let (rpc_url, requests) = spawn_rpc_server(vec![
            no_close_authority,
            wallet_close_authority.clone(),
            wallet_close_authority,
        ])
        .await;
        let (client, _, _) = planning_client_with_keypair(&rpc_url, "1.000000000", keypair);
        let (_, canonical) = builders::wrapped_sol_accounts(&wallet)?;

        let no_authority_info = client
            .canonical_wsol_account_info(&canonical, &wallet)
            .await?
            .ok_or("canonical account should exist")?;
        assert_eq!(no_authority_info.account_lamports, 52_049_280);
        assert_eq!(no_authority_info.token_amount_lamports, 50_000_000);
        assert_eq!(no_authority_info.native_reserve_lamports, 2_039_280);
        let wallet_authority_info = client
            .canonical_wsol_account_info(&canonical, &wallet)
            .await?
            .ok_or("canonical account should exist")?;
        assert_eq!(wallet_authority_info, no_authority_info);
        assert!(
            client
                .canonical_wsol_account_exists(&canonical, &wallet)
                .await?
        );
        assert_eq!(
            requests
                .lock()
                .map_err(|_| "requests mutex should not be poisoned")?
                .len(),
            3
        );
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn exact_account_inspection_rejects_noncanonical_address_before_rpc() {
        let (client, _, wallet) = planning_client("http://127.0.0.1:1", "1.000000000");

        let error = client
            .canonical_wsol_account_info(&Pubkey::new_unique(), &wallet)
            .await
            .unwrap_err();

        assert!(error.to_string().contains("Tokenkeg native-mint ATA"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn exact_account_inspection_rejects_wrong_close_authority(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, _) = spawn_rpc_server(vec![canonical_account_response_with_details(
            wallet,
            spl_token_interface::id(),
            50_000_000,
            2_039_280,
            52_039_280,
            1,
            Some(Pubkey::new_unique()),
        )])
        .await;
        let (client, _, _) = planning_client_with_keypair(&rpc_url, "1.000000000", keypair);
        let (_, canonical) = builders::wrapped_sol_accounts(&wallet)?;

        let error = client
            .canonical_wsol_account_info(&canonical, &wallet)
            .await
            .unwrap_err();

        assert!(error.to_string().contains("close authority"));
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn exact_account_inspection_rejects_incompatible_token_state_and_length(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let valid_owner = spl_token_interface::id();
        let native_mint = spl_token_interface::native_mint::id();
        let responses = vec![
            canonical_account_response_with_token_fields(
                valid_owner,
                Pubkey::new_unique(),
                wallet,
                0,
                Some(2_039_280),
                2_039_280,
                1,
                None,
            ),
            canonical_account_response_with_token_fields(
                valid_owner,
                native_mint,
                Pubkey::new_unique(),
                0,
                Some(2_039_280),
                2_039_280,
                1,
                None,
            ),
            canonical_account_response_with_token_fields(
                valid_owner,
                native_mint,
                wallet,
                0,
                Some(2_039_280),
                2_039_280,
                0,
                None,
            ),
            canonical_account_response_with_token_fields(
                valid_owner,
                native_mint,
                wallet,
                0,
                None,
                2_039_280,
                1,
                None,
            ),
            canonical_account_response_with_data(valid_owner, 0, vec![0; 164]),
        ];
        let (rpc_url, _) = spawn_rpc_server(responses).await;
        let (client, _, _) = planning_client_with_keypair(&rpc_url, "1.000000000", keypair);
        let (_, canonical) = builders::wrapped_sol_accounts(&wallet)?;

        for expected in [
            "incompatible mint, authority, or native state",
            "incompatible mint, authority, or native state",
            "token account is invalid",
            "incompatible mint, authority, or native state",
            "token account is invalid",
        ] {
            let error = client
                .canonical_wsol_account_info(&canonical, &wallet)
                .await
                .unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
        }
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn wrap_plan_creates_missing_account_with_live_costs_above_floor(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (rpc_url, requests) = spawn_rpc_server(vec![
            missing_account_response(),
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": 2_039_280}),
            blockhash_response(),
            fee_response(Some(2_000_000)),
        ])
        .await;
        let (client, state, _) = planning_client(&rpc_url, "1.000000000");

        let plan = client
            .positions()
            .plan_wrap_sol(100_000_000, &state)
            .await?;

        assert_eq!(plan.kind, super::SolActionKind::Wrap);
        assert_eq!(plan.transaction.message.instructions.len(), 3);
        assert_eq!(
            plan.costs,
            crate::domain::position::SolActionCosts {
                fee_lamports: 2_000_000,
                upfront_rent_lamports: 2_039_280,
                creates_canonical_wsol_account: true,
                sponsored: false,
            }
        );
        assert_eq!(plan.availability.reserve_lamports, 4_039_280);
        assert_eq!(plan.expected_delta.native_lamports, -104_039_280);
        assert_eq!(plan.expected_delta.canonical_wsol_lamports, 100_000_000);
        let requests = requests
            .lock()
            .map_err(|_| "requests mutex should not be poisoned")?;
        assert!(requests[0].contains("getAccountInfo"));
        assert!(requests[1].contains("getMinimumBalanceForRentExemption"));
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn wrap_plan_reuses_exact_matching_account_and_live_existing_floor(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, _) = spawn_rpc_server(vec![
            canonical_account_response_with(
                wallet,
                spl_token_interface::id(),
                50_000_000,
                52_039_280,
                1,
            ),
            blockhash_response(),
            fee_response(Some(1_500_000)),
        ])
        .await;
        let (client, mut state, _) = planning_client_with_keypair(&rpc_url, "1.000000000", keypair);
        set_canonical_balance(&mut state, 50_000_000);

        let plan = client
            .positions()
            .plan_wrap_sol(100_000_000, &state)
            .await?;

        assert_eq!(plan.transaction.message.instructions.len(), 2);
        assert_eq!(plan.costs.upfront_rent_lamports, 0);
        assert!(!plan.costs.creates_canonical_wsol_account);
        assert_eq!(plan.availability.reserve_lamports, 1_500_000);
        assert_eq!(plan.expected_delta.native_lamports, -101_500_000);
        assert_eq!(plan.expected_delta.canonical_wsol_lamports, 100_000_000);
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn wrap_plan_rejects_unsynchronized_donated_lamports_before_fee_rpc(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, requests) = spawn_rpc_server(vec![canonical_account_response_with(
            wallet,
            spl_token_interface::id(),
            50_000_000,
            52_049_280,
            1,
        )])
        .await;
        let (client, mut state, _) = planning_client_with_keypair(&rpc_url, "1.000000000", keypair);
        set_canonical_balance(&mut state, 50_000_000);

        let error = client
            .positions()
            .plan_wrap_sol(100_000_000, &state)
            .await
            .unwrap_err();

        assert!(error.to_string().contains("unsynchronized excess lamports"));
        let requests = requests
            .lock()
            .map_err(|_| "requests mutex should not be poisoned")?;
        assert_eq!(requests.len(), 1);
        assert!(requests[0].contains("getAccountInfo"));
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn wrap_plan_rejects_existing_canonical_u64_overflow_before_fee_rpc(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let token_lamports = 9_000_000_000_000_000;
        let (rpc_url, requests) = spawn_rpc_server(vec![canonical_account_response_with(
            wallet,
            spl_token_interface::id(),
            token_lamports,
            token_lamports + 2_039_280,
            1,
        )])
        .await;
        let (client, mut state, _) =
            planning_client_with_keypair(&rpc_url, "18446744073.709551615", keypair);
        set_canonical_balance(&mut state, token_lamports);

        let error = client
            .positions()
            .plan_wrap_sol(u64::MAX - token_lamports + 1, &state)
            .await
            .unwrap_err();

        assert!(error
            .to_string()
            .contains("canonical WSOL token or account u64 range"));
        let requests = requests
            .lock()
            .map_err(|_| "requests mutex should not be poisoned")?;
        assert_eq!(requests.len(), 1);
        assert!(requests[0].contains("getAccountInfo"));
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn wrap_plan_requires_native_amount_plus_reserve() {
        let (rpc_url, _) = spawn_rpc_server(vec![
            missing_account_response(),
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": 2_039_280}),
            blockhash_response(),
            fee_response(Some(5_000)),
        ])
        .await;
        let (client, state, _) = planning_client(&rpc_url, "0.100000000");

        let error = client
            .positions()
            .plan_wrap_sol(99_000_000, &state)
            .await
            .unwrap_err();

        assert!(error
            .to_string()
            .contains("cannot fund the wrap amount and transaction reserve"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn unwrap_all_accepts_unsynchronized_donation_and_credits_complete_account(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, _) = spawn_rpc_server(vec![
            canonical_account_response_with(
                wallet,
                spl_token_interface::id(),
                50_000_000,
                52_049_280,
                1,
            ),
            blockhash_response(),
            fee_response(Some(5_000)),
        ])
        .await;
        let (client, mut state, _) = planning_client_with_keypair(&rpc_url, "0.000010000", keypair);
        set_canonical_balance(&mut state, 50_000_000);

        let plan = client.positions().plan_unwrap_wsol_all(&state).await?;

        assert_eq!(plan.kind, super::SolActionKind::UnwrapAll);
        assert_eq!(plan.transaction.message.instructions.len(), 1);
        assert_eq!(
            plan.costs,
            crate::domain::position::SolActionCosts {
                fee_lamports: 5_000,
                upfront_rent_lamports: 0,
                creates_canonical_wsol_account: false,
                sponsored: false,
            }
        );
        assert_eq!(plan.availability.components.native_lamports, 10_000);
        assert_eq!(
            plan.availability.components.canonical_wsol_lamports,
            50_000_000
        );
        assert_eq!(plan.availability.displayed_lamports, 50_010_000);
        assert_eq!(plan.availability.reserve_lamports, 5_000);
        assert_eq!(plan.availability.spendable_lamports, 50_005_000);
        assert_eq!(plan.expected_delta.native_lamports, 52_044_280);
        assert_eq!(plan.expected_delta.canonical_wsol_lamports, -50_000_000);
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn unwrap_all_rejects_projected_native_overflow() {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, _) = spawn_rpc_server(vec![
            canonical_account_response_with(wallet, spl_token_interface::id(), 1, 2_039_281, 1),
            blockhash_response(),
            fee_response(Some(1)),
        ])
        .await;
        let (client, mut state, _) =
            planning_client_with_keypair(&rpc_url, "18446744073.709551614", keypair);
        set_canonical_balance(&mut state, 1);

        let error = client
            .positions()
            .plan_unwrap_wsol_all(&state)
            .await
            .unwrap_err();

        assert!(error
            .to_string()
            .contains("projected native balance overflows u64"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn unwrap_all_rejects_wrong_close_authority_before_fee_planning(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, requests) = spawn_rpc_server(vec![canonical_account_response_with_details(
            wallet,
            spl_token_interface::id(),
            50_000_000,
            2_039_280,
            52_039_280,
            1,
            Some(Pubkey::new_unique()),
        )])
        .await;
        let (client, mut state, _) = planning_client_with_keypair(&rpc_url, "1.000000000", keypair);
        set_canonical_balance(&mut state, 50_000_000);

        let error = client
            .positions()
            .plan_unwrap_wsol_all(&state)
            .await
            .unwrap_err();

        assert!(error.to_string().contains("close authority"));
        let requests = requests
            .lock()
            .map_err(|_| "requests mutex should not be poisoned")?;
        assert_eq!(requests.len(), 1);
        assert!(requests[0].contains("getAccountInfo"));
        Ok(())
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn unwrap_all_rejects_impossible_native_accounting() {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, _) = spawn_rpc_server(vec![canonical_account_response_with_details(
            wallet,
            spl_token_interface::id(),
            u64::MAX,
            1,
            u64::MAX,
            1,
            None,
        )])
        .await;
        let (client, mut state, _) = planning_client_with_keypair(&rpc_url, "1.000000000", keypair);
        set_canonical_balance(&mut state, u64::MAX);
        let overflow = client
            .positions()
            .plan_unwrap_wsol_all(&state)
            .await
            .unwrap_err();
        assert!(overflow
            .to_string()
            .contains("token amount plus native reserve overflows u64"));

        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, _) = spawn_rpc_server(vec![canonical_account_response_with_details(
            wallet,
            spl_token_interface::id(),
            50,
            100,
            149,
            1,
            None,
        )])
        .await;
        let (client, mut state, _) = planning_client_with_keypair(&rpc_url, "1.000000000", keypair);
        set_canonical_balance(&mut state, 50);
        let underfunded = client
            .positions()
            .plan_unwrap_wsol_all(&state)
            .await
            .unwrap_err();
        assert!(underfunded
            .to_string()
            .contains("accounted lamports exceed RPC account lamports"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn conversion_planners_reject_zero_overflow_stale_and_missing_state() {
        let (client, state, _) = planning_client("http://127.0.0.1:1", "1.000000000");
        let zero_wrap = client
            .positions()
            .plan_wrap_sol(0, &state)
            .await
            .unwrap_err();
        assert!(zero_wrap.to_string().contains("greater than zero"));
        let zero_unwrap = client
            .positions()
            .plan_unwrap_wsol_all(&state)
            .await
            .unwrap_err();
        assert!(zero_unwrap.to_string().contains("greater than zero"));

        let (rpc_url, _) = spawn_rpc_server(vec![
            missing_account_response(),
            serde_json::json!({"jsonrpc": "2.0", "id": 1, "result": 0}),
            blockhash_response(),
            fee_response(Some(0)),
        ])
        .await;
        let (client, state, _) = planning_client(&rpc_url, "18446744073.709551615");
        let overflow = client
            .positions()
            .plan_wrap_sol(u64::MAX, &state)
            .await
            .unwrap_err();
        assert!(overflow.to_string().contains("overflows u64"));

        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, _) = spawn_rpc_server(vec![canonical_account_response_with(
            wallet,
            spl_token_interface::id(),
            2,
            2_039_282,
            1,
        )])
        .await;
        let (client, mut state, _) = planning_client_with_keypair(&rpc_url, "1.000000000", keypair);
        set_canonical_balance(&mut state, 1);
        let stale = client
            .positions()
            .plan_wrap_sol(1, &state)
            .await
            .unwrap_err();
        assert!(stale.to_string().contains("does not match"));

        let (rpc_url, _) = spawn_rpc_server(vec![missing_account_response()]).await;
        let (client, mut state, _) = planning_client(&rpc_url, "1.000000000");
        set_canonical_balance(&mut state, 1);
        let missing = client
            .positions()
            .plan_unwrap_wsol_all(&state)
            .await
            .unwrap_err();
        assert!(missing.to_string().contains("account is required"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn both_conversion_planners_reject_external_signers_before_rpc() {
        let (client, mut state, wallet) = planning_client("http://127.0.0.1:1", "1.000000000");
        set_canonical_balance(&mut state, 1);
        client
            .set_signing_strategy(crate::shared::signing::SigningStrategy::WalletAdapter(
                Arc::new(IdentifiedExternalSigner(wallet)),
            ))
            .await;

        let wrap_error = client
            .positions()
            .plan_wrap_sol(1, &state)
            .await
            .unwrap_err();
        assert!(wrap_error.to_string().contains("native keypair signing"));
        let unwrap_error = client
            .positions()
            .plan_unwrap_wsol_all(&state)
            .await
            .unwrap_err();
        assert!(unwrap_error.to_string().contains("native keypair signing"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn both_conversion_planners_reject_mismatched_native_keypairs_before_rpc() {
        let (client, mut state, _) = planning_client("http://127.0.0.1:1", "1.000000000");
        set_canonical_balance(&mut state, 1);
        client
            .set_signing_strategy(crate::shared::signing::SigningStrategy::Native(Arc::new(
                Keypair::new(),
            )))
            .await;

        let wrap_error = client
            .positions()
            .plan_wrap_sol(1, &state)
            .await
            .unwrap_err();
        assert!(wrap_error
            .to_string()
            .contains("native signing keypair does not control authenticated wallet"));
        let unwrap_error = client
            .positions()
            .plan_unwrap_wsol_all(&state)
            .await
            .unwrap_err();
        assert!(unwrap_error
            .to_string()
            .contains("native signing keypair does not control authenticated wallet"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn unwrap_all_rejects_invalid_account_fee_unavailability_and_fee_shortfall() {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, _) = spawn_rpc_server(vec![canonical_account_response_with(
            wallet,
            spl_token_interface::id(),
            1,
            2_039_281,
            2,
        )])
        .await;
        let (client, mut state, _) = planning_client_with_keypair(&rpc_url, "1.000000000", keypair);
        set_canonical_balance(&mut state, 1);
        let invalid = client
            .positions()
            .plan_unwrap_wsol_all(&state)
            .await
            .unwrap_err();
        assert!(invalid
            .to_string()
            .contains("incompatible mint, authority, or native state"));

        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, _) = spawn_rpc_server(vec![
            canonical_account_response_with(wallet, spl_token_interface::id(), 1, 2_039_281, 1),
            blockhash_response(),
            fee_response(None),
        ])
        .await;
        let (client, mut state, _) = planning_client_with_keypair(&rpc_url, "1.000000000", keypair);
        set_canonical_balance(&mut state, 1);
        let unavailable = client
            .positions()
            .plan_unwrap_wsol_all(&state)
            .await
            .unwrap_err();
        assert!(unavailable
            .to_string()
            .contains("fee estimate is unavailable"));

        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, _) = spawn_rpc_server(vec![
            canonical_account_response_with(wallet, spl_token_interface::id(), 1, 2_039_281, 1),
            blockhash_response(),
            fee_response(Some(5_000)),
        ])
        .await;
        let (client, mut state, _) = planning_client_with_keypair(&rpc_url, "0.000004999", keypair);
        set_canonical_balance(&mut state, 1);
        let insufficient = client
            .positions()
            .plan_unwrap_wsol_all(&state)
            .await
            .unwrap_err();
        assert!(insufficient.to_string().contains("unwrap-all fee"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn sol_planners_reject_unsupported_sponsorship_and_invalid_redeem_outcomes() {
        let (client, state, _) = planning_client("http://127.0.0.1:1", "1.000000000");

        let sponsored = client
            .positions()
            .plan_native_sol_withdrawal(Pubkey::new_unique(), 1, &state, true)
            .await
            .unwrap_err();
        assert!(sponsored
            .to_string()
            .contains("sponsored SOL action planning is not supported"));

        let invalid_outcome = client
            .positions()
            .plan_sol_redeem(Pubkey::new_unique(), 1, 2, 2, &state, false)
            .await
            .unwrap_err();
        assert!(invalid_outcome.to_string().contains("outcome index"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn sol_planners_reject_a_mismatched_signing_wallet_before_rpc() {
        let (client, state, _) = planning_client("http://127.0.0.1:1", "1.000000000");
        client
            .set_signing_strategy(crate::shared::signing::SigningStrategy::Native(Arc::new(
                Keypair::new(),
            )))
            .await;

        let error = client
            .positions()
            .plan_native_sol_withdrawal(Pubkey::new_unique(), 1, &state, false)
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("signing strategy does not control authenticated wallet"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn sol_planners_reject_an_occupied_invalid_canonical_account() {
        let keypair = Keypair::new();
        let wallet = keypair.pubkey();
        let (rpc_url, _) = spawn_rpc_server(vec![canonical_account_response(
            wallet,
            solana_sdk_ids::system_program::id(),
        )])
        .await;
        let (client, mut state, _) = planning_client_with_keypair(&rpc_url, "1.000000000", keypair);
        let mint = PubkeyStr::from(WRAPPED_SOL_MINT_ADDRESS);
        state.balances.insert(
            mint.clone(),
            DepositTokenBalance {
                mint,
                idle: Decimal::ONE,
                symbol: "WSOL".into(),
                name: "Wrapped SOL".into(),
                icon_url_low: None,
                icon_url_medium: None,
                icon_url_high: None,
            },
        );

        let error = client
            .positions()
            .plan_sol_split(&market(), 1, &state, false)
            .await
            .unwrap_err();
        assert!(error.to_string().contains("legacy Token Program"));
    }

    #[cfg(feature = "native")]
    #[tokio::test]
    async fn prepared_submission_rejects_a_mismatched_signing_wallet_before_rpc() {
        let (client, _, _) = planning_client("http://127.0.0.1:1", "1.000000000");
        let mut transaction = Transaction::new_with_payer(&[], Some(&Pubkey::new_unique()));
        transaction.message.recent_blockhash = solana_hash::Hash::new_unique();

        let error = client
            .sign_and_submit_prepared_tx_confirmed_with_slot(transaction)
            .await
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("does not control prepared transaction fee payer"));
    }
}
