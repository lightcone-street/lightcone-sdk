//! Constants for the Lightcone Pinocchio program.
//!
//! This module contains all program IDs, seeds, discriminators, and size constants
//! matching the on-chain program exactly.

use solana_pubkey::Pubkey;
use std::str::FromStr;

// ============================================================================
// Program IDs
// ============================================================================

lazy_static::lazy_static! {
    /// Lightcone Pinocchio Program ID
    pub static ref PROGRAM_ID: Pubkey = Pubkey::from_str("9cCFQnmWqWmZF3LNdAVWTh7ECGJK4tCVPtgPMcYum81A").unwrap();

    /// Address Lookup Table Program ID
    pub static ref ALT_PROGRAM_ID: Pubkey = Pubkey::from_str("AddressLookupTab1e1111111111111111111111111").unwrap();
}

/// SPL Token Program ID
pub const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;

/// Token-2022 Program ID (for conditional tokens)
pub const TOKEN_2022_PROGRAM_ID: Pubkey = spl_token_2022::ID;

/// Associated Token Account Program ID
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey = spl_associated_token_account::ID;

/// System Program ID
pub const SYSTEM_PROGRAM_ID: Pubkey = solana_sdk_ids::system_program::ID;

/// Rent Sysvar ID
pub const RENT_SYSVAR_ID: Pubkey = solana_sdk_ids::sysvar::rent::ID;

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
    pub const SET_AUTHORITY: u8 = 14;
    pub const CREATE_ORDERBOOK: u8 = 15;
    pub const WHITELIST_DEPOSIT_TOKEN: u8 = 16;
    pub const DEPOSIT_TO_GLOBAL: u8 = 17;
    pub const GLOBAL_TO_MARKET_DEPOSIT: u8 = 18;
    pub const INIT_POSITION_TOKENS: u8 = 19;
    pub const DEPOSIT_AND_SWAP: u8 = 20;
}

// ============================================================================
// Account Discriminators (8 bytes each, SHA-256 hash bytes)
// ============================================================================

/// Exchange account discriminator
pub const EXCHANGE_DISCRIMINATOR: [u8; 8] = [0x1e, 0xc8, 0xdc, 0x95, 0x03, 0x3d, 0x68, 0x32];
/// Market account discriminator
pub const MARKET_DISCRIMINATOR: [u8; 8] = [0xdb, 0xbe, 0xd5, 0x37, 0x00, 0xe3, 0xc6, 0x9a];
/// Order status account discriminator
pub const ORDER_STATUS_DISCRIMINATOR: [u8; 8] = [0x2e, 0x5a, 0xf1, 0x49, 0xb2, 0x68, 0x41, 0x03];
/// User nonce account discriminator
pub const USER_NONCE_DISCRIMINATOR: [u8; 8] = [0xeb, 0x85, 0x01, 0xf3, 0x12, 0x87, 0x58, 0xe0];
/// Position account discriminator
pub const POSITION_DISCRIMINATOR: [u8; 8] = [0xaa, 0xbc, 0x8f, 0xe4, 0x7a, 0x40, 0xf7, 0xd0];
/// Orderbook account discriminator
pub const ORDERBOOK_DISCRIMINATOR: [u8; 8] = [0x2b, 0x22, 0x19, 0x71, 0xc3, 0x45, 0x48, 0x07];
/// GlobalDepositToken account discriminator
pub const GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR: [u8; 8] = [0x25, 0xbe, 0xa1, 0xe8, 0x7b, 0x92, 0x2a, 0x57];

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
/// Orderbook PDA seed
pub const ORDERBOOK_SEED: &[u8] = b"orderbook";
/// GlobalDepositToken PDA seed (also used for user global deposit accounts)
pub const GLOBAL_DEPOSIT_TOKEN_SEED: &[u8] = b"global_deposit";

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
/// Orderbook account size in bytes
pub const ORDERBOOK_SIZE: usize = 144;
/// GlobalDepositToken account size in bytes
pub const GLOBAL_DEPOSIT_TOKEN_SIZE: usize = 48;

// ============================================================================
// Order Sizes
// ============================================================================

/// Signed order size in bytes
pub const SIGNED_ORDER_SIZE: usize = 225;
/// Order size in bytes (compact on-chain format)
pub const ORDER_SIZE: usize = 29;
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
pub const MAX_MAKERS: usize = 7;
