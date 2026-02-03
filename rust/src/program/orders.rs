//! Order types, serialization, hashing, and signing.
//!
//! This module provides the full and compact order structures with
//! Keccak256 hashing and Ed25519 signing functionality.

use sha3::{Digest, Keccak256};
use solana_pubkey::Pubkey;
use solana_signature::Signature;

#[cfg(feature = "client")]
use solana_keypair::Keypair;
#[cfg(feature = "client")]
use solana_signer::Signer;

use crate::program::constants::{COMPACT_ORDER_SIZE, FULL_ORDER_SIZE};
use crate::program::error::{SdkError, SdkResult};
use crate::program::types::{AskOrderParams, BidOrderParams, OrderSide};
use crate::shared::SubmitOrderRequest;

// ============================================================================
// Full Order (225 bytes)
// ============================================================================

/// Full order structure with signature.
///
/// Layout (225 bytes):
/// - [0..8]     nonce (8 bytes)
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
pub struct FullOrder {
    /// Unique order ID and replay protection
    pub nonce: u64,
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

impl FullOrder {
    /// Order size in bytes
    pub const LEN: usize = FULL_ORDER_SIZE;

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

    /// Compute the Keccak256 hash of the order (excludes signature).
    ///
    /// Hash layout (161 bytes):
    /// - nonce (8)
    /// - maker (32)
    /// - market (32)
    /// - base_mint (32)
    /// - quote_mint (32)
    /// - side (1)
    /// - maker_amount (8)
    /// - taker_amount (8)
    /// - expiration (8)
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Keccak256::new();

        hasher.update(self.nonce.to_le_bytes());
        hasher.update(self.maker.as_ref());
        hasher.update(self.market.as_ref());
        hasher.update(self.base_mint.as_ref());
        hasher.update(self.quote_mint.as_ref());
        hasher.update([self.side as u8]);
        hasher.update(self.maker_amount.to_le_bytes());
        hasher.update(self.taker_amount.to_le_bytes());
        hasher.update(self.expiration.to_le_bytes());

        hasher.finalize().into()
    }

    /// Sign the order with the given keypair.
    #[cfg(feature = "client")]
    pub fn sign(&mut self, keypair: &Keypair) {
        let hash = self.hash();
        let sig = keypair.sign_message(&hash);
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

    /// Verify a hash signature against the maker's pubkey.
    pub fn verify_signature(&self) -> SdkResult<()> {
        let hash: [u8; 32] = self.hash();

        let sig = Signature::try_from(self.signature.as_slice())
            .map_err(|_| SdkError::InvalidSignature)?;

        if !sig.verify(self.maker.as_ref(), &hash) {
            return Err(SdkError::SignatureVerificationFailed);
        }
        Ok(())
    }

    /// Apply a signature to the order.
    pub fn apply_signature(&mut self, signature: &[u8]) -> SdkResult<()> {
        self.signature = signature
            .try_into()
            .map_err(|_| SdkError::InvalidSignature)?;
        Ok(())
    }

    /// Serialize to bytes (225 bytes).
    pub fn serialize(&self) -> [u8; FULL_ORDER_SIZE] {
        let mut data = [0u8; FULL_ORDER_SIZE];

        data[0..8].copy_from_slice(&self.nonce.to_le_bytes());
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
        if data.len() < FULL_ORDER_SIZE {
            return Err(SdkError::InvalidDataLength {
                expected: FULL_ORDER_SIZE,
                actual: data.len(),
            });
        }

        let mut nonce_bytes = [0u8; 8];
        nonce_bytes.copy_from_slice(&data[0..8]);

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
            nonce: u64::from_le_bytes(nonce_bytes),
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

    /// Convert to compact order format.
    pub fn to_compact(&self) -> CompactOrder {
        CompactOrder {
            nonce: self.nonce,
            maker: self.maker,
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

    /// Get the order hash as a hex string (64 chars).
    pub fn hash_hex(&self) -> String {
        hex::encode(self.hash())
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
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut order = FullOrder::new_bid(params);
    /// order.sign(&keypair);
    ///
    /// let request = order.to_submit_request(order.derive_orderbook_id());
    /// let response = api_client.submit_order(request).await?;
    /// ```
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
// Compact Order (65 bytes)
// ============================================================================

/// Compact order format for transaction size optimization.
///
/// Excludes market, base_mint, quote_mint which are passed via accounts.
///
/// Layout (65 bytes):
/// - [0..8]   nonce (8 bytes)
/// - [8..40]  maker (32 bytes)
/// - [40]     side (1 byte)
/// - [41..49] maker_amount (8 bytes)
/// - [49..57] taker_amount (8 bytes)
/// - [57..65] expiration (8 bytes)
#[derive(Debug, Clone)]
pub struct CompactOrder {
    /// Unique order ID and replay protection
    pub nonce: u64,
    /// Order maker's pubkey
    pub maker: Pubkey,
    /// Order side (0 = Bid, 1 = Ask)
    pub side: OrderSide,
    /// Amount maker gives
    pub maker_amount: u64,
    /// Amount maker receives
    pub taker_amount: u64,
    /// Expiration timestamp (0 = no expiration)
    pub expiration: i64,
}

impl CompactOrder {
    /// Order size in bytes
    pub const LEN: usize = COMPACT_ORDER_SIZE;

    /// Serialize to bytes (65 bytes).
    pub fn serialize(&self) -> [u8; COMPACT_ORDER_SIZE] {
        let mut data = [0u8; COMPACT_ORDER_SIZE];

        data[0..8].copy_from_slice(&self.nonce.to_le_bytes());
        data[8..40].copy_from_slice(self.maker.as_ref());
        data[40] = self.side as u8;
        data[41..49].copy_from_slice(&self.maker_amount.to_le_bytes());
        data[49..57].copy_from_slice(&self.taker_amount.to_le_bytes());
        data[57..65].copy_from_slice(&self.expiration.to_le_bytes());

        data
    }

    /// Deserialize from bytes.
    pub fn deserialize(data: &[u8]) -> SdkResult<Self> {
        if data.len() < COMPACT_ORDER_SIZE {
            return Err(SdkError::InvalidDataLength {
                expected: COMPACT_ORDER_SIZE,
                actual: data.len(),
            });
        }

        let mut nonce_bytes = [0u8; 8];
        nonce_bytes.copy_from_slice(&data[0..8]);

        let mut maker_bytes = [0u8; 32];
        maker_bytes.copy_from_slice(&data[8..40]);

        let mut maker_amount_bytes = [0u8; 8];
        maker_amount_bytes.copy_from_slice(&data[41..49]);

        let mut taker_amount_bytes = [0u8; 8];
        taker_amount_bytes.copy_from_slice(&data[49..57]);

        let mut expiration_bytes = [0u8; 8];
        expiration_bytes.copy_from_slice(&data[57..65]);

        Ok(Self {
            nonce: u64::from_le_bytes(nonce_bytes),
            maker: Pubkey::new_from_array(maker_bytes),
            side: OrderSide::try_from(data[40])?,
            maker_amount: u64::from_le_bytes(maker_amount_bytes),
            taker_amount: u64::from_le_bytes(taker_amount_bytes),
            expiration: i64::from_le_bytes(expiration_bytes),
        })
    }

    /// Expand to full order using pubkeys from accounts.
    pub fn to_full_order(
        &self,
        market: Pubkey,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        signature: [u8; 64],
    ) -> FullOrder {
        FullOrder {
            nonce: self.nonce,
            maker: self.maker,
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
pub fn is_order_expired(order: &FullOrder, current_time: i64) -> bool {
    order.expiration != 0 && current_time >= order.expiration
}

/// Check if two orders can cross (prices are compatible).
///
/// Returns true if the buyer's price >= seller's price.
pub fn orders_can_cross(buy_order: &FullOrder, sell_order: &FullOrder) -> bool {
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
pub fn calculate_taker_fill(maker_order: &FullOrder, maker_fill_amount: u64) -> SdkResult<u64> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_serialization_roundtrip() {
        let order = FullOrder {
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
        let deserialized = FullOrder::deserialize(&serialized).unwrap();

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
    fn test_compact_order_serialization_roundtrip() {
        let order = CompactOrder {
            nonce: 12345,
            maker: Pubkey::new_unique(),
            side: OrderSide::Ask,
            maker_amount: 1000000,
            taker_amount: 500000,
            expiration: 1234567890,
        };

        let serialized = order.serialize();
        let deserialized = CompactOrder::deserialize(&serialized).unwrap();

        assert_eq!(order.nonce, deserialized.nonce);
        assert_eq!(order.maker, deserialized.maker);
        assert_eq!(order.side, deserialized.side);
        assert_eq!(order.maker_amount, deserialized.maker_amount);
        assert_eq!(order.taker_amount, deserialized.taker_amount);
        assert_eq!(order.expiration, deserialized.expiration);
    }

    #[test]
    fn test_order_hash_consistency() {
        let order = FullOrder {
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
    fn test_orders_can_cross() {
        let buy_order = FullOrder {
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

        let sell_order = FullOrder {
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
        let buy_order = FullOrder {
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

        let sell_order = FullOrder {
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
        let maker_order = FullOrder {
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

        let mut order = FullOrder {
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
        let order = FullOrder {
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
        let mut order = FullOrder {
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
        let mut order = FullOrder {
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
        let order = FullOrder {
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
}
