//! Fluent builders for mint and merge complete-set operations.
//!
//! Created via `client.markets().mint_complete_set()` and `client.markets().merge_complete_set()`.

use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::program::instructions;
use crate::program::types::{MergeCompleteSetParams, MintCompleteSetParams};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_transaction::Transaction;

// ─── MintCompleteSetBuilder ─────────────────────────────────────────────────

/// Fluent builder for mint-complete-set operations.
///
/// Created via `client.markets().mint_complete_set()` — direct construction is not exposed.
///
/// # Example
///
/// ```rust,ignore
/// let signature = client.markets().mint_complete_set()
///     .user(keypair.pubkey())
///     .market(market_pubkey)
///     .mint(deposit_mint)
///     .amount(1_000_000)
///     .num_outcomes(2)
///     .sign_and_submit()
///     .await?;
/// ```
pub struct MintCompleteSetBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    market: Option<Pubkey>,
    mint: Option<Pubkey>,
    amount: Option<u64>,
    num_outcomes: Option<u8>,
}

impl<'a> MintCompleteSetBuilder<'a> {
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

    /// Set the user's public key (payer and recipient).
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

    /// Set the amount of collateral to deposit.
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the number of outcomes in the market.
    pub fn num_outcomes(mut self, num_outcomes: u8) -> Self {
        self.num_outcomes = Some(num_outcomes);
        self
    }

    /// Build a mint-complete-set instruction.
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

        Ok(instructions::build_mint_complete_set_ix(
            &MintCompleteSetParams {
                user,
                market,
                deposit_mint: mint,
                amount,
            },
            num_outcomes,
            &self.client.program_id,
        ))
    }

    /// Build a mint-complete-set transaction.
    pub fn build_tx(self) -> Result<Transaction, SdkError> {
        let payer = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let instruction = self.build_ix()?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&payer)))
    }

    /// Build, sign, and submit the mint-complete-set transaction.
    pub async fn sign_and_submit(self) -> Result<String, SdkError> {
        let client = self.client;
        let transaction = self.build_tx()?;
        client.sign_and_submit_tx(transaction).await
    }
}

// ─── MergeCompleteSetBuilder ────────────────────────────────────────────────

/// Fluent builder for merge-complete-set operations.
///
/// Created via `client.markets().merge_complete_set()` — direct construction is not exposed.
///
/// # Example
///
/// ```rust,ignore
/// let signature = client.markets().merge_complete_set()
///     .user(keypair.pubkey())
///     .market(market_pubkey)
///     .mint(deposit_mint)
///     .amount(1_000_000)
///     .num_outcomes(2)
///     .sign_and_submit()
///     .await?;
/// ```
pub struct MergeCompleteSetBuilder<'a> {
    client: &'a LightconeClient,
    user: Option<Pubkey>,
    market: Option<Pubkey>,
    mint: Option<Pubkey>,
    amount: Option<u64>,
    num_outcomes: Option<u8>,
}

impl<'a> MergeCompleteSetBuilder<'a> {
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

    /// Set the amount of each outcome token to burn.
    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Set the number of outcomes in the market.
    pub fn num_outcomes(mut self, num_outcomes: u8) -> Self {
        self.num_outcomes = Some(num_outcomes);
        self
    }

    /// Build a merge-complete-set instruction.
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

        Ok(instructions::build_merge_complete_set_ix(
            &MergeCompleteSetParams {
                user,
                market,
                deposit_mint: mint,
                amount,
            },
            num_outcomes,
            &self.client.program_id,
        ))
    }

    /// Build a merge-complete-set transaction.
    pub fn build_tx(self) -> Result<Transaction, SdkError> {
        let payer = self
            .user
            .ok_or_else(|| SdkError::Validation("user is required".into()))?;
        let instruction = self.build_ix()?;
        Ok(Transaction::new_with_payer(&[instruction], Some(&payer)))
    }

    /// Build, sign, and submit the merge-complete-set transaction.
    pub async fn sign_and_submit(self) -> Result<String, SdkError> {
        let client = self.client;
        let transaction = self.build_tx()?;
        client.sign_and_submit_tx(transaction).await
    }
}
