//! Order types, serialization, hashing, and signing.
//!
//! This module provides the signed and compact order structures with
//! Keccak256 hashing and Ed25519 signing functionality.

use sha3::{Digest, Keccak256};
use solana_pubkey::Pubkey;
use solana_signature::Signature;

#[cfg(feature = "client")]
use solana_keypair::Keypair;
#[cfg(feature = "client")]
use solana_signer::Signer;

use crate::program::constants::{ORDER_SIZE, SIGNED_ORDER_SIZE};
use crate::program::error::{SdkError, SdkResult};
use crate::program::types::{AskOrderParams, BidOrderParams, OrderSide};
use crate::shared::{CancelAllOrdersRequest, CancelOrderRequest, SubmitOrderRequest};

// ============================================================================
// Signed Order (225 bytes)
// ============================================================================

/// Signed order structure with full context and signature.
///
/// Layout (225 bytes):
/// - [0..8]     nonce (u32 value, serialized as u64 LE for wire compatibility)
/// - [8..40]    maker (32 bytes)
/// - [40..72]   market (32 bytes)
/// - [72..104]  base_mint (32 bytes)
/// - [104..136] quote_mint (32 bytes)
/// - [136]      side (1 byte)
/// - [137..145] maker_amount (8 bytes)
/// - [145..153] taker_amount (8 bytes)
/// - [153..161] expiration (8 bytes)
/// - [161..225] signature (64 bytes)
#[derive(Debug, Clone)]
pub struct SignedOrder {
    /// Unique order ID and replay protection (u32 range, serialized as u64 on wire)
    pub nonce: u32,
    /// Order maker's pubkey
    pub maker: Pubkey,
    /// Market pubkey
    pub market: Pubkey,
    /// Base mint (token being bought/sold)
    pub base_mint: Pubkey,
    /// Quote mint (token used for payment)
    pub quote_mint: Pubkey,
    /// Order side (0 = Bid, 1 = Ask)
    pub side: OrderSide,
    /// Amount maker gives
    pub maker_amount: u64,
    /// Amount maker receives
    pub taker_amount: u64,
    /// Expiration timestamp (0 = no expiration)
    pub expiration: i64,
    /// Ed25519 signature
    pub signature: [u8; 64],
}

impl SignedOrder {
    /// Order size in bytes
    pub const LEN: usize = SIGNED_ORDER_SIZE;

    /// Size of the signed portion of the order (for hashing)
    pub const HASH_SIZE: usize = 161;

    /// Create a new bid order (maker buys base, gives quote)
    pub fn new_bid(params: BidOrderParams) -> Self {
        Self {
            nonce: params.nonce,
            maker: params.maker,
            market: params.market,
            base_mint: params.base_mint,
            quote_mint: params.quote_mint,
            side: OrderSide::Bid,
            maker_amount: params.maker_amount,
            taker_amount: params.taker_amount,
            expiration: params.expiration,
            signature: [0u8; 64],
        }
    }

    /// Create a new ask order (maker sells base, receives quote)
    pub fn new_ask(params: AskOrderParams) -> Self {
        Self {
            nonce: params.nonce,
            maker: params.maker,
            market: params.market,
            base_mint: params.base_mint,
            quote_mint: params.quote_mint,
            side: OrderSide::Ask,
            maker_amount: params.maker_amount,
            taker_amount: params.taker_amount,
            expiration: params.expiration,
            signature: [0u8; 64],
        }
    }

    /// Build the raw 161-byte order message from the signed fields.
    /// This is hashed (keccak256) and hex-encoded to produce the bytes that users sign.
    fn signing_message(&self) -> [u8; Self::HASH_SIZE] {
        let mut data = [0u8; Self::HASH_SIZE];

        data[0..8].copy_from_slice(&(self.nonce as u64).to_le_bytes());
        data[8..40].copy_from_slice(self.maker.as_ref());
        data[40..72].copy_from_slice(self.market.as_ref());
        data[72..104].copy_from_slice(self.base_mint.as_ref());
        data[104..136].copy_from_slice(self.quote_mint.as_ref());
        data[136] = self.side as u8;
        data[137..145].copy_from_slice(&self.maker_amount.to_le_bytes());
        data[145..153].copy_from_slice(&self.taker_amount.to_le_bytes());
        data[153..161].copy_from_slice(&self.expiration.to_le_bytes());

        data
    }

    /// Compute the 32-byte Keccak256 hash of the signed fields.
    pub fn hash(&self) -> [u8; 32] {
        Keccak256::digest(self.signing_message()).into()
    }

    /// Compute the order hash as a hex string.
    pub fn hash_hex(&self) -> String {
        hex::encode(self.hash())
    }

    /// Sign the order with the given keypair.
    #[cfg(feature = "client")]
    pub fn sign(&mut self, keypair: &Keypair) {
        let hash = self.hash_hex();
        let sig = keypair.sign_message(hash.as_bytes());

        self.signature.copy_from_slice(sig.as_ref());
    }

    /// Create and sign an order in one step.
    #[cfg(feature = "client")]
    pub fn new_bid_signed(params: BidOrderParams, keypair: &Keypair) -> Self {
        let mut order = Self::new_bid(params);
        order.sign(keypair);
        order
    }

    /// Create and sign an ask order in one step.
    #[cfg(feature = "client")]
    pub fn new_ask_signed(params: AskOrderParams, keypair: &Keypair) -> Self {
        let mut order = Self::new_ask(params);
        order.sign(keypair);
        order
    }

    /// Verify the Ed25519 signature over hex(keccak256(order_message)).
    /// The signed payload is a 64-char ASCII hex string (UTF-8 safe for wallet compatibility).
    pub fn verify_signature(&self) -> SdkResult<()> {
        let hash_hex = self.hash_hex();
        let sig = Signature::try_from(self.signature.as_slice())
            .map_err(|_| SdkError::InvalidSignature)?;

        if !sig.verify(self.maker.as_ref(), hash_hex.as_bytes()) {
            return Err(SdkError::SignatureVerificationFailed);
        }
        Ok(())
    }

    /// Apply a signature to the order.
    pub fn apply_signature(&mut self, sig_bs58: String) -> SdkResult<()> {
        let signature = sig_bs58
            .parse::<Signature>()
            .map_err(|_| SdkError::InvalidSignature)?;

        self.signature = signature.into();
        Ok(())
    }

    /// Serialize to bytes (225 bytes).
    pub fn serialize(&self) -> [u8; SIGNED_ORDER_SIZE] {
        let mut data = [0u8; SIGNED_ORDER_SIZE];

        data[0..8].copy_from_slice(&(self.nonce as u64).to_le_bytes());
        data[8..40].copy_from_slice(self.maker.as_ref());
        data[40..72].copy_from_slice(self.market.as_ref());
        data[72..104].copy_from_slice(self.base_mint.as_ref());
        data[104..136].copy_from_slice(self.quote_mint.as_ref());
        data[136] = self.side as u8;
        data[137..145].copy_from_slice(&self.maker_amount.to_le_bytes());
        data[145..153].copy_from_slice(&self.taker_amount.to_le_bytes());
        data[153..161].copy_from_slice(&self.expiration.to_le_bytes());
        data[161..225].copy_from_slice(&self.signature);

        data
    }

    /// Deserialize from bytes.
    pub fn deserialize(data: &[u8]) -> SdkResult<Self> {
        if data.len() < SIGNED_ORDER_SIZE {
            return Err(SdkError::InvalidDataLength {
                expected: SIGNED_ORDER_SIZE,
                actual: data.len(),
            });
        }

        let mut nonce_bytes = [0u8; 8];
        nonce_bytes.copy_from_slice(&data[0..8]);
        let nonce_u64 = u64::from_le_bytes(nonce_bytes);
        if nonce_u64 > u32::MAX as u64 {
            return Err(SdkError::Overflow);
        }

        let mut maker_bytes = [0u8; 32];
        maker_bytes.copy_from_slice(&data[8..40]);

        let mut market_bytes = [0u8; 32];
        market_bytes.copy_from_slice(&data[40..72]);

        let mut base_mint_bytes = [0u8; 32];
        base_mint_bytes.copy_from_slice(&data[72..104]);

        let mut quote_mint_bytes = [0u8; 32];
        quote_mint_bytes.copy_from_slice(&data[104..136]);

        let mut maker_amount_bytes = [0u8; 8];
        maker_amount_bytes.copy_from_slice(&data[137..145]);

        let mut taker_amount_bytes = [0u8; 8];
        taker_amount_bytes.copy_from_slice(&data[145..153]);

        let mut expiration_bytes = [0u8; 8];
        expiration_bytes.copy_from_slice(&data[153..161]);

        let mut signature = [0u8; 64];
        signature.copy_from_slice(&data[161..225]);

        Ok(Self {
            nonce: nonce_u64 as u32,
            maker: Pubkey::new_from_array(maker_bytes),
            market: Pubkey::new_from_array(market_bytes),
            base_mint: Pubkey::new_from_array(base_mint_bytes),
            quote_mint: Pubkey::new_from_array(quote_mint_bytes),
            side: OrderSide::try_from(data[136])?,
            maker_amount: u64::from_le_bytes(maker_amount_bytes),
            taker_amount: u64::from_le_bytes(taker_amount_bytes),
            expiration: i64::from_le_bytes(expiration_bytes),
            signature,
        })
    }

    /// Convert to compact order format (29 bytes, no maker field).
    pub fn to_order(&self) -> Order {
        Order {
            nonce: self.nonce,
            side: self.side,
            maker_amount: self.maker_amount,
            taker_amount: self.taker_amount,
            expiration: self.expiration,
        }
    }

    /// Get the signature as a hex string (128 chars).
    pub fn signature_hex(&self) -> String {
        hex::encode(self.signature)
    }

    /// Check if the order has been signed.
    pub fn is_signed(&self) -> bool {
        self.signature != [0u8; 64]
    }

    // =========================================================================
    // API Bridge Methods
    // =========================================================================

    /// Convert a signed order to an API SubmitOrderRequest.
    ///
    /// This bridges on-chain order creation with REST API submission.
    ///
    /// # Arguments
    ///
    /// * `orderbook_id` - Target orderbook (get from market API or use `derive_orderbook_id()`)
    ///
    /// # Panics
    ///
    /// Panics if the order has not been signed (signature is all zeros).
    pub fn to_submit_request(&self, orderbook_id: impl Into<String>) -> SubmitOrderRequest {
        assert!(
            self.signature != [0u8; 64],
            "Order must be signed before converting to submit request"
        );

        SubmitOrderRequest {
            maker: self.maker.to_string(),
            nonce: self.nonce,
            market_pubkey: self.market.to_string(),
            base_token: self.base_mint.to_string(),
            quote_token: self.quote_mint.to_string(),
            side: self.side as u32,
            maker_amount: self.maker_amount,
            taker_amount: self.taker_amount,
            expiration: self.expiration,
            signature: hex::encode(self.signature),
            orderbook_id: orderbook_id.into(),
        }
    }

    /// Derive the orderbook ID for this order.
    ///
    /// Format: `{base_token[0:8]}_{quote_token[0:8]}`
    pub fn derive_orderbook_id(&self) -> String {
        crate::shared::derive_orderbook_id(
            &self.base_mint.to_string(),
            &self.quote_mint.to_string(),
        )
    }
}

// ============================================================================
// Order (29 bytes)
// ============================================================================

/// Compact order format for on-chain transaction data.
///
/// No `maker` field (derived from Position PDA on-chain).
///
/// Layout (29 bytes):
/// - [0..4]   nonce (4 bytes, u32)
/// - [4]      side (1 byte)
/// - [5..13]  maker_amount (8 bytes)
/// - [13..21] taker_amount (8 bytes)
/// - [21..29] expiration (8 bytes)
#[derive(Debug, Clone)]
pub struct Order {
    /// Unique order ID and replay protection (truncated to u32)
    pub nonce: u32,
    /// Order side (0 = Bid, 1 = Ask)
    pub side: OrderSide,
    /// Amount maker gives
    pub maker_amount: u64,
    /// Amount maker receives
    pub taker_amount: u64,
    /// Expiration timestamp (0 = no expiration)
    pub expiration: i64,
}

impl Order {
    /// Order size in bytes
    pub const LEN: usize = ORDER_SIZE;

    /// Serialize to bytes (29 bytes).
    pub fn serialize(&self) -> [u8; ORDER_SIZE] {
        let mut data = [0u8; ORDER_SIZE];

        data[0..4].copy_from_slice(&self.nonce.to_le_bytes());
        data[4] = self.side as u8;
        data[5..13].copy_from_slice(&self.maker_amount.to_le_bytes());
        data[13..21].copy_from_slice(&self.taker_amount.to_le_bytes());
        data[21..29].copy_from_slice(&self.expiration.to_le_bytes());

        data
    }

    /// Deserialize from bytes.
    pub fn deserialize(data: &[u8]) -> SdkResult<Self> {
        if data.len() < ORDER_SIZE {
            return Err(SdkError::InvalidDataLength {
                expected: ORDER_SIZE,
                actual: data.len(),
            });
        }

        let mut nonce_bytes = [0u8; 4];
        nonce_bytes.copy_from_slice(&data[0..4]);

        let mut maker_amount_bytes = [0u8; 8];
        maker_amount_bytes.copy_from_slice(&data[5..13]);

        let mut taker_amount_bytes = [0u8; 8];
        taker_amount_bytes.copy_from_slice(&data[13..21]);

        let mut expiration_bytes = [0u8; 8];
        expiration_bytes.copy_from_slice(&data[21..29]);

        Ok(Self {
            nonce: u32::from_le_bytes(nonce_bytes),
            side: OrderSide::try_from(data[4])?,
            maker_amount: u64::from_le_bytes(maker_amount_bytes),
            taker_amount: u64::from_le_bytes(taker_amount_bytes),
            expiration: i64::from_le_bytes(expiration_bytes),
        })
    }

    /// Expand to signed order using pubkeys from accounts.
    pub fn to_signed(
        &self,
        maker: Pubkey,
        market: Pubkey,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        signature: [u8; 64],
    ) -> SignedOrder {
        SignedOrder {
            nonce: self.nonce,
            maker,
            market,
            base_mint,
            quote_mint,
            side: self.side,
            maker_amount: self.maker_amount,
            taker_amount: self.taker_amount,
            expiration: self.expiration,
            signature,
        }
    }
}

// ============================================================================
// Order Validation Helpers
// ============================================================================

/// Check if an order is expired.
pub fn is_order_expired(order: &SignedOrder, current_time: i64) -> bool {
    order.expiration != 0 && current_time >= order.expiration
}

/// Check if two orders can cross (prices are compatible).
///
/// Returns true if the buyer's price >= seller's price.
pub fn orders_can_cross(buy_order: &SignedOrder, sell_order: &SignedOrder) -> bool {
    if buy_order.side != OrderSide::Bid || sell_order.side != OrderSide::Ask {
        return false;
    }

    if buy_order.maker_amount == 0
        || buy_order.taker_amount == 0
        || sell_order.maker_amount == 0
        || sell_order.taker_amount == 0
    {
        return false;
    }

    // Buyer gives quote, receives base
    // Seller gives base, receives quote
    // Cross condition: buyer's price >= seller's price
    // buyer_price = buyer.maker_amount / buyer.taker_amount (quote per base)
    // seller_price = seller.taker_amount / seller.maker_amount (quote per base)
    // Cross: buyer.maker_amount / buyer.taker_amount >= seller.taker_amount / seller.maker_amount
    // Rearrange: buyer.maker_amount * seller.maker_amount >= buyer.taker_amount * seller.taker_amount

    let buyer_cross = (buy_order.maker_amount as u128) * (sell_order.maker_amount as u128);
    let seller_cross = (buy_order.taker_amount as u128) * (sell_order.taker_amount as u128);

    buyer_cross >= seller_cross
}

/// Calculate the taker fill amount given a maker fill amount.
pub fn calculate_taker_fill(maker_order: &SignedOrder, maker_fill_amount: u64) -> SdkResult<u64> {
    if maker_order.maker_amount == 0 {
        return Err(SdkError::Overflow);
    }

    let result = (maker_fill_amount as u128)
        .checked_mul(maker_order.taker_amount as u128)
        .ok_or(SdkError::Overflow)?
        .checked_div(maker_order.maker_amount as u128)
        .ok_or(SdkError::Overflow)?;

    if result > u64::MAX as u128 {
        return Err(SdkError::Overflow);
    }

    Ok(result as u64)
}

/// Derive condition ID from oracle, question_id, and num_outcomes.
pub fn derive_condition_id(oracle: &Pubkey, question_id: &[u8; 32], num_outcomes: u8) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(oracle.as_ref());
    hasher.update(question_id);
    hasher.update([num_outcomes]);
    hasher.finalize().into()
}

// ============================================================================
// Signed Cancel Order
// ============================================================================

/// Signed cancel-order request.
///
/// Mirrors the [`SignedOrder`] pattern: construct, sign (or apply external
/// signature), then convert to an API request via [`to_cancel_request`].
#[derive(Debug, Clone)]
pub struct SignedCancelOrder {
    /// Hex-encoded order hash (64-char hex string)
    pub order_hash: String,
    /// Order creator's pubkey
    pub maker: Pubkey,
    /// Ed25519 signature (64 bytes, all zeros until signed)
    pub signature: [u8; 64],
}

impl SignedCancelOrder {
    /// Create a new unsigned cancel request.
    pub fn new(order_hash: impl Into<String>, maker: Pubkey) -> Self {
        Self {
            order_hash: order_hash.into(),
            maker,
            signature: [0u8; 64],
        }
    }

    /// The message bytes that get signed: the order hash hex as UTF-8 bytes.
    pub fn signing_message(&self) -> Vec<u8> {
        self.order_hash.as_bytes().to_vec()
    }

    /// Sign the cancel request with the given keypair.
    #[cfg(feature = "client")]
    pub fn sign(&mut self, keypair: &Keypair) {
        let message = self.signing_message();
        let sig = keypair.sign_message(&message);
        self.signature.copy_from_slice(sig.as_ref());
    }

    /// Create and sign in one step.
    #[cfg(feature = "client")]
    pub fn new_signed(order_hash: impl Into<String>, maker: Pubkey, keypair: &Keypair) -> Self {
        let mut cancel = Self::new(order_hash, maker);
        cancel.sign(keypair);
        cancel
    }

    /// Apply an externally-produced signature (Base58-encoded).
    pub fn apply_signature(&mut self, sig_bs58: &str) -> SdkResult<()> {
        let signature = sig_bs58
            .parse::<Signature>()
            .map_err(|_| SdkError::InvalidSignature)?;
        self.signature = signature.into();
        Ok(())
    }

    /// Verify the signature against the maker pubkey.
    pub fn verify_signature(&self) -> SdkResult<()> {
        let sig = Signature::try_from(self.signature.as_slice())
            .map_err(|_| SdkError::InvalidSignature)?;
        if !sig.verify(self.maker.as_ref(), &self.signing_message()) {
            return Err(SdkError::SignatureVerificationFailed);
        }
        Ok(())
    }

    /// Get the signature as a hex string (128 chars).
    pub fn signature_hex(&self) -> String {
        hex::encode(self.signature)
    }

    /// Check if the cancel request has been signed.
    pub fn is_signed(&self) -> bool {
        self.signature != [0u8; 64]
    }

    /// Convert to the API payload type.
    ///
    /// # Panics
    ///
    /// Panics if the cancel request has not been signed.
    pub fn to_cancel_request(&self) -> CancelOrderRequest {
        assert!(
            self.is_signed(),
            "Cancel request must be signed before converting to API request"
        );
        CancelOrderRequest {
            order_hash: self.order_hash.clone(),
            maker: self.maker.to_string(),
            signature: self.signature_hex(),
        }
    }
}

// ============================================================================
// Signed Cancel All
// ============================================================================

/// Signed cancel-all-orders request.
///
/// Mirrors the [`SignedOrder`] pattern: construct, sign (or apply external
/// signature), then convert to an API request via [`to_cancel_all_request`].
#[derive(Debug, Clone)]
pub struct SignedCancelAll {
    /// User's public key
    pub user_pubkey: Pubkey,
    /// Unix timestamp used in the signed message
    pub timestamp: i64,
    /// Optional: limit to a specific orderbook
    pub orderbook_id: Option<String>,
    /// Ed25519 signature (64 bytes, all zeros until signed)
    pub signature: [u8; 64],
}

impl SignedCancelAll {
    /// Create a new unsigned cancel-all request.
    pub fn new(user_pubkey: Pubkey, timestamp: i64) -> Self {
        Self {
            user_pubkey,
            timestamp,
            orderbook_id: None,
            signature: [0u8; 64],
        }
    }

    /// Create with a specific orderbook scope.
    pub fn new_for_orderbook(
        user_pubkey: Pubkey,
        timestamp: i64,
        orderbook_id: impl Into<String>,
    ) -> Self {
        Self {
            user_pubkey,
            timestamp,
            orderbook_id: Some(orderbook_id.into()),
            signature: [0u8; 64],
        }
    }

    /// Create with the current system timestamp.
    ///
    /// # Panics
    ///
    /// Panics if the system clock is before the Unix epoch.
    pub fn now(user_pubkey: Pubkey) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_secs() as i64;
        Self::new(user_pubkey, timestamp)
    }

    /// The signing message: `"cancel_all:{pubkey}:{timestamp}"` as UTF-8 bytes.
    pub fn signing_message(&self) -> Vec<u8> {
        format!("cancel_all:{}:{}", self.user_pubkey, self.timestamp)
            .into_bytes()
    }

    /// Sign the cancel-all request with the given keypair.
    #[cfg(feature = "client")]
    pub fn sign(&mut self, keypair: &Keypair) {
        let message = self.signing_message();
        let sig = keypair.sign_message(&message);
        self.signature.copy_from_slice(sig.as_ref());
    }

    /// Create and sign in one step.
    #[cfg(feature = "client")]
    pub fn new_signed(user_pubkey: Pubkey, timestamp: i64, keypair: &Keypair) -> Self {
        let mut cancel = Self::new(user_pubkey, timestamp);
        cancel.sign(keypair);
        cancel
    }

    /// Create with current timestamp and sign in one step.
    #[cfg(feature = "client")]
    pub fn now_signed(user_pubkey: Pubkey, keypair: &Keypair) -> Self {
        let mut cancel = Self::now(user_pubkey);
        cancel.sign(keypair);
        cancel
    }

    /// Apply an externally-produced signature (Base58-encoded).
    pub fn apply_signature(&mut self, sig_bs58: &str) -> SdkResult<()> {
        let signature = sig_bs58
            .parse::<Signature>()
            .map_err(|_| SdkError::InvalidSignature)?;
        self.signature = signature.into();
        Ok(())
    }

    /// Verify the signature against the user pubkey.
    pub fn verify_signature(&self) -> SdkResult<()> {
        let sig = Signature::try_from(self.signature.as_slice())
            .map_err(|_| SdkError::InvalidSignature)?;
        if !sig.verify(self.user_pubkey.as_ref(), &self.signing_message()) {
            return Err(SdkError::SignatureVerificationFailed);
        }
        Ok(())
    }

    /// Get the signature as a hex string (128 chars).
    pub fn signature_hex(&self) -> String {
        hex::encode(self.signature)
    }

    /// Check if the cancel request has been signed.
    pub fn is_signed(&self) -> bool {
        self.signature != [0u8; 64]
    }

    /// Convert to the API payload type.
    ///
    /// # Panics
    ///
    /// Panics if the cancel request has not been signed.
    pub fn to_cancel_all_request(&self) -> CancelAllOrdersRequest {
        assert!(
            self.is_signed(),
            "Cancel-all request must be signed before converting to API request"
        );
        CancelAllOrdersRequest {
            user_pubkey: self.user_pubkey.to_string(),
            orderbook_id: self.orderbook_id.clone(),
            signature: self.signature_hex(),
            timestamp: self.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signed_order_serialization_roundtrip() {
        let order = SignedOrder {
            nonce: 12345,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            maker_amount: 1000000,
            taker_amount: 500000,
            expiration: 1234567890,
            signature: [0u8; 64],
        };

        let serialized = order.serialize();
        let deserialized = SignedOrder::deserialize(&serialized).unwrap();

        assert_eq!(order.nonce, deserialized.nonce);
        assert_eq!(order.maker, deserialized.maker);
        assert_eq!(order.market, deserialized.market);
        assert_eq!(order.base_mint, deserialized.base_mint);
        assert_eq!(order.quote_mint, deserialized.quote_mint);
        assert_eq!(order.side, deserialized.side);
        assert_eq!(order.maker_amount, deserialized.maker_amount);
        assert_eq!(order.taker_amount, deserialized.taker_amount);
        assert_eq!(order.expiration, deserialized.expiration);
    }

    #[test]
    fn test_order_serialization_roundtrip() {
        let order = Order {
            nonce: 12345,
            side: OrderSide::Ask,
            maker_amount: 1000000,
            taker_amount: 500000,
            expiration: 1234567890,
        };

        let serialized = order.serialize();
        let deserialized = Order::deserialize(&serialized).unwrap();

        assert_eq!(order.nonce, deserialized.nonce);
        assert_eq!(order.side, deserialized.side);
        assert_eq!(order.maker_amount, deserialized.maker_amount);
        assert_eq!(order.taker_amount, deserialized.taker_amount);
        assert_eq!(order.expiration, deserialized.expiration);
    }

    #[test]
    fn test_order_size() {
        assert_eq!(ORDER_SIZE, 29);
        let order = Order {
            nonce: 1,
            side: OrderSide::Bid,
            maker_amount: 100,
            taker_amount: 50,
            expiration: 0,
        };
        assert_eq!(order.serialize().len(), 29);
    }

    #[test]
    fn test_order_hash_consistency() {
        let order = SignedOrder {
            nonce: 1,
            maker: Pubkey::new_from_array([1u8; 32]),
            market: Pubkey::new_from_array([2u8; 32]),
            base_mint: Pubkey::new_from_array([3u8; 32]),
            quote_mint: Pubkey::new_from_array([4u8; 32]),
            side: OrderSide::Bid,
            maker_amount: 100,
            taker_amount: 50,
            expiration: 0,
            signature: [0u8; 64],
        };

        let hash1 = order.hash();
        let hash2 = order.hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_signed_order_to_order_roundtrip() {
        let signed = SignedOrder {
            nonce: 42,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            maker_amount: 1000,
            taker_amount: 500,
            expiration: 12345,
            signature: [7u8; 64],
        };

        let order = signed.to_order();
        assert_eq!(order.nonce, 42);
        assert_eq!(order.side, OrderSide::Bid);
        assert_eq!(order.maker_amount, 1000);
        assert_eq!(order.taker_amount, 500);
        assert_eq!(order.expiration, 12345);

        let back = order.to_signed(
            signed.maker,
            signed.market,
            signed.base_mint,
            signed.quote_mint,
            signed.signature,
        );
        assert_eq!(back.nonce, 42);
        assert_eq!(back.maker, signed.maker);
        assert_eq!(back.maker_amount, 1000);
    }

    #[test]
    fn test_orders_can_cross() {
        let buy_order = SignedOrder {
            nonce: 1,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            maker_amount: 100, // 100 quote
            taker_amount: 50,  // for 50 base (price = 2 quote/base)
            expiration: 0,
            signature: [0u8; 64],
        };

        let sell_order = SignedOrder {
            nonce: 2,
            maker: Pubkey::new_unique(),
            market: buy_order.market,
            base_mint: buy_order.base_mint,
            quote_mint: buy_order.quote_mint,
            side: OrderSide::Ask,
            maker_amount: 50, // 50 base
            taker_amount: 90, // for 90 quote (price = 1.8 quote/base)
            expiration: 0,
            signature: [0u8; 64],
        };

        // Buyer pays 2 quote/base, seller wants 1.8 quote/base - should cross
        assert!(orders_can_cross(&buy_order, &sell_order));
    }

    #[test]
    fn test_orders_cannot_cross() {
        let buy_order = SignedOrder {
            nonce: 1,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            maker_amount: 50, // 50 quote
            taker_amount: 50, // for 50 base (price = 1 quote/base)
            expiration: 0,
            signature: [0u8; 64],
        };

        let sell_order = SignedOrder {
            nonce: 2,
            maker: Pubkey::new_unique(),
            market: buy_order.market,
            base_mint: buy_order.base_mint,
            quote_mint: buy_order.quote_mint,
            side: OrderSide::Ask,
            maker_amount: 50,  // 50 base
            taker_amount: 100, // for 100 quote (price = 2 quote/base)
            expiration: 0,
            signature: [0u8; 64],
        };

        // Buyer pays 1 quote/base, seller wants 2 quote/base - should not cross
        assert!(!orders_can_cross(&buy_order, &sell_order));
    }

    #[test]
    fn test_calculate_taker_fill() {
        let maker_order = SignedOrder {
            nonce: 1,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Ask,
            maker_amount: 100, // gives 100 base
            taker_amount: 200, // wants 200 quote
            expiration: 0,
            signature: [0u8; 64],
        };

        // If filling 50 maker_amount, taker should get 50 * 200 / 100 = 100
        let taker_fill = calculate_taker_fill(&maker_order, 50).unwrap();
        assert_eq!(taker_fill, 100);
    }

    #[test]
    #[cfg(feature = "client")]
    fn test_to_submit_request() {
        use solana_keypair::Keypair;
        use solana_signer::Signer;

        let keypair = Keypair::new();
        let maker = keypair.pubkey();
        let market = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let mut order = SignedOrder {
            nonce: 42,
            maker,
            market,
            base_mint,
            quote_mint,
            side: OrderSide::Bid,
            maker_amount: 1_000_000,
            taker_amount: 500_000,
            expiration: 1234567890,
            signature: [0u8; 64],
        };

        order.sign(&keypair);

        let request = order.to_submit_request("test_orderbook");

        assert_eq!(request.maker, maker.to_string());
        assert_eq!(request.nonce, 42);
        assert_eq!(request.market_pubkey, market.to_string());
        assert_eq!(request.base_token, base_mint.to_string());
        assert_eq!(request.quote_token, quote_mint.to_string());
        assert_eq!(request.side, 0); // Bid
        assert_eq!(request.maker_amount, 1_000_000);
        assert_eq!(request.taker_amount, 500_000);
        assert_eq!(request.expiration, 1234567890);
        assert_eq!(request.orderbook_id, "test_orderbook");
        assert_eq!(request.signature.len(), 128); // 64 bytes = 128 hex chars
    }

    #[test]
    fn test_derive_orderbook_id() {
        let order = SignedOrder {
            nonce: 1,
            maker: Pubkey::new_from_array([1u8; 32]),
            market: Pubkey::new_from_array([2u8; 32]),
            base_mint: Pubkey::new_from_array([3u8; 32]),
            quote_mint: Pubkey::new_from_array([4u8; 32]),
            side: OrderSide::Bid,
            maker_amount: 100,
            taker_amount: 50,
            expiration: 0,
            signature: [0u8; 64],
        };

        let orderbook_id = order.derive_orderbook_id();
        // The orderbook ID should be first 8 chars of each pubkey string
        let base_str = order.base_mint.to_string();
        let quote_str = order.quote_mint.to_string();
        let expected = format!("{}_{}", &base_str[..8], &quote_str[..8]);
        assert_eq!(orderbook_id, expected);
    }

    #[test]
    #[cfg(feature = "client")]
    fn test_is_signed() {
        use solana_keypair::Keypair;
        use solana_signer::Signer;

        let keypair = Keypair::new();
        let mut order = SignedOrder {
            nonce: 1,
            maker: keypair.pubkey(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            maker_amount: 100,
            taker_amount: 50,
            expiration: 0,
            signature: [0u8; 64],
        };

        assert!(!order.is_signed());

        order.sign(&keypair);

        assert!(order.is_signed());
    }

    #[test]
    #[cfg(feature = "client")]
    fn test_signature_and_hash_hex() {
        use solana_keypair::Keypair;
        use solana_signer::Signer;

        let keypair = Keypair::new();
        let mut order = SignedOrder {
            nonce: 1,
            maker: keypair.pubkey(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            maker_amount: 100,
            taker_amount: 50,
            expiration: 0,
            signature: [0u8; 64],
        };

        order.sign(&keypair);

        let sig_hex = order.signature_hex();
        let hash_hex = order.hash_hex();

        // Signature should be 128 hex chars (64 bytes)
        assert_eq!(sig_hex.len(), 128);
        // Hash should be 64 hex chars (32 bytes)
        assert_eq!(hash_hex.len(), 64);

        // Verify they are valid hex
        assert!(hex::decode(&sig_hex).is_ok());
        assert!(hex::decode(&hash_hex).is_ok());
    }

    #[test]
    #[should_panic(expected = "Order must be signed before converting to submit request")]
    fn test_to_submit_request_panics_unsigned() {
        let order = SignedOrder {
            nonce: 1,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            maker_amount: 100,
            taker_amount: 50,
            expiration: 0,
            signature: [0u8; 64],
        };

        order.to_submit_request("test_orderbook");
    }

    // ========================================================================
    // SignedCancelOrder tests
    // ========================================================================

    #[test]
    fn test_signed_cancel_order_new() {
        let maker = Pubkey::new_unique();
        let hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let cancel = SignedCancelOrder::new(hash, maker);
        assert!(!cancel.is_signed());
        assert_eq!(cancel.order_hash, hash);
        assert_eq!(cancel.maker, maker);
    }

    #[test]
    fn test_signed_cancel_order_signing_message() {
        let hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let cancel = SignedCancelOrder::new(hash, Pubkey::new_unique());
        assert_eq!(cancel.signing_message(), hash.as_bytes().to_vec());
    }

    #[test]
    #[cfg(feature = "client")]
    fn test_signed_cancel_order_sign_and_verify() {
        let keypair = Keypair::new();
        let hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";

        let mut cancel = SignedCancelOrder::new(hash, keypair.pubkey());
        cancel.sign(&keypair);
        assert!(cancel.is_signed());
        assert_eq!(cancel.signature_hex().len(), 128);
        cancel.verify_signature().expect("signature should verify");
    }

    #[test]
    #[cfg(feature = "client")]
    fn test_signed_cancel_order_new_signed() {
        let keypair = Keypair::new();
        let cancel = SignedCancelOrder::new_signed("abc123", keypair.pubkey(), &keypair);
        assert!(cancel.is_signed());
        cancel.verify_signature().expect("signature should verify");
    }

    #[test]
    #[should_panic(expected = "Cancel request must be signed")]
    fn test_signed_cancel_order_to_request_panics_unsigned() {
        let cancel = SignedCancelOrder::new("abc123", Pubkey::new_unique());
        cancel.to_cancel_request();
    }

    #[test]
    #[cfg(feature = "client")]
    fn test_signed_cancel_order_to_request() {
        let keypair = Keypair::new();
        let cancel = SignedCancelOrder::new_signed("abc123", keypair.pubkey(), &keypair);
        let request = cancel.to_cancel_request();
        assert_eq!(request.order_hash, "abc123");
        assert_eq!(request.maker, keypair.pubkey().to_string());
        assert_eq!(request.signature.len(), 128);
    }

    // ========================================================================
    // SignedCancelAll tests
    // ========================================================================

    #[test]
    fn test_signed_cancel_all_new() {
        let pubkey = Pubkey::new_unique();
        let cancel = SignedCancelAll::new(pubkey, 1700000000);
        assert!(!cancel.is_signed());
        assert_eq!(cancel.user_pubkey, pubkey);
        assert_eq!(cancel.timestamp, 1700000000);
        assert!(cancel.orderbook_id.is_none());
    }

    #[test]
    fn test_signed_cancel_all_for_orderbook() {
        let pubkey = Pubkey::new_unique();
        let cancel = SignedCancelAll::new_for_orderbook(pubkey, 1700000000, "my_orderbook");
        assert_eq!(cancel.orderbook_id, Some("my_orderbook".to_string()));
    }

    #[test]
    fn test_signed_cancel_all_now() {
        let pubkey = Pubkey::new_unique();
        let cancel = SignedCancelAll::now(pubkey);
        assert!(cancel.timestamp > 0);
        assert!(!cancel.is_signed());
    }

    #[test]
    fn test_signed_cancel_all_signing_message() {
        let pubkey = Pubkey::new_unique();
        let cancel = SignedCancelAll::new(pubkey, 1700000000);
        let expected = format!("cancel_all:{}:1700000000", pubkey);
        assert_eq!(cancel.signing_message(), expected.into_bytes());
    }

    #[test]
    #[cfg(feature = "client")]
    fn test_signed_cancel_all_sign_and_verify() {
        let keypair = Keypair::new();

        let mut cancel = SignedCancelAll::new(keypair.pubkey(), 1700000000);
        cancel.sign(&keypair);
        assert!(cancel.is_signed());
        assert_eq!(cancel.signature_hex().len(), 128);
        cancel.verify_signature().expect("signature should verify");
    }

    #[test]
    #[cfg(feature = "client")]
    fn test_signed_cancel_all_now_signed() {
        let keypair = Keypair::new();
        let cancel = SignedCancelAll::now_signed(keypair.pubkey(), &keypair);
        assert!(cancel.is_signed());
        cancel.verify_signature().expect("signature should verify");
    }

    #[test]
    #[should_panic(expected = "Cancel-all request must be signed")]
    fn test_signed_cancel_all_to_request_panics_unsigned() {
        let cancel = SignedCancelAll::new(Pubkey::new_unique(), 1700000000);
        cancel.to_cancel_all_request();
    }

    #[test]
    #[cfg(feature = "client")]
    fn test_signed_cancel_all_to_request() {
        let keypair = Keypair::new();
        let cancel = SignedCancelAll::new_signed(keypair.pubkey(), 1700000000, &keypair);
        let request = cancel.to_cancel_all_request();
        assert_eq!(request.user_pubkey, keypair.pubkey().to_string());
        assert_eq!(request.timestamp, 1700000000);
        assert!(request.orderbook_id.is_none());
        assert_eq!(request.signature.len(), 128);
    }

    #[test]
    #[cfg(feature = "client")]
    fn test_signed_cancel_all_to_request_with_orderbook() {
        let keypair = Keypair::new();
        let mut cancel = SignedCancelAll::new_for_orderbook(
            keypair.pubkey(),
            1700000000,
            "test_ob",
        );
        cancel.sign(&keypair);
        let request = cancel.to_cancel_all_request();
        assert_eq!(request.orderbook_id, Some("test_ob".to_string()));
    }
}
