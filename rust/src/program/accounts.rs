//! Account structures and deserialization for Lightcone Pinocchio.
//!
//! This module contains all on-chain account structures with their exact
//! byte layouts matching the program.

use solana_pubkey::Pubkey;

use crate::program::constants::{
    EXCHANGE_DISCRIMINATOR, EXCHANGE_SIZE, GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR,
    GLOBAL_DEPOSIT_TOKEN_SIZE, MARKET_DISCRIMINATOR, MARKET_SIZE, ORDERBOOK_DISCRIMINATOR,
    ORDERBOOK_SIZE, ORDER_STATUS_DISCRIMINATOR, ORDER_STATUS_SIZE, POSITION_DISCRIMINATOR,
    POSITION_SIZE, USER_NONCE_DISCRIMINATOR, USER_NONCE_SIZE,
};
use crate::program::error::{SdkError, SdkResult};
use crate::program::types::MarketStatus;

/// Helper to extract a fixed-size array from a slice
#[inline]
fn read_bytes<const N: usize>(data: &[u8], offset: usize) -> [u8; N] {
    let mut arr = [0u8; N];
    arr.copy_from_slice(&data[offset..offset + N]);
    arr
}

/// Helper to read a Pubkey from data
#[inline]
fn read_pubkey(data: &[u8], offset: usize) -> Pubkey {
    Pubkey::new_from_array(read_bytes::<32>(data, offset))
}

/// Helper to read a u64 from data (little-endian)
#[inline]
fn read_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(read_bytes::<8>(data, offset))
}

// ============================================================================
// Exchange Account (88 bytes)
// ============================================================================

/// Exchange account - singleton state for the exchange
///
/// Layout:
/// - [0..8]   discriminator (8 bytes)
/// - [8..40]  authority (32 bytes)
/// - [40..72] operator (32 bytes)
/// - [72..80] market_count (8 bytes)
/// - [80]     paused (1 byte)
/// - [81]     bump (1 byte)
/// - [82..88] _padding (6 bytes)
#[derive(Debug, Clone)]
pub struct Exchange {
    /// Account discriminator
    pub discriminator: [u8; 8],
    /// Exchange authority (can pause, set operator, create markets)
    pub authority: Pubkey,
    /// Operator (can match orders)
    pub operator: Pubkey,
    /// Number of markets created
    pub market_count: u64,
    /// Whether the exchange is paused
    pub paused: bool,
    /// PDA bump seed
    pub bump: u8,
}

impl Exchange {
    /// Account size in bytes
    pub const LEN: usize = EXCHANGE_SIZE;

    /// Deserialize from account data
    pub fn deserialize(data: &[u8]) -> SdkResult<Self> {
        if data.len() < Self::LEN {
            return Err(SdkError::InvalidDataLength {
                expected: Self::LEN,
                actual: data.len(),
            });
        }

        let discriminator = read_bytes::<8>(data, 0);
        if discriminator != EXCHANGE_DISCRIMINATOR {
            return Err(SdkError::InvalidDiscriminator {
                expected: hex::encode(EXCHANGE_DISCRIMINATOR),
                actual: hex::encode(discriminator),
            });
        }

        Ok(Self {
            discriminator,
            authority: read_pubkey(data, 8),
            operator: read_pubkey(data, 40),
            market_count: read_u64(data, 72),
            paused: data[80] != 0,
            bump: data[81],
        })
    }

    /// Check if account data has the exchange discriminator
    pub fn is_exchange_account(data: &[u8]) -> bool {
        data.len() >= 8 && data[0..8] == EXCHANGE_DISCRIMINATOR
    }
}

// ============================================================================
// Market Account (120 bytes)
// ============================================================================

/// Market account - represents a market
///
/// Layout:
/// - [0..8]     discriminator (8 bytes)
/// - [8..16]   market_id (8 bytes)
/// - [16]       num_outcomes (1 byte)
/// - [17]       status (1 byte)
/// - [18]       winning_outcome (1 byte)
/// - [19]       has_winning_outcome (1 byte)
/// - [20]       bump (1 byte)
/// - [21..24]   _padding (3 bytes)
/// - [24..56]   oracle (32 bytes)
/// - [56..88]   question_id (32 bytes)
/// - [88..120]  condition_id (32 bytes)
#[derive(Debug, Clone)]
pub struct Market {
    /// Account discriminator
    pub discriminator: [u8; 8],
    /// Unique market ID
    pub market_id: u64,
    /// Number of possible outcomes (2-6)
    pub num_outcomes: u8,
    /// Current market status
    pub status: MarketStatus,
    /// Winning outcome index (255 if not resolved)
    pub winning_outcome: u8,
    /// Whether a winning outcome has been set
    pub has_winning_outcome: bool,
    /// PDA bump seed
    pub bump: u8,
    /// Oracle pubkey that can settle this market
    pub oracle: Pubkey,
    /// Question ID (32 bytes)
    pub question_id: [u8; 32],
    /// Condition ID derived from oracle + question_id + num_outcomes
    pub condition_id: [u8; 32],
}

impl Market {
    /// Account size in bytes
    pub const LEN: usize = MARKET_SIZE;

    /// Deserialize from account data
    pub fn deserialize(data: &[u8]) -> SdkResult<Self> {
        if data.len() < Self::LEN {
            return Err(SdkError::InvalidDataLength {
                expected: Self::LEN,
                actual: data.len(),
            });
        }

        let discriminator = read_bytes::<8>(data, 0);
        if discriminator != MARKET_DISCRIMINATOR {
            return Err(SdkError::InvalidDiscriminator {
                expected: hex::encode(MARKET_DISCRIMINATOR),
                actual: hex::encode(discriminator),
            });
        }

        Ok(Self {
            discriminator,
            market_id: read_u64(data, 8),
            num_outcomes: data[16],
            status: MarketStatus::try_from(data[17])?,
            winning_outcome: data[18],
            has_winning_outcome: data[19] != 0,
            bump: data[20],
            oracle: read_pubkey(data, 24),
            question_id: read_bytes::<32>(data, 56),
            condition_id: read_bytes::<32>(data, 88),
        })
    }

    /// Check if account data has the market discriminator
    pub fn is_market_account(data: &[u8]) -> bool {
        data.len() >= 8 && data[0..8] == MARKET_DISCRIMINATOR
    }
}

// ============================================================================
// Position Account (80 bytes)
// ============================================================================

/// Position account - user's custody account for a market
///
/// Layout:
/// - [0..8]   discriminator (8 bytes)
/// - [8..40]  owner (32 bytes)
/// - [40..72] market (32 bytes)
/// - [72]     bump (1 byte)
/// - [73..80] _padding (7 bytes)
#[derive(Debug, Clone)]
pub struct Position {
    /// Account discriminator
    pub discriminator: [u8; 8],
    /// Owner of this position
    pub owner: Pubkey,
    /// Market this position is for
    pub market: Pubkey,
    /// PDA bump seed
    pub bump: u8,
}

impl Position {
    /// Account size in bytes
    pub const LEN: usize = POSITION_SIZE;

    /// Deserialize from account data
    pub fn deserialize(data: &[u8]) -> SdkResult<Self> {
        if data.len() < Self::LEN {
            return Err(SdkError::InvalidDataLength {
                expected: Self::LEN,
                actual: data.len(),
            });
        }

        let discriminator = read_bytes::<8>(data, 0);
        if discriminator != POSITION_DISCRIMINATOR {
            return Err(SdkError::InvalidDiscriminator {
                expected: hex::encode(POSITION_DISCRIMINATOR),
                actual: hex::encode(discriminator),
            });
        }

        Ok(Self {
            discriminator,
            owner: read_pubkey(data, 8),
            market: read_pubkey(data, 40),
            bump: data[72],
        })
    }

    /// Check if account data has the position discriminator
    pub fn is_position_account(data: &[u8]) -> bool {
        data.len() >= 8 && data[0..8] == POSITION_DISCRIMINATOR
    }
}

// ============================================================================
// OrderStatus Account (24 bytes)
// ============================================================================

/// Order status account - tracks partial fills and cancellations
///
/// Layout:
/// - [0..8]   discriminator (8 bytes)
/// - [8..16]  remaining (8 bytes)
/// - [16]     is_cancelled (1 byte)
/// - [17..24] _padding (7 bytes)
#[derive(Debug, Clone)]
pub struct OrderStatus {
    /// Account discriminator
    pub discriminator: [u8; 8],
    /// Remaining maker_amount to be filled
    pub remaining: u64,
    /// Whether the order has been cancelled
    pub is_cancelled: bool,
}

impl OrderStatus {
    /// Account size in bytes
    pub const LEN: usize = ORDER_STATUS_SIZE;

    /// Deserialize from account data
    pub fn deserialize(data: &[u8]) -> SdkResult<Self> {
        if data.len() < Self::LEN {
            return Err(SdkError::InvalidDataLength {
                expected: Self::LEN,
                actual: data.len(),
            });
        }

        let discriminator = read_bytes::<8>(data, 0);
        if discriminator != ORDER_STATUS_DISCRIMINATOR {
            return Err(SdkError::InvalidDiscriminator {
                expected: hex::encode(ORDER_STATUS_DISCRIMINATOR),
                actual: hex::encode(discriminator),
            });
        }

        Ok(Self {
            discriminator,
            remaining: read_u64(data, 8),
            is_cancelled: data[16] != 0,
        })
    }

    /// Check if account data has the order status discriminator
    pub fn is_order_status_account(data: &[u8]) -> bool {
        data.len() >= 8 && data[0..8] == ORDER_STATUS_DISCRIMINATOR
    }
}

// ============================================================================
// UserNonce Account (16 bytes)
// ============================================================================

/// User nonce account - tracks user's current nonce for mass cancellation
///
/// Layout:
/// - [0..8]  discriminator (8 bytes)
/// - [8..16] nonce (8 bytes)
#[derive(Debug, Clone)]
pub struct UserNonce {
    /// Account discriminator
    pub discriminator: [u8; 8],
    /// Current nonce value
    pub nonce: u64,
}

impl UserNonce {
    /// Account size in bytes
    pub const LEN: usize = USER_NONCE_SIZE;

    /// Deserialize from account data
    pub fn deserialize(data: &[u8]) -> SdkResult<Self> {
        if data.len() < Self::LEN {
            return Err(SdkError::InvalidDataLength {
                expected: Self::LEN,
                actual: data.len(),
            });
        }

        let discriminator = read_bytes::<8>(data, 0);
        if discriminator != USER_NONCE_DISCRIMINATOR {
            return Err(SdkError::InvalidDiscriminator {
                expected: hex::encode(USER_NONCE_DISCRIMINATOR),
                actual: hex::encode(discriminator),
            });
        }

        Ok(Self {
            discriminator,
            nonce: read_u64(data, 8),
        })
    }

    /// Check if account data has the user nonce discriminator
    pub fn is_user_nonce_account(data: &[u8]) -> bool {
        data.len() >= 8 && data[0..8] == USER_NONCE_DISCRIMINATOR
    }
}

// ============================================================================
// Orderbook Account (144 bytes)
// ============================================================================

/// Orderbook account - on-chain orderbook with lookup table
///
/// Layout:
/// - [0..8]     discriminator (8 bytes)
/// - [8..40]    market (32 bytes)
/// - [40..72]   mint_a (32 bytes)
/// - [72..104]  mint_b (32 bytes)
/// - [104..136] lookup_table (32 bytes)
/// - [136]      bump (1 byte)
/// - [137..144] _padding (7 bytes)
#[derive(Debug, Clone)]
pub struct Orderbook {
    /// Account discriminator
    pub discriminator: [u8; 8],
    /// Market this orderbook is for
    pub market: Pubkey,
    /// Mint A
    pub mint_a: Pubkey,
    /// Mint B
    pub mint_b: Pubkey,
    /// Address lookup table
    pub lookup_table: Pubkey,
    /// PDA bump seed
    pub bump: u8,
}

impl Orderbook {
    /// Account size in bytes
    pub const LEN: usize = ORDERBOOK_SIZE;

    /// Deserialize from account data
    pub fn deserialize(data: &[u8]) -> SdkResult<Self> {
        if data.len() < Self::LEN {
            return Err(SdkError::InvalidDataLength {
                expected: Self::LEN,
                actual: data.len(),
            });
        }

        let discriminator = read_bytes::<8>(data, 0);
        if discriminator != ORDERBOOK_DISCRIMINATOR {
            return Err(SdkError::InvalidDiscriminator {
                expected: hex::encode(ORDERBOOK_DISCRIMINATOR),
                actual: hex::encode(discriminator),
            });
        }

        Ok(Self {
            discriminator,
            market: read_pubkey(data, 8),
            mint_a: read_pubkey(data, 40),
            mint_b: read_pubkey(data, 72),
            lookup_table: read_pubkey(data, 104),
            bump: data[136],
        })
    }

    /// Check if account data has the orderbook discriminator
    pub fn is_orderbook_account(data: &[u8]) -> bool {
        data.len() >= 8 && data[0..8] == ORDERBOOK_DISCRIMINATOR
    }
}

// ============================================================================
// GlobalDepositToken Account (48 bytes)
// ============================================================================

/// GlobalDepositToken account - whitelist entry for global deposits
///
/// Layout:
/// - [0..8]   discriminator (8 bytes)
/// - [8..40]  mint (32 bytes)
/// - [40]     active (1 byte)
/// - [41]     bump (1 byte)
/// - [42..48] _padding (6 bytes)
#[derive(Debug, Clone)]
pub struct GlobalDepositToken {
    /// Account discriminator
    pub discriminator: [u8; 8],
    /// The whitelisted token mint
    pub mint: Pubkey,
    /// Whether this deposit token is currently active
    pub active: bool,
    /// PDA bump seed
    pub bump: u8,
}

impl GlobalDepositToken {
    /// Account size in bytes
    pub const LEN: usize = GLOBAL_DEPOSIT_TOKEN_SIZE;

    /// Deserialize from account data
    pub fn deserialize(data: &[u8]) -> SdkResult<Self> {
        if data.len() < Self::LEN {
            return Err(SdkError::InvalidDataLength {
                expected: Self::LEN,
                actual: data.len(),
            });
        }

        let discriminator = read_bytes::<8>(data, 0);
        if discriminator != GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR {
            return Err(SdkError::InvalidDiscriminator {
                expected: hex::encode(GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR),
                actual: hex::encode(discriminator),
            });
        }

        Ok(Self {
            discriminator,
            mint: read_pubkey(data, 8),
            active: data[40] != 0,
            bump: data[41],
        })
    }

    /// Check if account data has the global deposit token discriminator
    pub fn is_global_deposit_token_account(data: &[u8]) -> bool {
        data.len() >= 8 && data[0..8] == GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_deserialization() {
        let mut data = vec![0u8; EXCHANGE_SIZE];
        data[0..8].copy_from_slice(&EXCHANGE_DISCRIMINATOR);
        // authority at offset 8
        data[8..40].copy_from_slice(&[1u8; 32]);
        // operator at offset 40
        data[40..72].copy_from_slice(&[2u8; 32]);
        // market_count at offset 72
        data[72..80].copy_from_slice(&5u64.to_le_bytes());
        // paused at offset 80
        data[80] = 0;
        // bump at offset 81
        data[81] = 255;

        let exchange = Exchange::deserialize(&data).unwrap();
        assert_eq!(exchange.market_count, 5);
        assert!(!exchange.paused);
        assert_eq!(exchange.bump, 255);
    }

    #[test]
    fn test_market_deserialization() {
        let mut data = vec![0u8; MARKET_SIZE];
        data[0..8].copy_from_slice(&MARKET_DISCRIMINATOR);
        // market_id at offset 8
        data[8..16].copy_from_slice(&42u64.to_le_bytes());
        // num_outcomes at offset 16
        data[16] = 3;
        // status at offset 17
        data[17] = 1; // Active
        // winning_outcome at offset 18
        data[18] = 255;
        // has_winning_outcome at offset 19
        data[19] = 0;
        // bump at offset 20
        data[20] = 254;

        let market = Market::deserialize(&data).unwrap();
        assert_eq!(market.market_id, 42);
        assert_eq!(market.num_outcomes, 3);
        assert_eq!(market.status, MarketStatus::Active);
        assert_eq!(market.winning_outcome, 255);
        assert!(!market.has_winning_outcome);
    }

    #[test]
    fn test_position_deserialization() {
        let mut data = vec![0u8; POSITION_SIZE];
        data[0..8].copy_from_slice(&POSITION_DISCRIMINATOR);
        // owner at offset 8
        data[8..40].copy_from_slice(&[1u8; 32]);
        // market at offset 40
        data[40..72].copy_from_slice(&[2u8; 32]);
        // bump at offset 72
        data[72] = 253;

        let position = Position::deserialize(&data).unwrap();
        assert_eq!(position.bump, 253);
    }

    #[test]
    fn test_order_status_deserialization() {
        let mut data = vec![0u8; ORDER_STATUS_SIZE];
        data[0..8].copy_from_slice(&ORDER_STATUS_DISCRIMINATOR);
        // remaining at offset 8
        data[8..16].copy_from_slice(&1000u64.to_le_bytes());
        // is_cancelled at offset 16
        data[16] = 0;

        let order_status = OrderStatus::deserialize(&data).unwrap();
        assert_eq!(order_status.remaining, 1000);
        assert!(!order_status.is_cancelled);
    }

    #[test]
    fn test_user_nonce_deserialization() {
        let mut data = vec![0u8; USER_NONCE_SIZE];
        data[0..8].copy_from_slice(&USER_NONCE_DISCRIMINATOR);
        // nonce at offset 8
        data[8..16].copy_from_slice(&99u64.to_le_bytes());

        let user_nonce = UserNonce::deserialize(&data).unwrap();
        assert_eq!(user_nonce.nonce, 99);
    }

    #[test]
    fn test_orderbook_deserialization() {
        let mut data = vec![0u8; ORDERBOOK_SIZE];
        data[0..8].copy_from_slice(&ORDERBOOK_DISCRIMINATOR);
        // market at offset 8
        data[8..40].copy_from_slice(&[1u8; 32]);
        // mint_a at offset 40
        data[40..72].copy_from_slice(&[2u8; 32]);
        // mint_b at offset 72
        data[72..104].copy_from_slice(&[3u8; 32]);
        // lookup_table at offset 104
        data[104..136].copy_from_slice(&[4u8; 32]);
        // bump at offset 136
        data[136] = 252;

        let orderbook = Orderbook::deserialize(&data).unwrap();
        assert_eq!(orderbook.market, Pubkey::new_from_array([1u8; 32]));
        assert_eq!(orderbook.mint_a, Pubkey::new_from_array([2u8; 32]));
        assert_eq!(orderbook.mint_b, Pubkey::new_from_array([3u8; 32]));
        assert_eq!(orderbook.lookup_table, Pubkey::new_from_array([4u8; 32]));
        assert_eq!(orderbook.bump, 252);
    }

    #[test]
    fn test_orderbook_is_orderbook_account() {
        let mut data = vec![0u8; ORDERBOOK_SIZE];
        data[0..8].copy_from_slice(&ORDERBOOK_DISCRIMINATOR);
        assert!(Orderbook::is_orderbook_account(&data));

        let bad_data = vec![0u8; ORDERBOOK_SIZE];
        assert!(!Orderbook::is_orderbook_account(&bad_data));
    }

    #[test]
    fn test_global_deposit_token_deserialization() {
        let mut data = vec![0u8; GLOBAL_DEPOSIT_TOKEN_SIZE];
        data[0..8].copy_from_slice(&GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR);
        data[8..40].copy_from_slice(&[5u8; 32]);
        data[40] = 1;
        data[41] = 251;

        let gdt = GlobalDepositToken::deserialize(&data).unwrap();
        assert_eq!(gdt.mint, Pubkey::new_from_array([5u8; 32]));
        assert!(gdt.active);
        assert_eq!(gdt.bump, 251);
    }

    #[test]
    fn test_global_deposit_token_inactive() {
        let mut data = vec![0u8; GLOBAL_DEPOSIT_TOKEN_SIZE];
        data[0..8].copy_from_slice(&GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR);
        data[40] = 0;

        let gdt = GlobalDepositToken::deserialize(&data).unwrap();
        assert!(!gdt.active);
    }

    #[test]
    fn test_global_deposit_token_is_account() {
        let mut data = vec![0u8; GLOBAL_DEPOSIT_TOKEN_SIZE];
        data[0..8].copy_from_slice(&GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR);
        assert!(GlobalDepositToken::is_global_deposit_token_account(&data));

        let bad_data = vec![0u8; GLOBAL_DEPOSIT_TOKEN_SIZE];
        assert!(!GlobalDepositToken::is_global_deposit_token_account(&bad_data));
    }
}
