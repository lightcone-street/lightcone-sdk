//! Type definitions for the Lightcone Pinocchio on-chain program.
//!
//! This module contains enums, parameter structs, and other type definitions
//! used for on-chain program interaction.

use solana_pubkey::Pubkey;

use crate::program::error::SdkError;

// ============================================================================
// Enums
// ============================================================================

/// Market status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MarketStatus {
    /// Market is pending activation
    Pending = 0,
    /// Market is active and trading is enabled
    Active = 1,
    /// Market has been resolved with a winning outcome
    Resolved = 2,
    /// Market has been cancelled
    Cancelled = 3,
}

impl TryFrom<u8> for MarketStatus {
    type Error = SdkError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MarketStatus::Pending),
            1 => Ok(MarketStatus::Active),
            2 => Ok(MarketStatus::Resolved),
            3 => Ok(MarketStatus::Cancelled),
            _ => Err(SdkError::InvalidMarketStatus(value)),
        }
    }
}

/// Order side enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OrderSide {
    /// Bid/Buy - maker wants to buy base tokens, gives quote tokens
    Bid = 0,
    /// Ask/Sell - maker wants to sell base tokens, receives quote tokens
    Ask = 1,
}

impl TryFrom<u8> for OrderSide {
    type Error = SdkError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(OrderSide::Bid),
            1 => Ok(OrderSide::Ask),
            _ => Err(SdkError::InvalidSide(value)),
        }
    }
}

// ============================================================================
// Parameter Structs
// ============================================================================

/// Parameters for creating a market
#[derive(Debug, Clone)]
pub struct CreateMarketParams {
    /// Authority pubkey (must be exchange authority)
    pub authority: Pubkey,
    /// Number of outcomes (2-6)
    pub num_outcomes: u8,
    /// Oracle pubkey that can settle the market
    pub oracle: Pubkey,
    /// Question ID (32 bytes)
    pub question_id: [u8; 32],
}

/// Metadata for a single outcome token
#[derive(Debug, Clone)]
pub struct OutcomeMetadata {
    /// Token name (max 32 chars)
    pub name: String,
    /// Token symbol (max 10 chars)
    pub symbol: String,
    /// Token URI (max 200 chars)
    pub uri: String,
}

/// Parameters for adding a deposit mint to a market
#[derive(Debug, Clone)]
pub struct AddDepositMintParams {
    /// Payer for account creation
    pub payer: Pubkey,
    /// Market ID
    pub market_id: u64,
    /// Deposit mint pubkey
    pub deposit_mint: Pubkey,
    /// Metadata for each outcome token
    pub outcome_metadata: Vec<OutcomeMetadata>,
}

/// Parameters for minting a complete set
#[derive(Debug, Clone)]
pub struct MintCompleteSetParams {
    /// User pubkey (payer and recipient)
    pub user: Pubkey,
    /// Market pubkey
    pub market: Pubkey,
    /// Deposit mint pubkey
    pub deposit_mint: Pubkey,
    /// Amount of collateral to deposit
    pub amount: u64,
}

/// Parameters for merging a complete set
#[derive(Debug, Clone)]
pub struct MergeCompleteSetParams {
    /// User pubkey
    pub user: Pubkey,
    /// Market pubkey
    pub market: Pubkey,
    /// Deposit mint pubkey
    pub deposit_mint: Pubkey,
    /// Amount of each outcome token to burn
    pub amount: u64,
}

/// Parameters for settling a market
#[derive(Debug, Clone)]
pub struct SettleMarketParams {
    /// Oracle pubkey (must match market oracle)
    pub oracle: Pubkey,
    /// Market ID
    pub market_id: u64,
    /// Winning outcome index
    pub winning_outcome: u8,
}

/// Parameters for redeeming winnings
#[derive(Debug, Clone)]
pub struct RedeemWinningsParams {
    /// User pubkey
    pub user: Pubkey,
    /// Market pubkey
    pub market: Pubkey,
    /// Deposit mint pubkey
    pub deposit_mint: Pubkey,
    /// Amount of winning tokens to redeem
    pub amount: u64,
}

/// Parameters for withdrawing from a position
#[derive(Debug, Clone)]
pub struct WithdrawFromPositionParams {
    /// User pubkey (must be position owner)
    pub user: Pubkey,
    /// Market pubkey
    pub market: Pubkey,
    /// Mint pubkey (deposit or conditional)
    pub mint: Pubkey,
    /// Amount to withdraw
    pub amount: u64,
    /// Outcome index (255 for collateral)
    pub outcome_index: u8,
}

/// Parameters for activating a market
#[derive(Debug, Clone)]
pub struct ActivateMarketParams {
    /// Authority pubkey (must be exchange authority)
    pub authority: Pubkey,
    /// Market ID
    pub market_id: u64,
}

/// Parameters for creating a bid order
#[derive(Debug, Clone)]
pub struct BidOrderParams {
    /// Order nonce (unique per user, u32 range)
    pub nonce: u32,
    /// Maker pubkey
    pub maker: Pubkey,
    /// Market pubkey
    pub market: Pubkey,
    /// Base mint (token being bought)
    pub base_mint: Pubkey,
    /// Quote mint (token used for payment)
    pub quote_mint: Pubkey,
    /// Quote tokens to give (amount_in)
    pub amount_in: u64,
    /// Base tokens to receive (amount_out)
    pub amount_out: u64,
    /// Expiration timestamp (0 for no expiration)
    pub expiration: i64,
}

/// Parameters for creating an ask order
#[derive(Debug, Clone)]
pub struct AskOrderParams {
    /// Order nonce (unique per user, u32 range)
    pub nonce: u32,
    /// Maker pubkey
    pub maker: Pubkey,
    /// Market pubkey
    pub market: Pubkey,
    /// Base mint (token being sold)
    pub base_mint: Pubkey,
    /// Quote mint (token to receive)
    pub quote_mint: Pubkey,
    /// Base tokens to give (amount_in)
    pub amount_in: u64,
    /// Quote tokens to receive (amount_out)
    pub amount_out: u64,
    /// Expiration timestamp (0 for no expiration)
    pub expiration: i64,
}

/// Parameters for matching orders
#[derive(Debug, Clone)]
pub struct MatchOrdersMultiParams {
    /// Operator pubkey (must be exchange operator)
    pub operator: Pubkey,
    /// Market pubkey
    pub market: Pubkey,
    /// Base mint pubkey
    pub base_mint: Pubkey,
    /// Quote mint pubkey
    pub quote_mint: Pubkey,
    /// Taker order (signed)
    pub taker_order: crate::program::orders::SignedOrder,
    /// Maker orders (signed)
    pub maker_orders: Vec<crate::program::orders::SignedOrder>,
    /// Fill amounts for each maker (maker side)
    pub maker_fill_amounts: Vec<u64>,
    /// Fill amounts for each maker (taker side)
    pub taker_fill_amounts: Vec<u64>,
    /// Bitmask indicating which orders require full fill (bit i = maker i, bit 7 = taker)
    pub full_fill_bitmask: u8,
}

/// Parameters for creating an on-chain orderbook
#[derive(Debug, Clone)]
pub struct CreateOrderbookParams {
    /// Payer for account creation
    pub payer: Pubkey,
    /// Market pubkey
    pub market: Pubkey,
    /// Mint A pubkey
    pub mint_a: Pubkey,
    /// Mint B pubkey
    pub mint_b: Pubkey,
    /// Recent slot for ALT creation
    pub recent_slot: u64,
}

/// Parameters for setting a new authority
#[derive(Debug, Clone)]
pub struct SetAuthorityParams {
    /// Current authority pubkey
    pub current_authority: Pubkey,
    /// New authority pubkey
    pub new_authority: Pubkey,
}

/// Parameters for whitelisting a deposit token for global deposits
#[derive(Debug, Clone)]
pub struct WhitelistDepositTokenParams {
    /// Authority pubkey (must be exchange authority)
    pub authority: Pubkey,
    /// Mint pubkey to whitelist
    pub mint: Pubkey,
}

/// Parameters for depositing tokens to a global deposit account
#[derive(Debug, Clone)]
pub struct DepositToGlobalParams {
    /// User pubkey (depositor)
    pub user: Pubkey,
    /// Deposit token mint pubkey
    pub mint: Pubkey,
    /// Amount to deposit
    pub amount: u64,
}

/// Parameters for transferring from global deposit to a market vault
#[derive(Debug, Clone)]
pub struct GlobalToMarketDepositParams {
    /// User pubkey
    pub user: Pubkey,
    /// Market pubkey
    pub market: Pubkey,
    /// Deposit mint pubkey (e.g., USDC)
    pub deposit_mint: Pubkey,
    /// Amount of collateral to transfer and mint
    pub amount: u64,
}

/// Parameters for initializing position token accounts and ALT
#[derive(Debug, Clone)]
pub struct InitPositionTokensParams {
    /// User pubkey (payer and position owner)
    pub user: Pubkey,
    /// Market pubkey
    pub market: Pubkey,
    /// Deposit mint pubkey
    pub deposit_mint: Pubkey,
    /// Recent slot for ALT address derivation
    pub recent_slot: u64,
}

/// Parameters for deposit-and-swap (atomic deposit + mint + swap)
#[derive(Debug, Clone)]
pub struct DepositAndSwapParams {
    /// Operator pubkey (must be exchange operator)
    pub operator: Pubkey,
    /// Market pubkey
    pub market: Pubkey,
    /// Deposit mint pubkey (collateral, e.g., USDC)
    pub deposit_mint: Pubkey,
    /// Base mint pubkey (conditional token A)
    pub base_mint: Pubkey,
    /// Quote mint pubkey (conditional token B)
    pub quote_mint: Pubkey,
    /// Taker order (signed)
    pub taker_order: crate::program::orders::SignedOrder,
    /// Maker orders (signed)
    pub maker_orders: Vec<crate::program::orders::SignedOrder>,
    /// Fill amounts for each maker (maker side)
    pub maker_fill_amounts: Vec<u64>,
    /// Fill amounts for each maker (taker side)
    pub taker_fill_amounts: Vec<u64>,
    /// Bitmask indicating which orders require full fill
    pub full_fill_bitmask: u8,
}
