//! Fluent builders for deposit and withdraw operations.
//!
//! Created via `client.positions().deposit().await` and `client.positions().withdraw().await`.

use crate::client::LightconeClient;
use crate::domain::market::Market;
use crate::error::SdkError;
use crate::program::instructions;
use crate::program::types::{
    DepositToGlobalParams, ExtendPositionTokensParams, GlobalToMarketDepositParams,
    InitPositionTokensParams, RedeemWinningsParams, WithdrawFromGlobalParams,
    WithdrawFromPositionParams,
};
use crate::shared::DepositSource;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;

// ─── DepositBuilder ─────────────────────────────────────────────────────────

/// Fluent builder for deposit operations.
///
/// Created via `client.positions().deposit().await` — direct construction is not exposed.
/// Pre-seeded with the client's deposit source setting.
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
                let num_outcomes = market.outcomes.len() as u8;
                Ok(instructions::build_global_to_market_deposit_ix(
                    &GlobalToMarketDepositParams {
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

// ─── WithdrawBuilder ────────────────────────────────────────────────────────

/// Fluent builder for withdraw operations.
///
/// Created via `client.positions().withdraw().await` — direct construction is not exposed.
/// Pre-seeded with the client's deposit source setting.
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
///     .with_market_deposit_source(&market, 255, false)
///     .build_ix()
///     .await?;
/// ```
pub struct WithdrawBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    mint: Option<Pubkey>,
    amount: Option<u64>,
    market: Option<&'a Market>,
    outcome_index: Option<u8>,
    is_token_2022: bool,
    deposit_source: Option<DepositSource>,
}

impl<'a> WithdrawBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient, deposit_source: DepositSource) -> Self {
        Self {
            client,
            user: None,
            mint: None,
            amount: None,
            market: None,
            outcome_index: None,
            is_token_2022: false,
            deposit_source: Some(deposit_source),
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

    /// Set the market reference (required when deposit source is `Market`).
    ///
    /// Use this when the client is already configured with `DepositSource::Market`.
    /// Otherwise, prefer `with_market_deposit_source()` to set all market params at once.
    pub fn market(mut self, market: &'a Market) -> Self {
        self.market = Some(market);
        self
    }

    /// Set the outcome index (required for market withdrawals). Use `255` for collateral.
    pub fn outcome_index(mut self, index: u8) -> Self {
        self.outcome_index = Some(index);
        self
    }

    /// Set whether this is a Token-2022 (conditional) token. Defaults to `false`.
    pub fn token_2022(mut self, is_token_2022: bool) -> Self {
        self.is_token_2022 = is_token_2022;
        self
    }

    /// Override the deposit source for this call.
    pub fn deposit_source(mut self, source: DepositSource) -> Self {
        self.deposit_source = Some(source);
        self
    }

    /// Set deposit source to `Market` and provide all required market parameters.
    pub fn with_market_deposit_source(
        mut self,
        market: &'a Market,
        outcome_index: u8,
        is_token_2022: bool,
    ) -> Self {
        self.deposit_source = Some(DepositSource::Market);
        self.market = Some(market);
        self.outcome_index = Some(outcome_index);
        self.is_token_2022 = is_token_2022;
        self
    }

    /// Set deposit source to `Global`.
    pub fn with_global_deposit_source(mut self) -> Self {
        self.deposit_source = Some(DepositSource::Global);
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
                    SdkError::Validation(
                        "outcome_index is required for Market withdrawal".into(),
                    )
                })?;
                Ok(instructions::build_withdraw_from_position_ix(
                    &WithdrawFromPositionParams {
                        user,
                        market: market_pubkey,
                        mint,
                        amount,
                        outcome_index,
                    },
                    self.is_token_2022,
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
///     .winning_outcome(0)
///     .sign_and_submit()
///     .await?;
/// ```
pub struct RedeemWinningsBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    market: Option<Pubkey>,
    mint: Option<Pubkey>,
    amount: Option<u64>,
    winning_outcome: Option<u8>,
}

impl<'a> RedeemWinningsBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient) -> Self {
        Self {
            client,
            user: None,
            market: None,
            mint: None,
            amount: None,
            winning_outcome: None,
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

    /// Set the winning outcome index.
    pub fn winning_outcome(mut self, winning_outcome: u8) -> Self {
        self.winning_outcome = Some(winning_outcome);
        self
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
        let winning_outcome = self
            .winning_outcome
            .ok_or_else(|| SdkError::Validation("winning_outcome is required".into()))?;

        Ok(instructions::build_redeem_winnings_ix(
            &RedeemWinningsParams {
                user,
                market,
                deposit_mint: mint,
                amount,
            },
            winning_outcome,
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

/// Fluent builder for withdraw-from-position operations.
///
/// Created via `client.positions().withdraw_from_position()` — direct construction is not exposed.
///
/// # Example
///
/// ```rust,ignore
/// let tx_signature = client.positions().withdraw_from_position()
///     .user(keypair.pubkey())
///     .market(market_pubkey)
///     .mint(mint_pubkey)
///     .amount(1_000_000)
///     .outcome_index(0)
///     .sign_and_submit()
///     .await?;
/// ```
pub struct WithdrawFromPositionBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    market: Option<Pubkey>,
    mint: Option<Pubkey>,
    amount: Option<u64>,
    outcome_index: Option<u8>,
    is_token_2022: bool,
}

impl<'a> WithdrawFromPositionBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient) -> Self {
        Self {
            client,
            user: None,
            market: None,
            mint: None,
            amount: None,
            outcome_index: None,
            is_token_2022: false,
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

    /// Set the outcome index.
    pub fn outcome_index(mut self, outcome_index: u8) -> Self {
        self.outcome_index = Some(outcome_index);
        self
    }

    /// Set whether this is a Token-2022 (conditional) token. Defaults to `false`.
    pub fn token_2022(mut self, is_token_2022: bool) -> Self {
        self.is_token_2022 = is_token_2022;
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
        let mint = self
            .mint
            .ok_or_else(|| SdkError::Validation("mint is required".into()))?;
        let amount = self
            .amount
            .ok_or_else(|| SdkError::Validation("amount is required".into()))?;
        let outcome_index = self
            .outcome_index
            .ok_or_else(|| SdkError::Validation("outcome_index is required".into()))?;

        Ok(instructions::build_withdraw_from_position_ix(
            &WithdrawFromPositionParams {
                user,
                market,
                mint,
                amount,
                outcome_index,
            },
            self.is_token_2022,
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
///     .payer(keypair.pubkey())
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
    payer: Option<Pubkey>,
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
            payer: None,
            user: None,
            market: None,
            lookup_table: None,
            deposit_mints: None,
            num_outcomes: None,
        }
    }

    /// Set the payer's public key (signer).
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
        let payer = self
            .payer
            .ok_or_else(|| SdkError::Validation("payer is required".into()))?;
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
                payer,
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
        let payer = self
            .payer
            .ok_or_else(|| SdkError::Validation("payer is required".into()))?;
        let instruction = self.build_ix()?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&payer)))
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
}

impl<'a> DepositToGlobalBuilder<'a> {
    pub(crate) fn new(client: &'a LightconeClient) -> Self {
        Self {
            client,
            user: None,
            mint: None,
            amount: None,
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

        Ok(instructions::build_deposit_to_global_ix(
            &DepositToGlobalParams { user, mint, amount },
            &self.client.program_id,
        ))
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
