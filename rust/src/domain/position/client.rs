//! Positions sub-client — portfolio & position queries, and on-chain position operations.

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
        let mut data = vec![0_u8; super::TOKEN_ACCOUNT_SPACE];
        data[..32].copy_from_slice(spl_token_interface::native_mint::id().as_ref());
        data[32..64].copy_from_slice(wallet.as_ref());
        data[108] = 1;
        data[109..113].copy_from_slice(&1_u32.to_le_bytes());
        data[113..121].copy_from_slice(&2_039_280_u64.to_le_bytes());
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data);
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "context": {"slot": 1},
                "value": {
                    "data": [encoded, "base64"],
                    "executable": false,
                    "lamports": 2_039_280,
                    "owner": program_owner.to_string(),
                    "rentEpoch": 0,
                    "space": super::TOKEN_ACCOUNT_SPACE,
                }
            }
        })
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
