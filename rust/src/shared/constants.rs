//! Constants for the Lightcone Pinocchio program.
//!
//! This module contains all program IDs, seeds, discriminators, and size constants
//! matching the on-chain program exactly.

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

// ============================================================================
// Program IDs
// ============================================================================

lazy_static::lazy_static! {
    /// Lightcone Pinocchio Program ID
    pub static ref PROGRAM_ID: Pubkey = Pubkey::from_str("Aumw7EC9nnxDjQFzr1fhvXvnG3Rn3Bb5E3kbcbLrBdEk").unwrap();

    /// SPL Token Program ID
    pub static ref TOKEN_PROGRAM_ID: Pubkey = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

    /// Token-2022 Program ID (for conditional tokens)
    pub static ref TOKEN_2022_PROGRAM_ID: Pubkey = Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb").unwrap();

    /// Associated Token Account Program ID
    pub static ref ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();

    /// System Program ID
    pub static ref SYSTEM_PROGRAM_ID: Pubkey = Pubkey::from_str("11111111111111111111111111111111").unwrap();

    /// Rent Sysvar ID
    pub static ref RENT_SYSVAR_ID: Pubkey = Pubkey::from_str("SysvarRent111111111111111111111111111111111").unwrap();

    /// Instructions Sysvar ID (for Ed25519 verification)
    pub static ref INSTRUCTIONS_SYSVAR_ID: Pubkey = Pubkey::from_str("Sysvar1nstructions1111111111111111111111111").unwrap();

    /// Ed25519 Program ID (for signature verification)
    pub static ref ED25519_PROGRAM_ID: Pubkey = Pubkey::from_str("Ed25519SigVerify111111111111111111111111111").unwrap();
}

// ============================================================================
// Instruction Discriminators
// ============================================================================

/// Instruction discriminators (single byte indices)
pub mod instruction {
    pub const INITIALIZE: u8 = 0;
    pub const CREATE_MARKET: u8 = 1;
    pub const ADD_DEPOSIT_MINT: u8 = 2;
    pub const MINT_COMPLETE_SET: u8 = 3;
    pub const MERGE_COMPLETE_SET: u8 = 4;
    pub const CANCEL_ORDER: u8 = 5;
    pub const INCREMENT_NONCE: u8 = 6;
    pub const SETTLE_MARKET: u8 = 7;
    pub const REDEEM_WINNINGS: u8 = 8;
    pub const SET_PAUSED: u8 = 9;
    pub const SET_OPERATOR: u8 = 10;
    pub const WITHDRAW_FROM_POSITION: u8 = 11;
    pub const ACTIVATE_MARKET: u8 = 12;
    pub const MATCH_ORDERS_MULTI: u8 = 13;
}

// ============================================================================
// Account Discriminators (8 bytes each)
// ============================================================================

/// Exchange account discriminator
pub const EXCHANGE_DISCRIMINATOR: [u8; 8] = *b"exchange";
/// Market account discriminator
pub const MARKET_DISCRIMINATOR: [u8; 8] = *b"market\0\0";
/// Order status account discriminator
pub const ORDER_STATUS_DISCRIMINATOR: [u8; 8] = *b"ordstat\0";
/// User nonce account discriminator
pub const USER_NONCE_DISCRIMINATOR: [u8; 8] = *b"usrnonce";
/// Position account discriminator
pub const POSITION_DISCRIMINATOR: [u8; 8] = *b"position";

// ============================================================================
// PDA Seeds
// ============================================================================

/// Exchange PDA seed
pub const EXCHANGE_SEED: &[u8] = b"central_state";
/// Market PDA seed
pub const MARKET_SEED: &[u8] = b"market";
/// Vault PDA seed (for deposit token accounts)
pub const VAULT_SEED: &[u8] = b"market_deposit_token_account";
/// Mint authority PDA seed
pub const MINT_AUTHORITY_SEED: &[u8] = b"market_mint_authority";
/// Conditional mint PDA seed
pub const CONDITIONAL_MINT_SEED: &[u8] = b"conditional_mint";
/// Order status PDA seed
pub const ORDER_STATUS_SEED: &[u8] = b"order_status";
/// User nonce PDA seed
pub const USER_NONCE_SEED: &[u8] = b"user_nonce";
/// Position PDA seed
pub const POSITION_SEED: &[u8] = b"position";

// ============================================================================
// Account Sizes
// ============================================================================

/// Exchange account size in bytes
pub const EXCHANGE_SIZE: usize = 88;
/// Market account size in bytes
pub const MARKET_SIZE: usize = 120;
/// Order status account size in bytes
pub const ORDER_STATUS_SIZE: usize = 24;
/// User nonce account size in bytes
pub const USER_NONCE_SIZE: usize = 16;
/// Position account size in bytes
pub const POSITION_SIZE: usize = 80;

// ============================================================================
// Order Sizes
// ============================================================================

/// Full order size in bytes
pub const FULL_ORDER_SIZE: usize = 225;
/// Compact order size in bytes
pub const COMPACT_ORDER_SIZE: usize = 65;
/// Signature size in bytes
pub const SIGNATURE_SIZE: usize = 64;

// ============================================================================
// Limits
// ============================================================================

/// Maximum outcomes per market (limited by compute budget)
pub const MAX_OUTCOMES: u8 = 6;
/// Minimum outcomes per market
pub const MIN_OUTCOMES: u8 = 2;
/// Maximum makers in a single match_orders_multi instruction
pub const MAX_MAKERS: usize = 5;
