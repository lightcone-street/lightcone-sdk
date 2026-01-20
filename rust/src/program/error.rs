//! Error types for the Lightcone on-chain program module.

use thiserror::Error;

/// SDK-specific errors
#[derive(Debug, Error)]
pub enum SdkError {
    /// RPC client error
    #[error("RPC error: {0}")]
    Rpc(#[from] solana_client::client_error::ClientError),

    /// Invalid account discriminator
    #[error("Invalid account discriminator: expected {expected}, got {actual}")]
    InvalidDiscriminator {
        expected: String,
        actual: String,
    },

    /// Account not found
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    /// Invalid data length
    #[error("Invalid data length: expected {expected}, got {actual}")]
    InvalidDataLength {
        expected: usize,
        actual: usize,
    },

    /// Invalid order hash
    #[error("Invalid order hash")]
    InvalidOrderHash,

    /// Invalid signature
    #[error("Invalid signature")]
    InvalidSignature,

    /// Order expired
    #[error("Order expired")]
    OrderExpired,

    /// Invalid outcome count
    #[error("Invalid outcome count: {0} (must be {}-{})", crate::program::constants::MIN_OUTCOMES, crate::program::constants::MAX_OUTCOMES)]
    InvalidOutcomeCount(u8),

    /// Invalid outcome index
    #[error("Invalid outcome index: {index} (max {max})")]
    InvalidOutcomeIndex {
        index: u8,
        max: u8,
    },

    /// Too many makers
    #[error("Too many makers: {0} (max {})", crate::program::constants::MAX_MAKERS)]
    TooManyMakers(usize),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid side value
    #[error("Invalid side value: {0} (must be 0 or 1)")]
    InvalidSide(u8),

    /// Invalid market status
    #[error("Invalid market status: {0}")]
    InvalidMarketStatus(u8),

    /// Signature verification failed
    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Orders do not cross
    #[error("Orders do not cross (prices incompatible)")]
    OrdersDoNotCross,

    /// Fill amount exceeds remaining
    #[error("Fill amount {fill} exceeds remaining {remaining}")]
    FillAmountExceedsRemaining {
        fill: u64,
        remaining: u64,
    },

    /// Arithmetic overflow
    #[error("Arithmetic overflow")]
    Overflow,

    /// Invalid pubkey
    #[error("Invalid pubkey: {0}")]
    InvalidPubkey(String),
}

/// Result type alias for SDK operations
pub type SdkResult<T> = Result<T, SdkError>;
