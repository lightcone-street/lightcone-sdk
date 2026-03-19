//! Fluent builders for deposit and withdraw operations.
//!
//! Created via `client.positions().deposit().await` and `client.positions().withdraw().await`.

use crate::client::LightconeClient;
use crate::domain::market::Market;
use crate::error::SdkError;
use crate::program::instructions;
use crate::program::types::{
    DepositToGlobalParams, GlobalToMarketDepositParams, WithdrawFromGlobalParams,
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
        let user = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        // Re-set user since we consumed it for validation
        let ix = Self {
            user: Some(user),
            ..self
        }
        .build_ix()
        .await?;
        Ok(Transaction::new_with_payer(&[ix], Some(&user)))
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
        let user = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let ix = Self {
            user: Some(user),
            ..self
        }
        .build_ix()
        .await?;
        Ok(Transaction::new_with_payer(&[ix], Some(&user)))
    }
}
