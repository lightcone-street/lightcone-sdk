//! Error types for the Lightcone on-chain program module.

use thiserror::Error;

#[cfg(feature = "client")]
use solana_client::client_error::ClientError;

/// SDK-specific errors
#[derive(Debug, Error)]
pub enum SdkError {
    /// RPC client error
    #[cfg(feature = "client")]
    #[error("RPC error: {0}")]
    Rpc(#[from] ClientError),

    /// Invalid account discriminator
    #[error("Invalid account discriminator: expected {expected}, got {actual}")]
    InvalidDiscriminator { expected: String, actual: String },

    /// Account not found
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    /// Invalid data length
    #[error("Invalid data length: expected {expected}, got {actual}")]
    InvalidDataLength { expected: usize, actual: usize },

    /// Invalid outcome count
    #[error("Invalid outcome count: {count} (must be {min}-{max})", min = crate::program::constants::MIN_OUTCOMES, max = crate::program::constants::MAX_OUTCOMES)]
    InvalidOutcomeCount { count: u8 },

    /// Invalid outcome index
    #[error("Invalid outcome index: {index} (max {max})")]
    InvalidOutcomeIndex { index: u8, max: u8 },

    /// Too many makers
    #[error("Too many makers: {count} (max {max})", max = crate::program::constants::MAX_MAKERS)]
    TooManyMakers { count: usize },

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Invalid signature")]
    InvalidSignature,

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid side value
    #[error("Invalid side value: {0} (must be 0 or 1)")]
    InvalidSide(u8),

    /// Invalid market status
    #[error("Invalid market status: {0}")]
    InvalidMarketStatus(u8),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Arithmetic overflow
    #[error("Arithmetic overflow")]
    Overflow,

    /// Invalid mint order
    #[error("Invalid mint order")]
    InvalidMintOrder,

    /// Orderbook already exists
    #[error("Orderbook already exists")]
    OrderbookExists,

    /// Invalid market
    #[error("Invalid market")]
    InvalidMarket,

    /// Market already settled
    #[error("Market already settled")]
    MarketSettled,

    /// Invalid program ID
    #[error("Invalid program ID")]
    InvalidProgramId,

    /// Invalid orderbook
    #[error("Invalid orderbook")]
    InvalidOrderbook,

    /// Full fill required
    #[error("Full fill required")]
    FullFillRequired,

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Invalid pubkey
    #[error("Invalid pubkey: {0}")]
    InvalidPubkey(String),

    /// Scaling error (price/size conversion)
    #[error("Scaling error: {0}")]
    Scaling(#[from] crate::shared::scaling::ScalingError),
}

/// Result type alias for SDK operations
pub type SdkResult<T> = Result<T, SdkError>;
