//! Order types, serialization, hashing, and signing.
//!
//! This module provides the full and compact order structures with
//! Keccak256 hashing and Ed25519 signing functionality.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha3::{Digest, Keccak256};
use solana_sdk::{pubkey::Pubkey, signature::Keypair};

use crate::constants::{COMPACT_ORDER_SIZE, FULL_ORDER_SIZE};
use crate::error::{SdkError, SdkResult};
use crate::types::{AskOrderParams, BidOrderParams, OrderSide};

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

        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(self.maker.as_ref());
        hasher.update(self.market.as_ref());
        hasher.update(self.base_mint.as_ref());
        hasher.update(self.quote_mint.as_ref());
        hasher.update(&[self.side as u8]);
        hasher.update(&self.maker_amount.to_le_bytes());
        hasher.update(&self.taker_amount.to_le_bytes());
        hasher.update(&self.expiration.to_le_bytes());

        hasher.finalize().into()
    }

    /// Sign the order with the given keypair.
    pub fn sign(&mut self, keypair: &Keypair) {
        let hash = self.hash();
        let signing_key = SigningKey::from_bytes(&keypair.secret_bytes());
        let signature = signing_key.sign(&hash);
        self.signature = signature.to_bytes();
    }

    /// Create and sign an order in one step.
    pub fn new_bid_signed(params: BidOrderParams, keypair: &Keypair) -> Self {
        let mut order = Self::new_bid(params);
        order.sign(keypair);
        order
    }

    /// Create and sign an ask order in one step.
    pub fn new_ask_signed(params: AskOrderParams, keypair: &Keypair) -> Self {
        let mut order = Self::new_ask(params);
        order.sign(keypair);
        order
    }

    /// Verify the signature against the maker's pubkey.
    pub fn verify_signature(&self) -> SdkResult<bool> {
        let hash = self.hash();
        let pubkey_bytes: &[u8; 32] = self.maker.as_ref().try_into()
            .map_err(|_| SdkError::InvalidPubkey("Invalid maker pubkey".to_string()))?;
        let verifying_key = VerifyingKey::from_bytes(pubkey_bytes)
            .map_err(|_| SdkError::InvalidPubkey("Invalid maker pubkey".to_string()))?;
        let signature = Signature::from_bytes(&self.signature);

        Ok(verifying_key.verify(&hash, &signature).is_ok())
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
    hasher.update(&[num_outcomes]);
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
            maker_amount: 50,  // 50 base
            taker_amount: 90,  // for 90 quote (price = 1.8 quote/base)
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
            maker_amount: 50,  // 50 quote
            taker_amount: 50,  // for 50 base (price = 1 quote/base)
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
}
