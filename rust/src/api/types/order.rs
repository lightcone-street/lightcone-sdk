//! Order-related types for the Lightcone REST API.

use serde::{Deserialize, Serialize};

/// Order side enum (serializes as integer: 0=Bid, 1=Ask).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
#[repr(u32)]
pub enum ApiOrderSide {
    /// Buy base token with quote token
    Bid = 0,
    /// Sell base token for quote token
    Ask = 1,
}

/// Error returned when trying to convert an invalid value to ApiOrderSide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidOrderSideError(pub u32);

impl std::fmt::Display for InvalidOrderSideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid order side value: {} (expected 0 for Bid or 1 for Ask)", self.0)
    }
}

impl std::error::Error for InvalidOrderSideError {}

impl TryFrom<u32> for ApiOrderSide {
    type Error = InvalidOrderSideError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Bid),
            1 => Ok(Self::Ask),
            _ => Err(InvalidOrderSideError(value)),
        }
    }
}

impl From<ApiOrderSide> for u32 {
    fn from(side: ApiOrderSide) -> Self {
        side as u32
    }
}

/// Order status enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    /// Order placed on book
    Accepted,
    /// Partially filled, remainder on book
    PartialFill,
    /// Completely filled
    Filled,
    /// Order rejected
    Rejected,
}

/// Fill information from order matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    /// Counterparty address
    pub counterparty: String,
    /// Counterparty's order hash
    pub counterparty_order_hash: String,
    /// Amount filled as decimal string
    pub fill_amount: String,
    /// Fill price as decimal string
    pub price: String,
    /// Whether this order was the maker
    pub is_maker: bool,
}

/// Request for POST /api/orders/submit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitOrderRequest {
    /// Order creator's pubkey (Base58)
    pub maker: String,
    /// User's nonce for uniqueness
    pub nonce: u64,
    /// Market address (Base58)
    pub market_pubkey: String,
    /// Token being bought/sold (Base58)
    pub base_token: String,
    /// Token used for payment (Base58)
    pub quote_token: String,
    /// Order side (0=BID, 1=ASK)
    pub side: u32,
    /// Amount maker gives
    pub maker_amount: u64,
    /// Amount maker wants to receive
    pub taker_amount: u64,
    /// Unix timestamp, 0=no expiration
    #[serde(default)]
    pub expiration: i64,
    /// Ed25519 signature (hex, 128 chars)
    pub signature: String,
    /// Target orderbook
    pub orderbook_id: String,
}

/// Response for POST /api/orders/submit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    /// Order hash (hex)
    pub order_hash: String,
    /// Order status
    pub status: OrderStatus,
    /// Remaining amount as decimal string
    pub remaining: String,
    /// Filled amount as decimal string
    pub filled: String,
    /// Fill details
    #[serde(default)]
    pub fills: Vec<Fill>,
}

/// Request for POST /api/orders/cancel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderRequest {
    /// Hash of order to cancel (hex)
    pub order_hash: String,
    /// Must match order creator (Base58)
    pub maker: String,
}

/// Response for POST /api/orders/cancel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelResponse {
    /// Cancellation status
    pub status: String,
    /// Order hash
    pub order_hash: String,
    /// Remaining amount that was cancelled as decimal string
    pub remaining: String,
}

/// Request for POST /api/orders/cancel-all.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelAllOrdersRequest {
    /// User's public key (Base58)
    pub user_pubkey: String,
    /// Limit to specific market (empty = all)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_pubkey: Option<String>,
}

/// Response for POST /api/orders/cancel-all.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelAllResponse {
    /// Status (success)
    pub status: String,
    /// User pubkey
    pub user_pubkey: String,
    /// Market pubkey if specified
    #[serde(default)]
    pub market_pubkey: Option<String>,
    /// List of cancelled order hashes
    pub cancelled_order_hashes: Vec<String>,
    /// Count of cancelled orders
    pub count: u64,
    /// Human-readable message
    pub message: String,
}

/// User order from GET /api/users/orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOrder {
    /// Order hash
    pub order_hash: String,
    /// Market pubkey
    pub market_pubkey: String,
    /// Orderbook ID
    pub orderbook_id: String,
    /// Order side
    pub side: ApiOrderSide,
    /// Maker amount as decimal string
    pub maker_amount: String,
    /// Taker amount as decimal string
    pub taker_amount: String,
    /// Remaining amount as decimal string
    pub remaining: String,
    /// Filled amount as decimal string
    pub filled: String,
    /// Order price as decimal string
    pub price: String,
    /// Creation timestamp
    pub created_at: String,
    /// Expiration timestamp
    pub expiration: i64,
}

/// Request for POST /api/users/orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserOrdersRequest {
    /// User's public key (Base58)
    pub user_pubkey: String,
}

/// Outcome balance in user orders response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOrderOutcomeBalance {
    /// Outcome index
    pub outcome_index: u32,
    /// Conditional token address
    pub conditional_token: String,
    /// Idle balance as decimal string
    pub idle: String,
    /// Balance on order book as decimal string
    pub on_book: String,
}

/// User balance from GET /api/users/orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalance {
    /// Market pubkey
    pub market_pubkey: String,
    /// Deposit asset
    pub deposit_asset: String,
    /// Outcome balances
    pub outcomes: Vec<UserOrderOutcomeBalance>,
}

/// Response for POST /api/users/orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOrdersResponse {
    /// User pubkey
    pub user_pubkey: String,
    /// Open orders
    pub orders: Vec<UserOrder>,
    /// User balances
    pub balances: Vec<UserBalance>,
}
