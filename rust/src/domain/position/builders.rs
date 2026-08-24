//! Fluent builders for deposit, withdraw, and merge operations.
//!
//! Created via `client.positions().deposit().await`, `client.positions().withdraw().await`,
//! and `client.positions().merge()`.

use crate::client::LightconeClient;
use crate::domain::market::Market;
use crate::domain::position::state::{SolActionCosts, SolBalanceAvailability};
use crate::error::SdkError;
use crate::program::instructions;
use crate::program::types::{
    BuildDepositParams, BuildMergeParams, DepositToGlobalAltContext, DepositToGlobalParams,
    ExtendPositionTokensParams, GlobalToMarketDepositParams, InitPositionTokensParams,
    RedeemWinningsParams, WithdrawConditionalFromPositionParams, WithdrawFromGlobalParams,
};
use crate::shared::DepositSource;
use sha2::{Digest, Sha256};
use solana_hash::Hash;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;
use std::str::FromStr;

use super::state::WRAPPED_SOL_MINT_ADDRESS;

/// Byte allocation for accounts owned by Solana's legacy SPL Token Program.
///
/// “Tokenkeg” names that legacy program (`TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`).
/// Canonical native-mint WSOL accounts intentionally use it, not Token-2022,
/// because Solana's canonical native mint and established ATA convention are
/// pinned to the legacy program across the protocol and all three SDKs.
pub(crate) const TOKEN_ACCOUNT_SPACE: usize = 165;

/// SOL-aware operation represented by an SDK action plan.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SolActionKind {
    /// Mint a complete conditional-token set, wrapping only a WSOL shortfall.
    Split,
    /// Burn a complete set and retain returned collateral as canonical WSOL.
    Merge,
    /// Redeem winning tokens and retain returned collateral as canonical WSOL.
    Redeem,
    /// Deliver exact native lamports, converting canonical WSOL only if needed.
    NativeWithdraw,
    /// Represent an exact native-lamport wrap into the canonical WSOL account.
    #[cfg(feature = "native-auth")]
    Wrap,
    /// Represent closure of the complete canonical WSOL account to the Trading Wallet.
    #[cfg(feature = "native-auth")]
    UnwrapAll,
}

/// Expected change to the separately authoritative SOL components.
///
/// Values include the estimated transaction fee and net rent movement so Web
/// can freeze one post-confirmation projection without merging component state.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct SolComponentDelta {
    /// Expected system-account change in lamports, including unsponsored costs.
    pub native_lamports: i128,
    /// Expected canonical legacy-token WSOL ATA change in lamports.
    pub canonical_wsol_lamports: i128,
}

/// Unsigned transaction plus the exact preflight facts used to authorize it.
#[derive(Debug, Clone, PartialEq)]
pub struct SolActionPlan {
    /// Operation whose balance semantics produced this plan.
    pub kind: SolActionKind,
    /// Unsigned, fee-prepared transaction whose exact message must be preserved.
    pub transaction: Transaction,
    /// Live fee/rent observations and sponsorship capability used at preflight.
    pub costs: SolActionCosts,
    /// Authoritative component totals after reserving action-specific native SOL.
    pub availability: SolBalanceAvailability,
    /// Projection kept component-wise so callers do not erase state authority.
    pub expected_delta: SolComponentDelta,
}

/// Derive the canonical native mint and the wallet's persistent Tokenkeg ATA.
///
/// Tokenkeg is Solana's legacy SPL Token Program. The program ID is part of ATA
/// derivation, so this helper pins canonical WSOL to Tokenkeg rather than
/// Token-2022; changing it would address a different account and split protocol
/// authority across incompatible token programs.
pub(crate) fn wrapped_sol_accounts(wallet: &Pubkey) -> Result<(Pubkey, Pubkey), SdkError> {
    let mint = Pubkey::from_str(WRAPPED_SOL_MINT_ADDRESS)
        .map_err(|error| SdkError::Validation(format!("invalid wrapped SOL mint: {error}")))?;
    let token_program = spl_token_interface::id();
    let account = spl_associated_token_account_interface::address::get_associated_token_address_with_program_id(
        wallet,
        &mint,
        &token_program,
    );
    Ok((mint, account))
}

/// Build idempotent creation of the persistent legacy-token WSOL ATA.
fn create_idempotent_canonical_wsol_account(wallet: &Pubkey, mint: &Pubkey) -> Instruction {
    spl_associated_token_account_interface::instruction::create_associated_token_account_idempotent(
        wallet,
        wallet,
        mint,
        &spl_token_interface::id(),
    )
}

/// Return strict ATA creation for a canonical account observed missing during planning.
///
/// If the account appears before execution, this instruction fails instead of
/// accepting account state that was absent from the plan.
#[cfg(feature = "native-auth")]
fn create_new_canonical_wsol_account(wallet: &Pubkey, mint: &Pubkey) -> Instruction {
    spl_associated_token_account_interface::instruction::create_associated_token_account(
        wallet,
        wallet,
        mint,
        &spl_token_interface::id(),
    )
}

/// Return an unsigned transaction for an exact native-to-canonical WSOL wrap.
///
/// When planning observed a missing canonical Tokenkeg ATA, the first instruction
/// is strict ATA creation. A concurrently created ATA therefore makes execution
/// fail instead of using account state that was absent from the plan. The next
/// instructions transfer `amount_lamports` from the Trading Wallet and run
/// `SyncNative`. The planner later attaches the blockhash used for fee estimation.
#[cfg(feature = "native-auth")]
pub(crate) fn build_wrap_sol_transaction(
    wallet: Pubkey,
    amount_lamports: u64,
    create_canonical_account: bool,
) -> Result<Transaction, SdkError> {
    let token_program = spl_token_interface::id();
    let (mint, account) = wrapped_sol_accounts(&wallet)?;
    let mut instructions = Vec::with_capacity(if create_canonical_account { 3 } else { 2 });
    if create_canonical_account {
        instructions.push(create_new_canonical_wsol_account(&wallet, &mint));
    }
    instructions.push(solana_system_interface::instruction::transfer(
        &wallet,
        &account,
        amount_lamports,
    ));
    instructions.push(
        spl_token_interface::instruction::sync_native(&token_program, &account)
            .map_err(|error| SdkError::Other(format!("failed to build SyncNative: {error}")))?,
    );
    Ok(Transaction::new_with_payer(&instructions, Some(&wallet)))
}

/// Return an unsigned transaction containing one canonical `CloseAccount` instruction.
///
/// If submitted successfully, `CloseAccount` transfers every account lamport,
/// including rent and donated excess, to the same Trading Wallet that authorizes
/// the close. Ordinary split, merge, redeem, and native-withdrawal builders never
/// call this builder.
#[cfg(feature = "native-auth")]
pub(crate) fn build_unwrap_wsol_all_transaction(wallet: Pubkey) -> Result<Transaction, SdkError> {
    let token_program = spl_token_interface::id();
    let (_, account) = wrapped_sol_accounts(&wallet)?;
    let close = spl_token_interface::instruction::close_account(
        &token_program,
        &account,
        &wallet,
        &wallet,
        &[],
    )
    .map_err(|error| SdkError::Other(format!("failed to close canonical WSOL account: {error}")))?;
    Ok(Transaction::new_with_payer(&[close], Some(&wallet)))
}

/// Build one atomic wrap-shortfall and market split transaction.
pub(crate) fn build_sol_split_transaction(
    program_id: &Pubkey,
    wallet: Pubkey,
    market: &Market,
    amount: u64,
    shortfall: u64,
    create_canonical_account: bool,
) -> Result<Transaction, SdkError> {
    let token_program = spl_token_interface::id();
    let (mint, account) = wrapped_sol_accounts(&wallet)?;
    let mut instructions = Vec::with_capacity(4);
    if create_canonical_account {
        instructions.push(create_idempotent_canonical_wsol_account(&wallet, &mint));
    }
    if shortfall > 0 {
        instructions.push(solana_system_interface::instruction::transfer(
            &wallet, &account, shortfall,
        ));
        instructions.push(
            spl_token_interface::instruction::sync_native(&token_program, &account)
                .map_err(|error| SdkError::Other(format!("failed to build SyncNative: {error}")))?,
        );
    }
    instructions.push(instructions::build_deposit_ix(
        &BuildDepositParams {
            user: wallet,
            market: market.pubkey.to_pubkey().map_err(SdkError::Validation)?,
            deposit_mint: mint,
            amount,
        },
        market.num_outcomes,
        program_id,
    ));
    Ok(Transaction::new_with_payer(&instructions, Some(&wallet)))
}

/// Build a merge that creates the canonical ATA when needed and never closes it.
pub(crate) fn build_sol_merge_transaction(
    program_id: &Pubkey,
    wallet: Pubkey,
    market: &Market,
    amount: u64,
    create_canonical_account: bool,
) -> Result<Transaction, SdkError> {
    let (mint, _) = wrapped_sol_accounts(&wallet)?;
    let mut instructions = Vec::with_capacity(2);
    if create_canonical_account {
        instructions.push(create_idempotent_canonical_wsol_account(&wallet, &mint));
    }
    instructions.push(instructions::build_merge_ix(
        &BuildMergeParams {
            user: wallet,
            market: market.pubkey.to_pubkey().map_err(SdkError::Validation)?,
            deposit_mint: mint,
            amount,
        },
        market.num_outcomes,
        program_id,
    ));
    Ok(Transaction::new_with_payer(&instructions, Some(&wallet)))
}

/// Build a winnings redemption that leaves resulting WSOL in the canonical ATA.
pub(crate) fn build_sol_redeem_transaction(
    program_id: &Pubkey,
    wallet: Pubkey,
    market: Pubkey,
    amount: u64,
    outcome_index: u8,
    create_canonical_account: bool,
) -> Result<Transaction, SdkError> {
    let (mint, _) = wrapped_sol_accounts(&wallet)?;
    let mut instructions = Vec::with_capacity(2);
    if create_canonical_account {
        instructions.push(create_idempotent_canonical_wsol_account(&wallet, &mint));
    }
    instructions.push(instructions::build_redeem_winnings_ix(
        &RedeemWinningsParams {
            user: wallet,
            market,
            deposit_mint: mint,
            amount,
        },
        outcome_index,
        program_id,
    ));
    Ok(Transaction::new_with_payer(&instructions, Some(&wallet)))
}

/// Build the direct path when native lamports already cover amount and reserve.
pub(crate) fn build_direct_native_withdraw_transaction(
    wallet: Pubkey,
    recipient: Pubkey,
    amount: u64,
) -> Transaction {
    Transaction::new_with_payer(
        &[solana_system_interface::instruction::transfer(
            &wallet, &recipient, amount,
        )],
        Some(&wallet),
    )
}

/// Return a byte-exact, domain-separated temporary-account seed.
///
/// The SHA-256 preimage is the ASCII domain `lightcone:wsol-withdraw:v1`, one
/// zero byte, raw 32-byte blockhash, wallet, and recipient keys, the amount as
/// eight-byte unsigned big-endian lamports, then the one-byte attempt. This
/// exact order is shared by all three SDKs. The first 16 digest bytes become 32
/// lowercase hexadecimal ASCII characters to satisfy Solana's seed limit.
pub fn native_withdraw_seed(
    recent_blockhash: &Hash,
    wallet: &Pubkey,
    recipient: &Pubkey,
    amount_lamports: u64,
    attempt: u8,
) -> String {
    let mut preimage = Vec::with_capacity(24 + 1 + 32 + 32 + 32 + 8 + 1);
    preimage.extend_from_slice(b"lightcone:wsol-withdraw:v1");
    preimage.push(0);
    preimage.extend_from_slice(recent_blockhash.as_ref());
    preimage.extend_from_slice(wallet.as_ref());
    preimage.extend_from_slice(recipient.as_ref());
    preimage.extend_from_slice(&amount_lamports.to_be_bytes());
    preimage.push(attempt);
    let digest = Sha256::digest(preimage);
    hex::encode(&digest[..16])
}

/// Derive the legacy-token temporary WSOL account for a validated seed.
pub(crate) fn temporary_wsol_account(wallet: &Pubkey, seed: &str) -> Result<Pubkey, SdkError> {
    Pubkey::create_with_seed(wallet, seed, &spl_token_interface::id())
        .map_err(|error| SdkError::Validation(format!("invalid temporary WSOL seed: {error}")))
}

/// Build the only supported WSOL-to-native conversion path.
///
/// The temporary Tokenkeg account is created, initialized, funded from the
/// canonical ATA, and closed back to the Trading Wallet before the exact native
/// recipient transfer. All five instructions execute in one Solana transaction,
/// so any instruction failure rolls back the temporary account and both transfers.
/// The canonical account is never closed.
pub(crate) fn build_temporary_native_withdraw_transaction(
    wallet: Pubkey,
    recipient: Pubkey,
    amount: u64,
    canonical_transfer: u64,
    temporary_rent: u64,
    seed: &str,
    temporary_account: Pubkey,
) -> Result<Transaction, SdkError> {
    let token_program = spl_token_interface::id();
    let (mint, canonical_account) = wrapped_sol_accounts(&wallet)?;
    let create = solana_system_interface::instruction::create_account_with_seed(
        &wallet,
        &temporary_account,
        &wallet,
        seed,
        temporary_rent,
        TOKEN_ACCOUNT_SPACE as u64,
        &token_program,
    );
    let initialize = spl_token_interface::instruction::initialize_account3(
        &token_program,
        &temporary_account,
        &mint,
        &wallet,
    )
    .map_err(|error| {
        SdkError::Other(format!(
            "failed to initialize temporary WSOL account: {error}"
        ))
    })?;
    let transfer_wrapped = spl_token_interface::instruction::transfer(
        &token_program,
        &canonical_account,
        &temporary_account,
        &wallet,
        &[],
        canonical_transfer,
    )
    .map_err(|error| SdkError::Other(format!("failed to transfer canonical WSOL: {error}")))?;
    let close = spl_token_interface::instruction::close_account(
        &token_program,
        &temporary_account,
        &wallet,
        &wallet,
        &[],
    )
    .map_err(|error| SdkError::Other(format!("failed to close temporary WSOL account: {error}")))?;
    let transfer_native =
        solana_system_interface::instruction::transfer(&wallet, &recipient, amount);
    Ok(Transaction::new_with_payer(
        &[create, initialize, transfer_wrapped, close, transfer_native],
        Some(&wallet),
    ))
}

// ─── DepositBuilder ─────────────────────────────────────────────────────────

/// Fluent builder for deposit operations.
///
/// Created via `client.positions().deposit().await` — direct construction is not exposed.
/// Pre-seeded with the client's deposit source setting.
///
/// Dispatches based on deposit source:
/// - **Global**: `deposit_to_global` — wallet → global pool
/// - **Market**: `deposit` (mint complete set) — wallet → market, mints conditional tokens
///
/// # Example (global deposit)
///
/// ```rust,ignore
/// let ix = client.positions().deposit().await
///     .user(keypair.pubkey())
///     .mint(deposit_mint)
///     .amount(1_000_000)
///     .build_ix()
///     .await?;
/// ```
///
/// # Example (market deposit)
///
/// ```rust,ignore
/// let ix = client.positions().deposit().await
///     .user(keypair.pubkey())
///     .mint(deposit_mint)
///     .amount(1_000_000)
///     .with_market_deposit_source(&market)
///     .build_ix()
///     .await?;
/// ```
pub struct DepositBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    mint: Option<Pubkey>,
    amount: Option<u64>,
    market: Option<&'a Market>,
    deposit_source: Option<DepositSource>,
}

impl<'a> DepositBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient, deposit_source: DepositSource) -> Self {
        Self {
            client,
            user: None,
            mint: None,
            amount: None,
            market: None,
            deposit_source: Some(deposit_source),
        }
    }

    /// Set the depositor's public key.
    pub fn user(mut self, user: Pubkey) -> Self {
        self.user = Some(user);
        self
    }

    /// Set the deposit token mint.
    pub fn mint(mut self, mint: Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the deposit amount.
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the market reference (required when deposit source is `Market`).
    ///
    /// Use this when the client is already configured with `DepositSource::Market`.
    /// Otherwise, prefer `with_market_deposit_source()` to set both at once.
    pub fn market(mut self, market: &'a Market) -> Self {
        self.market = Some(market);
        self
    }

    /// Override the deposit source for this call.
    pub fn deposit_source(mut self, source: DepositSource) -> Self {
        self.deposit_source = Some(source);
        self
    }

    /// Set deposit source to `Market` and provide the required market reference.
    pub fn with_market_deposit_source(mut self, market: &'a Market) -> Self {
        self.deposit_source = Some(DepositSource::Market);
        self.market = Some(market);
        self
    }

    /// Set deposit source to `Global`.
    pub fn with_global_deposit_source(mut self) -> Self {
        self.deposit_source = Some(DepositSource::Global);
        self
    }

    /// Build a deposit instruction.
    pub async fn build_ix(self) -> Result<Instruction, SdkError> {
        let user = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let mint = self
            .mint
            .ok_or_else(|| SdkError::Validation("mint is required".into()))?;
        let amount = self
            .amount
            .ok_or_else(|| SdkError::Validation("amount is required".into()))?;

        let source = self
            .client
            .resolve_deposit_source(self.deposit_source)
            .await;

        let program_id = &self.client.program_id;

        match source {
            DepositSource::Global => Ok(instructions::build_deposit_to_global_ix(
                &DepositToGlobalParams { user, mint, amount },
                program_id,
            )),
            DepositSource::Market => {
                let market = self.market.ok_or(SdkError::MissingMarketContext(
                    "market is required for Market deposit source",
                ))?;
                let market_pubkey = market
                    .pubkey
                    .to_pubkey()
                    .map_err(|error| SdkError::Validation(error))?;
                let num_outcomes = market.num_outcomes;
                Ok(instructions::build_deposit_ix(
                    &BuildDepositParams {
                        user,
                        market: market_pubkey,
                        deposit_mint: mint,
                        amount,
                    },
                    num_outcomes,
                    program_id,
                ))
            }
        }
    }

    /// Build a deposit transaction.
    pub async fn build_tx(self) -> Result<Transaction, SdkError> {
        let payer = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let instruction = self.build_ix().await?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&payer)))
    }

    /// Build, sign, and submit the deposit transaction.
    pub async fn sign_and_submit(self) -> Result<String, SdkError> {
        let client = self.client;
        let transaction = self.build_tx().await?;
        client.sign_and_submit_tx(transaction).await
    }
}

// ─── MergeBuilder ────────────────────────────────────────────────────────

/// Fluent builder for merge operations.
///
/// Created via `client.positions().merge()` — direct construction is not exposed.
///
/// Burns a complete set of conditional tokens (one of each outcome) from a market
/// position and releases the underlying collateral back to the user's wallet.
///
/// # Example
///
/// ```rust,ignore
/// let ix = client.positions().merge()
///     .user(keypair.pubkey())
///     .market(&market)
///     .mint(deposit_mint)
///     .amount(1_000_000)
///     .build_ix()?;
/// ```
pub struct MergeBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    mint: Option<Pubkey>,
    amount: Option<u64>,
    market: Option<&'a Market>,
}

impl<'a> MergeBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient) -> Self {
        Self {
            client,
            user: None,
            mint: None,
            amount: None,
            market: None,
        }
    }

    /// Set the user's public key.
    pub fn user(mut self, user: Pubkey) -> Self {
        self.user = Some(user);
        self
    }

    /// Set the token mint.
    pub fn mint(mut self, mint: Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the merge amount.
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the market reference (required).
    pub fn market(mut self, market: &'a Market) -> Self {
        self.market = Some(market);
        self
    }

    /// Build a merge instruction.
    pub fn build_ix(self) -> Result<Instruction, SdkError> {
        let user = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let mint = self
            .mint
            .ok_or_else(|| SdkError::Validation("mint is required".into()))?;
        let amount = self
            .amount
            .ok_or_else(|| SdkError::Validation("amount is required".into()))?;
        let market = self.market.ok_or(SdkError::MissingMarketContext(
            "market is required for merge",
        ))?;
        let market_pubkey = market
            .pubkey
            .to_pubkey()
            .map_err(|error| SdkError::Validation(error))?;
        let num_outcomes = market.num_outcomes;
        let program_id = &self.client.program_id;

        Ok(instructions::build_merge_ix(
            &BuildMergeParams {
                user,
                market: market_pubkey,
                deposit_mint: mint,
                amount,
            },
            num_outcomes,
            program_id,
        ))
    }

    /// Build a merge transaction.
    pub fn build_tx(self) -> Result<Transaction, SdkError> {
        let payer = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let instruction = self.build_ix()?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&payer)))
    }

    /// Build, sign, and submit the merge transaction.
    pub async fn sign_and_submit(self) -> Result<String, SdkError> {
        let client = self.client;
        let transaction = self.build_tx()?;
        client.sign_and_submit_tx(transaction).await
    }
}

// ─── WithdrawBuilder ─────────────────────────────────────────────────────────

/// Fluent builder for withdraw operations.
///
/// Created via `client.positions().withdraw().await` — direct construction is not exposed.
/// Pre-seeded with the client's deposit source setting.
///
/// Dispatches based on deposit source:
/// - **Global**: `withdraw_from_global` — global pool → wallet
/// - **Market**: `withdraw_conditional_from_position` — conditional-token ATA → user's wallet
///
/// # Example (global withdraw)
///
/// ```rust,ignore
/// let ix = client.positions().withdraw().await
///     .user(keypair.pubkey())
///     .mint(deposit_mint)
///     .amount(1_000_000)
///     .build_ix()
///     .await?;
/// ```
///
/// # Example (market withdraw)
///
/// ```rust,ignore
/// let ix = client.positions().withdraw().await
///     .user(keypair.pubkey())
///     .mint(deposit_mint)
///     .amount(1_000_000)
///     .with_market_deposit_source(&market)
///     .outcome_index(0)
///     .build_ix()
///     .await?;
/// ```
pub struct WithdrawBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    mint: Option<Pubkey>,
    amount: Option<u64>,
    deposit_source: Option<DepositSource>,
    market: Option<&'a Market>,
    outcome_index: Option<u8>,
}

impl<'a> WithdrawBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient, deposit_source: DepositSource) -> Self {
        Self {
            client,
            user: None,
            mint: None,
            amount: None,
            deposit_source: Some(deposit_source),
            market: None,
            outcome_index: None,
        }
    }

    /// Set the user's public key.
    pub fn user(mut self, user: Pubkey) -> Self {
        self.user = Some(user);
        self
    }

    /// Set the token mint.
    ///
    /// In `Global` mode this is the deposit token mint to withdraw. In `Market`
    /// mode this is the market's registered deposit mint; the conditional mint
    /// is derived from this mint plus `outcome_index`.
    pub fn mint(mut self, mint: Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the registered deposit mint for a market withdrawal.
    ///
    /// This is an alias for `mint` that makes conditional withdrawals explicit.
    pub fn deposit_mint(self, deposit_mint: Pubkey) -> Self {
        self.mint(deposit_mint)
    }

    /// Set the withdrawal amount.
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Override the deposit source for this call.
    pub fn deposit_source(mut self, source: DepositSource) -> Self {
        self.deposit_source = Some(source);
        self
    }

    /// Set deposit source to `Global`.
    pub fn with_global_deposit_source(mut self) -> Self {
        self.deposit_source = Some(DepositSource::Global);
        self
    }

    /// Set deposit source to `Market` and provide the required market reference.
    pub fn with_market_deposit_source(mut self, market: &'a Market) -> Self {
        self.deposit_source = Some(DepositSource::Market);
        self.market = Some(market);
        self
    }

    /// Set the market reference (required when deposit source is `Market`).
    pub fn market(mut self, market: &'a Market) -> Self {
        self.market = Some(market);
        self
    }

    /// Set the outcome index (required when deposit source is `Market`).
    pub fn outcome_index(mut self, outcome_index: u8) -> Self {
        self.outcome_index = Some(outcome_index);
        self
    }

    /// Build a withdraw instruction.
    pub async fn build_ix(self) -> Result<Instruction, SdkError> {
        let user = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let mint = self
            .mint
            .ok_or_else(|| SdkError::Validation("mint is required".into()))?;
        let amount = self
            .amount
            .ok_or_else(|| SdkError::Validation("amount is required".into()))?;

        let source = self
            .client
            .resolve_deposit_source(self.deposit_source)
            .await;

        let program_id = &self.client.program_id;

        match source {
            DepositSource::Global => Ok(instructions::build_withdraw_from_global_ix(
                &WithdrawFromGlobalParams { user, mint, amount },
                program_id,
            )),
            DepositSource::Market => {
                let market = self.market.ok_or(SdkError::MissingMarketContext(
                    "market is required for Market withdrawal",
                ))?;
                let market_pubkey = market
                    .pubkey
                    .to_pubkey()
                    .map_err(|error| SdkError::Validation(error))?;
                let outcome_index = self.outcome_index.ok_or_else(|| {
                    SdkError::Validation("outcome_index is required for Market withdrawal".into())
                })?;
                crate::program::utils::validate_outcome_index(outcome_index, market.num_outcomes)?;
                Ok(instructions::build_withdraw_conditional_from_position_ix(
                    &WithdrawConditionalFromPositionParams {
                        user,
                        market: market_pubkey,
                        deposit_mint: mint,
                        amount,
                        outcome_index,
                    },
                    program_id,
                ))
            }
        }
    }

    /// Build a withdraw transaction.
    pub async fn build_tx(self) -> Result<Transaction, SdkError> {
        let payer = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let instruction = self.build_ix().await?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&payer)))
    }

    /// Build, sign, and submit the withdraw transaction.
    pub async fn sign_and_submit(self) -> Result<String, SdkError> {
        let client = self.client;
        let transaction = self.build_tx().await?;
        client.sign_and_submit_tx(transaction).await
    }
}

// ─── RedeemWinningsBuilder ─────────────────────────────────────────────────

/// Fluent builder for redeem winnings operations.
///
/// Created via `client.positions().redeem_winnings()` — direct construction is not exposed.
///
/// # Example
///
/// ```rust,ignore
/// let tx_signature = client.positions().redeem_winnings()
///     .user(keypair.pubkey())
///     .market(market_pubkey)
///     .mint(mint_pubkey)
///     .amount(1_000_000)
///     .outcome_index(0)
///     .sign_and_submit()
///     .await?;
/// ```
pub struct RedeemWinningsBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    market: Option<Pubkey>,
    mint: Option<Pubkey>,
    amount: Option<u64>,
    outcome_index: Option<u8>,
}

impl<'a> RedeemWinningsBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient) -> Self {
        Self {
            client,
            user: None,
            market: None,
            mint: None,
            amount: None,
            outcome_index: None,
        }
    }

    /// Set the user's public key.
    pub fn user(mut self, user: Pubkey) -> Self {
        self.user = Some(user);
        self
    }

    /// Set the market public key.
    pub fn market(mut self, market: Pubkey) -> Self {
        self.market = Some(market);
        self
    }

    /// Set the deposit token mint.
    pub fn mint(mut self, mint: Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the amount of winning tokens to redeem.
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the outcome index to redeem.
    pub fn outcome_index(mut self, outcome_index: u8) -> Self {
        self.outcome_index = Some(outcome_index);
        self
    }

    /// Deprecated alias for `outcome_index`.
    pub fn winning_outcome(self, winning_outcome: u8) -> Self {
        self.outcome_index(winning_outcome)
    }

    /// Build a redeem winnings instruction.
    pub fn build_ix(self) -> Result<Instruction, SdkError> {
        let user = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let market = self
            .market
            .ok_or_else(|| SdkError::Validation("market is required".into()))?;
        let mint = self
            .mint
            .ok_or_else(|| SdkError::Validation("mint is required".into()))?;
        let amount = self
            .amount
            .ok_or_else(|| SdkError::Validation("amount is required".into()))?;
        let outcome_index = self
            .outcome_index
            .ok_or_else(|| SdkError::Validation("outcome_index is required".into()))?;

        Ok(instructions::build_redeem_winnings_ix(
            &RedeemWinningsParams {
                user,
                market,
                deposit_mint: mint,
                amount,
            },
            outcome_index,
            &self.client.program_id,
        ))
    }

    /// Build a redeem winnings transaction.
    pub fn build_tx(self) -> Result<Transaction, SdkError> {
        let payer = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let instruction = self.build_ix()?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&payer)))
    }

    /// Build, sign, and submit the redeem winnings transaction.
    pub async fn sign_and_submit(self) -> Result<String, SdkError> {
        let client = self.client;
        let transaction = self.build_tx()?;
        client.sign_and_submit_tx(transaction).await
    }
}

// ─── WithdrawFromPositionBuilder ───────────────────────────────────────────

/// Fluent builder for conditional-token withdraw-from-position operations.
///
/// Created via `client.positions().withdraw_conditional_from_position()` — direct construction is not exposed.
///
/// # Example
///
/// ```rust,ignore
/// let tx_signature = client.positions().withdraw_conditional_from_position()
///     .user(keypair.pubkey())
///     .market(market_pubkey)
///     .deposit_mint(deposit_mint)
///     .amount(1_000_000)
///     .outcome_index(0)
///     .num_outcomes(market.num_outcomes)
///     .sign_and_submit()
///     .await?;
/// ```
pub struct WithdrawFromPositionBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    market: Option<Pubkey>,
    deposit_mint: Option<Pubkey>,
    amount: Option<u64>,
    outcome_index: Option<u8>,
    num_outcomes: Option<u8>,
}

impl<'a> WithdrawFromPositionBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient) -> Self {
        Self {
            client,
            user: None,
            market: None,
            deposit_mint: None,
            amount: None,
            outcome_index: None,
            num_outcomes: None,
        }
    }

    /// Set the withdrawer's public key.
    pub fn user(mut self, user: Pubkey) -> Self {
        self.user = Some(user);
        self
    }

    /// Set the market public key.
    pub fn market(mut self, market: Pubkey) -> Self {
        self.market = Some(market);
        self
    }

    /// Set the registered deposit mint for the market.
    pub fn deposit_mint(mut self, deposit_mint: Pubkey) -> Self {
        self.deposit_mint = Some(deposit_mint);
        self
    }

    /// Set the registered deposit mint for the market.
    ///
    /// This alias preserves the SDK's existing fluent builder style.
    pub fn mint(self, deposit_mint: Pubkey) -> Self {
        self.deposit_mint(deposit_mint)
    }

    /// Set the withdrawal amount.
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the outcome index.
    pub fn outcome_index(mut self, outcome_index: u8) -> Self {
        self.outcome_index = Some(outcome_index);
        self
    }

    /// Set the market's authoritative outcome count.
    pub fn num_outcomes(mut self, num_outcomes: u8) -> Self {
        self.num_outcomes = Some(num_outcomes);
        self
    }

    /// Build a withdraw-from-position instruction.
    pub fn build_ix(self) -> Result<Instruction, SdkError> {
        let user = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let market = self
            .market
            .ok_or_else(|| SdkError::Validation("market is required".into()))?;
        let deposit_mint = self
            .deposit_mint
            .ok_or_else(|| SdkError::Validation("deposit_mint is required".into()))?;
        let amount = self
            .amount
            .ok_or_else(|| SdkError::Validation("amount is required".into()))?;
        let outcome_index = self
            .outcome_index
            .ok_or_else(|| SdkError::Validation("outcome_index is required".into()))?;
        let num_outcomes = self
            .num_outcomes
            .ok_or_else(|| SdkError::Validation("num_outcomes is required".into()))?;
        crate::program::utils::validate_outcome_count(num_outcomes)?;
        crate::program::utils::validate_outcome_index(outcome_index, num_outcomes)?;

        Ok(instructions::build_withdraw_conditional_from_position_ix(
            &WithdrawConditionalFromPositionParams {
                user,
                market,
                deposit_mint,
                amount,
                outcome_index,
            },
            &self.client.program_id,
        ))
    }

    /// Build a withdraw-from-position transaction.
    pub fn build_tx(self) -> Result<Transaction, SdkError> {
        let payer = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let instruction = self.build_ix()?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&payer)))
    }

    /// Build, sign, and submit the withdraw-from-position transaction.
    pub async fn sign_and_submit(self) -> Result<String, SdkError> {
        let client = self.client;
        let transaction = self.build_tx()?;
        client.sign_and_submit_tx(transaction).await
    }
}

#[cfg(test)]
mod withdraw_from_position_tests {
    use super::*;

    fn client() -> LightconeClient {
        match LightconeClient::builder().build() {
            Ok(client) => client,
            Err(error) => panic!("failed to build test client: {error}"),
        }
    }

    fn builder(client: &LightconeClient) -> WithdrawFromPositionBuilder<'_> {
        client
            .positions()
            .withdraw_from_position()
            .user(Pubkey::new_unique())
            .market(Pubkey::new_unique())
            .deposit_mint(Pubkey::new_unique())
            .amount(1)
            .outcome_index(2)
    }

    #[test]
    fn requires_num_outcomes() {
        let client = client();
        let error = match builder(&client).build_ix() {
            Ok(_) => panic!("expected missing num_outcomes to fail"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("num_outcomes is required"));
    }

    #[test]
    fn validates_against_num_outcomes() {
        let client = client();
        let error = match builder(&client).num_outcomes(2).build_ix() {
            Ok(_) => panic!("expected invalid outcome index to fail"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("Invalid outcome index"));
    }
}

// ─── InitPositionTokensBuilder ─────────────────────────────────────────────

/// Fluent builder for init-position-tokens operations.
///
/// Created via `client.positions().init_position_tokens()` — direct construction is not exposed.
///
/// # Example
///
/// ```rust,ignore
/// let tx_signature = client.positions().init_position_tokens()
///     .payer(keypair.pubkey())
///     .user(user_pubkey)
///     .market(market_pubkey)
///     .deposit_mints(vec![mint_a, mint_b])
///     .recent_slot(slot)
///     .num_outcomes(2)
///     .sign_and_submit()
///     .await?;
/// ```
pub struct InitPositionTokensBuilder<'a> {
    client: &'a LightconeClient,
    payer: Option<Pubkey>,
    user: Option<Pubkey>,
    market: Option<Pubkey>,
    deposit_mints: Option<Vec<Pubkey>>,
    recent_slot: Option<u64>,
    num_outcomes: Option<u8>,
}

impl<'a> InitPositionTokensBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient) -> Self {
        Self {
            client,
            payer: None,
            user: None,
            market: None,
            deposit_mints: None,
            recent_slot: None,
            num_outcomes: None,
        }
    }

    /// Set the payer's public key (signer, does not need to be the user).
    pub fn payer(mut self, payer: Pubkey) -> Self {
        self.payer = Some(payer);
        self
    }

    /// Set the position owner's public key.
    pub fn user(mut self, user: Pubkey) -> Self {
        self.user = Some(user);
        self
    }

    /// Set the market public key.
    pub fn market(mut self, market: Pubkey) -> Self {
        self.market = Some(market);
        self
    }

    /// Set the deposit mints to initialize.
    pub fn deposit_mints(mut self, deposit_mints: Vec<Pubkey>) -> Self {
        self.deposit_mints = Some(deposit_mints);
        self
    }

    /// Set the recent slot for ALT address derivation.
    pub fn recent_slot(mut self, recent_slot: u64) -> Self {
        self.recent_slot = Some(recent_slot);
        self
    }

    /// Set the number of outcomes in the market.
    pub fn num_outcomes(mut self, num_outcomes: u8) -> Self {
        self.num_outcomes = Some(num_outcomes);
        self
    }

    /// Build an init-position-tokens instruction.
    pub fn build_ix(self) -> Result<Instruction, SdkError> {
        let payer = self
            .payer
            .ok_or_else(|| SdkError::Validation("payer is required".into()))?;
        let user = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let market = self
            .market
            .ok_or_else(|| SdkError::Validation("market is required".into()))?;
        let deposit_mints = self
            .deposit_mints
            .ok_or_else(|| SdkError::Validation("deposit_mints is required".into()))?;
        let recent_slot = self
            .recent_slot
            .ok_or_else(|| SdkError::Validation("recent_slot is required".into()))?;
        let num_outcomes = self
            .num_outcomes
            .ok_or_else(|| SdkError::Validation("num_outcomes is required".into()))?;

        Ok(instructions::build_init_position_tokens_ix(
            &InitPositionTokensParams {
                payer,
                user,
                market,
                deposit_mints,
                recent_slot,
            },
            num_outcomes,
            &self.client.program_id,
        ))
    }

    /// Build an init-position-tokens transaction.
    pub fn build_tx(self) -> Result<Transaction, SdkError> {
        let payer = self
            .payer
            .ok_or_else(|| SdkError::Validation("payer is required".into()))?;
        let instruction = self.build_ix()?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&payer)))
    }

    /// Build, sign, and submit the init-position-tokens transaction.
    pub async fn sign_and_submit(self) -> Result<String, SdkError> {
        let client = self.client;
        let transaction = self.build_tx()?;
        client.sign_and_submit_tx(transaction).await
    }
}

// ─── ExtendPositionTokensBuilder ───────────────────────────────────────────

/// Fluent builder for extend-position-tokens operations.
///
/// Created via `client.positions().extend_position_tokens()` — direct construction is not exposed.
///
/// # Example
///
/// ```rust,ignore
/// let tx_signature = client.positions().extend_position_tokens()
///     .operator(keypair.pubkey())
///     .user(user_pubkey)
///     .market(market_pubkey)
///     .lookup_table(alt_pubkey)
///     .deposit_mints(vec![mint_c, mint_d])
///     .num_outcomes(2)
///     .sign_and_submit()
///     .await?;
/// ```
pub struct ExtendPositionTokensBuilder<'a> {
    client: &'a LightconeClient,
    operator: Option<Pubkey>,
    user: Option<Pubkey>,
    market: Option<Pubkey>,
    lookup_table: Option<Pubkey>,
    deposit_mints: Option<Vec<Pubkey>>,
    num_outcomes: Option<u8>,
}

impl<'a> ExtendPositionTokensBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient) -> Self {
        Self {
            client,
            operator: None,
            user: None,
            market: None,
            lookup_table: None,
            deposit_mints: None,
            num_outcomes: None,
        }
    }

    /// Set the operator's public key (signer).
    pub fn operator(mut self, operator: Pubkey) -> Self {
        self.operator = Some(operator);
        self
    }

    /// Set the position owner's public key.
    pub fn user(mut self, user: Pubkey) -> Self {
        self.user = Some(user);
        self
    }

    /// Set the market public key.
    pub fn market(mut self, market: Pubkey) -> Self {
        self.market = Some(market);
        self
    }

    /// Set the existing ALT public key from init_position_tokens.
    pub fn lookup_table(mut self, lookup_table: Pubkey) -> Self {
        self.lookup_table = Some(lookup_table);
        self
    }

    /// Set the new deposit mints to add.
    pub fn deposit_mints(mut self, deposit_mints: Vec<Pubkey>) -> Self {
        self.deposit_mints = Some(deposit_mints);
        self
    }

    /// Set the number of outcomes in the market.
    pub fn num_outcomes(mut self, num_outcomes: u8) -> Self {
        self.num_outcomes = Some(num_outcomes);
        self
    }

    /// Build an extend-position-tokens instruction.
    pub fn build_ix(self) -> Result<Instruction, SdkError> {
        let operator = self
            .operator
            .ok_or_else(|| SdkError::Validation("operator is required".into()))?;
        let user = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let market = self
            .market
            .ok_or_else(|| SdkError::Validation("market is required".into()))?;
        let lookup_table = self
            .lookup_table
            .ok_or_else(|| SdkError::Validation("lookup_table is required".into()))?;
        let deposit_mints = self
            .deposit_mints
            .ok_or_else(|| SdkError::Validation("deposit_mints is required".into()))?;
        let num_outcomes = self
            .num_outcomes
            .ok_or_else(|| SdkError::Validation("num_outcomes is required".into()))?;

        Ok(instructions::build_extend_position_tokens_ix(
            &ExtendPositionTokensParams {
                operator,
                user,
                market,
                lookup_table,
                deposit_mints,
            },
            num_outcomes,
            &self.client.program_id,
        )?)
    }

    /// Build an extend-position-tokens transaction.
    pub fn build_tx(self) -> Result<Transaction, SdkError> {
        let operator = self
            .operator
            .ok_or_else(|| SdkError::Validation("operator is required".into()))?;
        let instruction = self.build_ix()?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&operator)))
    }

    /// Build, sign, and submit the extend-position-tokens transaction.
    pub async fn sign_and_submit(self) -> Result<String, SdkError> {
        let client = self.client;
        let transaction = self.build_tx()?;
        client.sign_and_submit_tx(transaction).await
    }
}

// ─── DepositToGlobalBuilder ────────────────────────────────────────────────

/// Fluent builder for deposit-to-global operations.
///
/// Created via `client.positions().deposit_to_global()` — direct construction is not exposed.
///
/// # Example
///
/// ```rust,ignore
/// let tx_signature = client.positions().deposit_to_global()
///     .user(keypair.pubkey())
///     .mint(mint_pubkey)
///     .amount(1_000_000)
///     .sign_and_submit()
///     .await?;
/// ```
pub struct DepositToGlobalBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    mint: Option<Pubkey>,
    amount: Option<u64>,
    alt_context: Option<DepositToGlobalAltContext>,
}

impl<'a> DepositToGlobalBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient) -> Self {
        Self {
            client,
            user: None,
            mint: None,
            amount: None,
            alt_context: None,
        }
    }

    /// Set the depositor's public key.
    pub fn user(mut self, user: Pubkey) -> Self {
        self.user = Some(user);
        self
    }

    /// Set the deposit token mint.
    pub fn mint(mut self, mint: Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the deposit amount.
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Create the user's deposit ALT while depositing.
    pub fn create_alt(mut self, recent_slot: u64) -> Self {
        self.alt_context = Some(DepositToGlobalAltContext::Create { recent_slot });
        self
    }

    /// Extend an existing user deposit ALT while depositing.
    pub fn extend_alt(mut self, lookup_table: Pubkey) -> Self {
        self.alt_context = Some(DepositToGlobalAltContext::Extend { lookup_table });
        self
    }

    /// Build a deposit-to-global instruction.
    pub fn build_ix(self) -> Result<Instruction, SdkError> {
        let user = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let mint = self
            .mint
            .ok_or_else(|| SdkError::Validation("mint is required".into()))?;
        let amount = self
            .amount
            .ok_or_else(|| SdkError::Validation("amount is required".into()))?;

        let params = DepositToGlobalParams { user, mint, amount };
        Ok(match self.alt_context {
            Some(alt_context) => instructions::build_deposit_to_global_ix_with_alt(
                &params,
                alt_context,
                &self.client.program_id,
            ),
            None => instructions::build_deposit_to_global_ix(&params, &self.client.program_id),
        })
    }

    /// Build a deposit-to-global transaction.
    pub fn build_tx(self) -> Result<Transaction, SdkError> {
        let payer = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let instruction = self.build_ix()?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&payer)))
    }

    /// Build, sign, and submit the deposit-to-global transaction.
    pub async fn sign_and_submit(self) -> Result<String, SdkError> {
        let client = self.client;
        let transaction = self.build_tx()?;
        client.sign_and_submit_tx(transaction).await
    }
}

// ─── WithdrawFromGlobalBuilder ─────────────────────────────────────────────

/// Fluent builder for withdraw-from-global operations.
///
/// Created via `client.positions().withdraw_from_global()` — direct construction is not exposed.
///
/// # Example
///
/// ```rust,ignore
/// let tx_signature = client.positions().withdraw_from_global()
///     .user(keypair.pubkey())
///     .mint(mint_pubkey)
///     .amount(1_000_000)
///     .sign_and_submit()
///     .await?;
/// ```
pub struct WithdrawFromGlobalBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    mint: Option<Pubkey>,
    amount: Option<u64>,
}

impl<'a> WithdrawFromGlobalBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient) -> Self {
        Self {
            client,
            user: None,
            mint: None,
            amount: None,
        }
    }

    /// Set the withdrawer's public key.
    pub fn user(mut self, user: Pubkey) -> Self {
        self.user = Some(user);
        self
    }

    /// Set the token mint to withdraw.
    pub fn mint(mut self, mint: Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the withdrawal amount.
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Build a withdraw-from-global instruction.
    pub fn build_ix(self) -> Result<Instruction, SdkError> {
        let user = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let mint = self
            .mint
            .ok_or_else(|| SdkError::Validation("mint is required".into()))?;
        let amount = self
            .amount
            .ok_or_else(|| SdkError::Validation("amount is required".into()))?;

        Ok(instructions::build_withdraw_from_global_ix(
            &WithdrawFromGlobalParams { user, mint, amount },
            &self.client.program_id,
        ))
    }

    /// Build a withdraw-from-global transaction.
    pub fn build_tx(self) -> Result<Transaction, SdkError> {
        let payer = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let instruction = self.build_ix()?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&payer)))
    }

    /// Build, sign, and submit the withdraw-from-global transaction.
    pub async fn sign_and_submit(self) -> Result<String, SdkError> {
        let client = self.client;
        let transaction = self.build_tx()?;
        client.sign_and_submit_tx(transaction).await
    }
}

// ─── GlobalToMarketDepositBuilder ──────────────────────────────────────────

/// Fluent builder for global-to-market deposit operations.
///
/// Created via `client.positions().global_to_market_deposit()` — direct construction is not exposed.
///
/// # Example
///
/// ```rust,ignore
/// let tx_signature = client.positions().global_to_market_deposit()
///     .user(keypair.pubkey())
///     .market(market_pubkey)
///     .mint(mint_pubkey)
///     .amount(1_000_000)
///     .num_outcomes(2)
///     .sign_and_submit()
///     .await?;
/// ```
pub struct GlobalToMarketDepositBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    market: Option<Pubkey>,
    mint: Option<Pubkey>,
    amount: Option<u64>,
    num_outcomes: Option<u8>,
}

impl<'a> GlobalToMarketDepositBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient) -> Self {
        Self {
            client,
            user: None,
            market: None,
            mint: None,
            amount: None,
            num_outcomes: None,
        }
    }

    /// Set the depositor's public key.
    pub fn user(mut self, user: Pubkey) -> Self {
        self.user = Some(user);
        self
    }

    /// Set the market public key.
    pub fn market(mut self, market: Pubkey) -> Self {
        self.market = Some(market);
        self
    }

    /// Set the deposit token mint.
    pub fn mint(mut self, mint: Pubkey) -> Self {
        self.mint = Some(mint);
        self
    }

    /// Set the deposit amount.
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the number of outcomes in the market.
    pub fn num_outcomes(mut self, num_outcomes: u8) -> Self {
        self.num_outcomes = Some(num_outcomes);
        self
    }

    /// Build a global-to-market deposit instruction.
    pub fn build_ix(self) -> Result<Instruction, SdkError> {
        let user = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let market = self
            .market
            .ok_or_else(|| SdkError::Validation("market is required".into()))?;
        let mint = self
            .mint
            .ok_or_else(|| SdkError::Validation("mint is required".into()))?;
        let amount = self
            .amount
            .ok_or_else(|| SdkError::Validation("amount is required".into()))?;
        let num_outcomes = self
            .num_outcomes
            .ok_or_else(|| SdkError::Validation("num_outcomes is required".into()))?;

        Ok(instructions::build_global_to_market_deposit_ix(
            &GlobalToMarketDepositParams {
                user,
                market,
                deposit_mint: mint,
                amount,
            },
            num_outcomes,
            &self.client.program_id,
        ))
    }

    /// Build a global-to-market deposit transaction.
    pub fn build_tx(self) -> Result<Transaction, SdkError> {
        let payer = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let instruction = self.build_ix()?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&payer)))
    }

    /// Build, sign, and submit the global-to-market deposit transaction.
    pub async fn sign_and_submit(self) -> Result<String, SdkError> {
        let client = self.client;
        let transaction = self.build_tx()?;
        client.sign_and_submit_tx(transaction).await
    }
}

#[cfg(test)]
mod sol_action_tests {
    //! Cross-SDK canonical and temporary WSOL instruction invariants.

    use super::*;
    use solana_system_interface::instruction::SystemInstruction;
    use spl_token_interface::instruction::TokenInstruction;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn market() -> Market {
        use crate::{domain::market::Status, shared::PubkeyStr};

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
            created_at: chrono::Utc::now(),
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
            token_metadata: std::collections::HashMap::new(),
        }
    }

    fn account_key(transaction: &Transaction, instruction: usize, account: usize) -> Pubkey {
        transaction.message.account_keys
            [transaction.message.instructions[instruction].accounts[account] as usize]
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn exact_wrap_uses_strict_creation_before_transfer_and_sync() -> TestResult {
        let wallet = Pubkey::new_unique();
        let (mint, canonical) = wrapped_sol_accounts(&wallet)?;
        let create = create_idempotent_canonical_wsol_account(&wallet, &mint);
        let expected_create_accounts = [
            (wallet, true, true),
            (canonical, false, true),
            (wallet, false, false),
            (mint, false, false),
            (solana_sdk_ids::system_program::id(), false, false),
            (spl_token_interface::id(), false, false),
        ];
        assert_eq!(
            create.program_id,
            spl_associated_token_account_interface::program::id()
        );
        assert_eq!(create.data, [1]); // Ordinary builders retain idempotent creation.
        assert_eq!(create.accounts.len(), expected_create_accounts.len());
        for (meta, (pubkey, is_signer, is_writable)) in
            create.accounts.iter().zip(expected_create_accounts)
        {
            assert_eq!(meta.pubkey, pubkey);
            assert_eq!(
                meta.is_signer, is_signer,
                "unexpected signer flag for {pubkey}"
            );
            assert_eq!(
                meta.is_writable, is_writable,
                "unexpected writable flag for {pubkey}"
            );
        }

        let strict_create = create_new_canonical_wsol_account(&wallet, &mint);
        assert_eq!(strict_create.program_id, create.program_id);
        assert_eq!(strict_create.accounts, create.accounts);
        assert_eq!(strict_create.data, [0]);

        let transaction = build_wrap_sol_transaction(wallet, 42, true)?;
        let instructions = &transaction.message.instructions;

        assert_eq!(instructions.len(), 3);
        // Exact wrap must abort if the planned-missing ATA appears before execution.
        assert_eq!(instructions[0].data, [0]);
        assert_eq!(account_key(&transaction, 0, 0), wallet);
        assert_eq!(account_key(&transaction, 0, 1), canonical);
        assert_eq!(account_key(&transaction, 0, 3), mint);
        assert_eq!(
            bincode::deserialize::<SystemInstruction>(&instructions[1].data)?,
            SystemInstruction::Transfer { lamports: 42 }
        );
        assert_eq!(account_key(&transaction, 1, 0), wallet);
        assert_eq!(account_key(&transaction, 1, 1), canonical);
        assert!(matches!(
            TokenInstruction::unpack(&instructions[2].data)?,
            TokenInstruction::SyncNative
        ));
        assert_eq!(account_key(&transaction, 2, 0), canonical);

        let reused = build_wrap_sol_transaction(wallet, 7, false)?;
        assert_eq!(reused.message.instructions.len(), 2);
        assert_eq!(
            bincode::deserialize::<SystemInstruction>(&reused.message.instructions[0].data)?,
            SystemInstruction::Transfer { lamports: 7 }
        );
        assert!(matches!(
            TokenInstruction::unpack(&reused.message.instructions[1].data)?,
            TokenInstruction::SyncNative
        ));
        Ok(())
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn unwrap_all_closes_canonical_to_and_by_the_same_trading_wallet() -> TestResult {
        let wallet = Pubkey::new_unique();
        let (_, canonical) = wrapped_sol_accounts(&wallet)?;
        let transaction = build_unwrap_wsol_all_transaction(wallet)?;
        let instruction = &transaction.message.instructions[0];

        assert_eq!(transaction.message.instructions.len(), 1);
        assert!(matches!(
            TokenInstruction::unpack(&instruction.data)?,
            TokenInstruction::CloseAccount
        ));
        assert_eq!(account_key(&transaction, 0, 0), canonical);
        assert_eq!(account_key(&transaction, 0, 1), wallet);
        assert_eq!(account_key(&transaction, 0, 2), wallet);
        assert!(transaction
            .message
            .is_signer(instruction.accounts[2] as usize));
        Ok(())
    }

    #[test]
    fn ordinary_sol_builders_never_close_the_canonical_account() -> TestResult {
        let wallet = Pubkey::new_unique();
        let recipient = Pubkey::new_unique();
        let program_id = Pubkey::new_unique();
        let (_, canonical) = wrapped_sol_accounts(&wallet)?;
        let seed = "0123456789abcdef0123456789abcdef";
        let temporary = temporary_wsol_account(&wallet, seed)?;
        let transactions = [
            build_sol_split_transaction(&program_id, wallet, &market(), 10, 10, true)?,
            build_sol_merge_transaction(&program_id, wallet, &market(), 10, true)?,
            build_sol_redeem_transaction(&program_id, wallet, Pubkey::new_unique(), 10, 0, true)?,
            build_direct_native_withdraw_transaction(wallet, recipient, 10),
            build_temporary_native_withdraw_transaction(
                wallet, recipient, 10, 5, 2_039_280, seed, temporary,
            )?,
        ];

        for transaction in transactions {
            for instruction in &transaction.message.instructions {
                if transaction.message.account_keys[instruction.program_id_index as usize]
                    == spl_token_interface::id()
                    && matches!(
                        TokenInstruction::unpack(&instruction.data),
                        Ok(TokenInstruction::CloseAccount)
                    )
                {
                    assert_ne!(
                        transaction.message.account_keys[instruction.accounts[0] as usize],
                        canonical
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    /// Pins the shared preimage encoding and lowercase 32-byte seed text.
    fn native_withdraw_seed_is_byte_exact_and_lowercase_hex() {
        let blockhash = Hash::default();
        let wallet = Pubkey::new_from_array([1; 32]);
        let recipient = Pubkey::new_from_array([2; 32]);

        let seed = native_withdraw_seed(&blockhash, &wallet, &recipient, 0x0102_0304_0506_0708, 7);

        assert_eq!(seed, "4dce744c636478f024df5aefd987f933");
        assert_eq!(seed.len(), 32);
        assert_eq!(
            temporary_wsol_account(&wallet, &seed).unwrap().to_string(),
            "71S4MLz9scZhY8BomAjfTkVn6HhFo8yFU7G6tSLto5g6"
        );
    }

    #[test]
    /// Proves the atomic conversion orders all instructions and closes only the temporary account.
    fn temporary_withdraw_closes_only_the_seeded_account_before_native_transfer() {
        let wallet = Pubkey::new_unique();
        let recipient = Pubkey::new_unique();
        let seed = "0123456789abcdef0123456789abcdef";
        let temporary = temporary_wsol_account(&wallet, seed).unwrap();
        let (_, canonical) = wrapped_sol_accounts(&wallet).unwrap();
        let transaction = build_temporary_native_withdraw_transaction(
            wallet, recipient, 50, 25, 2_039_280, seed, temporary,
        )
        .unwrap();
        let instructions = &transaction.message.instructions;
        assert_eq!(instructions.len(), 5);

        let create: SystemInstruction = bincode::deserialize(&instructions[0].data).unwrap();
        let SystemInstruction::CreateAccountWithSeed {
            seed: created_seed,
            lamports,
            space,
            ..
        } = create
        else {
            panic!("expected CreateAccountWithSeed");
        };
        assert_eq!(created_seed, seed);
        assert_eq!(lamports, 2_039_280);
        assert_eq!(space, TOKEN_ACCOUNT_SPACE as u64);
        assert!(matches!(
            TokenInstruction::unpack(&instructions[1].data).unwrap(),
            TokenInstruction::InitializeAccount3 { owner } if owner == wallet
        ));
        assert!(matches!(
            TokenInstruction::unpack(&instructions[2].data).unwrap(),
            TokenInstruction::Transfer { amount: 25 }
        ));
        assert_eq!(
            transaction.message.account_keys[instructions[2].accounts[0] as usize],
            canonical
        );
        assert_eq!(
            transaction.message.account_keys[instructions[2].accounts[1] as usize],
            temporary
        );
        assert!(matches!(
            TokenInstruction::unpack(&instructions[3].data).unwrap(),
            TokenInstruction::CloseAccount
        ));
        assert_eq!(
            transaction.message.account_keys[instructions[3].accounts[0] as usize],
            temporary
        );
        assert_ne!(
            transaction.message.account_keys[instructions[3].accounts[0] as usize],
            canonical
        );
        assert_eq!(
            bincode::deserialize::<SystemInstruction>(&instructions[4].data).unwrap(),
            SystemInstruction::Transfer { lamports: 50 }
        );
        assert_eq!(
            transaction.message.account_keys[instructions[4].accounts[1] as usize],
            recipient
        );
    }
}
