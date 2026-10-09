//! Order types, serialization, hashing, and signing.
//!
//! This module provides the signed and compact order structures with
//! Keccak256 hashing and Ed25519 signing functionality.

use sha3::{Digest, Keccak256};
use solana_pubkey::Pubkey;
use solana_signature::Signature;

#[cfg(feature = "native-auth")]
use solana_keypair::Keypair;
#[cfg(feature = "native-auth")]
use solana_signer::Signer;

use crate::program::constants::{ORDER_PREIMAGE_SIZE, ORDER_SIZE, SIGNED_ORDER_SIZE};
use crate::program::error::{SdkError, SdkResult};
use crate::program::types::{AskOrderParams, BidOrderParams, OrderSide};
use crate::shared::{validate_raw_amounts, OrderbookRules, SubmitOrderRequest};

// ============================================================================
// Signed Order (225 bytes)
// ============================================================================

/// Signed order structure with full context and signature.
///
/// There is no per-user nonce: the salt is the order's only identity field.
/// Two orders with identical fields and salt have the same hash.
///
/// Layout (225 bytes, integers little-endian). The first 161 bytes are the
/// signing preimage:
/// - [0..8]     salt (8 bytes, u64)
/// - [8..40]    maker (32 bytes)
/// - [40..72]   market (32 bytes)
/// - [72..104]  base_mint (32 bytes)
/// - [104..136] quote_mint (32 bytes)
/// - [136]      side (1 byte)
/// - [137..145] amount_in (8 bytes)
/// - [145..153] amount_out (8 bytes)
/// - [153..161] expiration (8 bytes)
/// - [161..225] signature (64 bytes)
#[derive(Debug, Clone)]
pub struct OrderPayload {
    /// Order identity salt. Any u64, including 0, is valid.
    pub salt: u64,
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
    /// Amount the maker gives, in raw atoms of the given mint (the program's `maker_amount`)
    pub amount_in: u64,
    /// Amount the maker receives, in raw atoms of the received mint (the program's `taker_amount`)
    pub amount_out: u64,
    /// Expiration as Unix seconds (0 = no expiration)
    pub expiration: i64,
    /// Ed25519 signature
    pub signature: [u8; 64],
}

impl OrderPayload {
    /// Order size in bytes
    pub const LEN: usize = SIGNED_ORDER_SIZE;

    /// Size of the signed preimage (the hashed portion of the order)
    pub const HASH_SIZE: usize = ORDER_PREIMAGE_SIZE;

    /// Create a new bid order (maker buys base, gives quote)
    pub fn new_bid(params: BidOrderParams) -> Self {
        Self {
            salt: params.salt,
            maker: params.maker,
            market: params.market,
            base_mint: params.base_mint,
            quote_mint: params.quote_mint,
            side: OrderSide::Bid,
            amount_in: params.amount_in,
            amount_out: params.amount_out,
            expiration: params.expiration,
            signature: [0u8; 64],
        }
    }

    /// Create a new ask order (maker sells base, receives quote)
    pub fn new_ask(params: AskOrderParams) -> Self {
        Self {
            salt: params.salt,
            maker: params.maker,
            market: params.market,
            base_mint: params.base_mint,
            quote_mint: params.quote_mint,
            side: OrderSide::Ask,
            amount_in: params.amount_in,
            amount_out: params.amount_out,
            expiration: params.expiration,
            signature: [0u8; 64],
        }
    }

    /// Build the raw 161-byte order preimage from the signed fields.
    /// This is hashed (keccak256) and hex-encoded to produce the bytes that users sign.
    fn preimage(&self) -> [u8; Self::HASH_SIZE] {
        let mut data = [0u8; Self::HASH_SIZE];

        data[0..8].copy_from_slice(&self.salt.to_le_bytes());
        data[8..40].copy_from_slice(self.maker.as_ref());
        data[40..72].copy_from_slice(self.market.as_ref());
        data[72..104].copy_from_slice(self.base_mint.as_ref());
        data[104..136].copy_from_slice(self.quote_mint.as_ref());
        data[136] = self.side as u8;
        data[137..145].copy_from_slice(&self.amount_in.to_le_bytes());
        data[145..153].copy_from_slice(&self.amount_out.to_le_bytes());
        data[153..161].copy_from_slice(&self.expiration.to_le_bytes());

        data
    }

    /// Compute the 32-byte Keccak256 hash of the signed fields.
    ///
    /// This is the order ID and the order-status PDA seed.
    pub fn hash(&self) -> [u8; 32] {
        Keccak256::digest(self.preimage()).into()
    }

    /// Compute the order hash as a lowercase hex string.
    ///
    /// Its 64 ASCII bytes are the message the maker signs.
    pub fn hash_hex(&self) -> String {
        hex::encode(self.hash())
    }

    /// Sign the order with the given keypair.
    #[cfg(feature = "native-auth")]
    pub fn sign(&mut self, keypair: &Keypair, rules: &OrderbookRules) -> SdkResult<()> {
        crate::shared::validate_signed_fields(self.amount_in, self.amount_out)?;
        validate_raw_amounts(self.amount_in, self.amount_out, self.side, rules)?;
        let hash = self.hash_hex();
        let sig = keypair.sign_message(hash.as_bytes());

        self.signature.copy_from_slice(sig.as_ref());
        Ok(())
    }

    /// Create and sign an order in one step.
    #[cfg(feature = "native-auth")]
    pub fn new_bid_signed(
        params: BidOrderParams,
        keypair: &Keypair,
        rules: &OrderbookRules,
    ) -> SdkResult<Self> {
        let mut order = Self::new_bid(params);
        order.sign(keypair, rules)?;
        Ok(order)
    }

    /// Create and sign an ask order in one step.
    #[cfg(feature = "native-auth")]
    pub fn new_ask_signed(
        params: AskOrderParams,
        keypair: &Keypair,
        rules: &OrderbookRules,
    ) -> SdkResult<Self> {
        let mut order = Self::new_ask(params);
        order.sign(keypair, rules)?;
        Ok(order)
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
    pub fn apply_signature(&mut self, sig_bs58: String, rules: &OrderbookRules) -> SdkResult<()> {
        crate::shared::validate_signed_fields(self.amount_in, self.amount_out)?;
        validate_raw_amounts(self.amount_in, self.amount_out, self.side, rules)?;
        let signature = sig_bs58
            .parse::<Signature>()
            .map_err(|_| SdkError::InvalidSignature)?;

        self.signature = signature.into();
        Ok(())
    }

    /// Serialize to bytes (225 bytes): the preimage followed by the signature.
    pub fn serialize(&self) -> [u8; SIGNED_ORDER_SIZE] {
        let mut data = [0u8; SIGNED_ORDER_SIZE];

        data[..ORDER_PREIMAGE_SIZE].copy_from_slice(&self.preimage());
        data[ORDER_PREIMAGE_SIZE..].copy_from_slice(&self.signature);

        data
    }

    /// Deserialize from exactly 225 bytes.
    pub fn deserialize(data: &[u8]) -> SdkResult<Self> {
        if data.len() != SIGNED_ORDER_SIZE {
            return Err(SdkError::InvalidDataLength {
                expected: SIGNED_ORDER_SIZE,
                actual: data.len(),
            });
        }

        let mut salt_bytes = [0u8; 8];
        salt_bytes.copy_from_slice(&data[0..8]);

        let mut maker_bytes = [0u8; 32];
        maker_bytes.copy_from_slice(&data[8..40]);

        let mut market_bytes = [0u8; 32];
        market_bytes.copy_from_slice(&data[40..72]);

        let mut base_mint_bytes = [0u8; 32];
        base_mint_bytes.copy_from_slice(&data[72..104]);

        let mut quote_mint_bytes = [0u8; 32];
        quote_mint_bytes.copy_from_slice(&data[104..136]);

        let mut amount_in_bytes = [0u8; 8];
        amount_in_bytes.copy_from_slice(&data[137..145]);

        let mut amount_out_bytes = [0u8; 8];
        amount_out_bytes.copy_from_slice(&data[145..153]);

        let mut expiration_bytes = [0u8; 8];
        expiration_bytes.copy_from_slice(&data[153..161]);

        let mut signature = [0u8; 64];
        signature.copy_from_slice(&data[161..225]);

        Ok(Self {
            salt: u64::from_le_bytes(salt_bytes),
            maker: Pubkey::new_from_array(maker_bytes),
            market: Pubkey::new_from_array(market_bytes),
            base_mint: Pubkey::new_from_array(base_mint_bytes),
            quote_mint: Pubkey::new_from_array(quote_mint_bytes),
            side: OrderSide::try_from(data[136])?,
            amount_in: u64::from_le_bytes(amount_in_bytes),
            amount_out: u64::from_le_bytes(amount_out_bytes),
            expiration: i64::from_le_bytes(expiration_bytes),
            signature,
        })
    }

    /// Convert to compact order format (33 bytes, no maker or mint fields).
    pub fn to_order(&self) -> Order {
        Order {
            salt: self.salt,
            side: self.side,
            amount_in: self.amount_in,
            amount_out: self.amount_out,
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

    /// Convert a signed payload to a limit-order `SubmitOrderRequest`.
    ///
    /// Intended for internal use by envelope types. Prefer using
    /// `LimitOrderEnvelope::sign()`.
    pub(crate) fn to_submit_request(
        &self,
        orderbook_id: impl Into<String>,
        time_in_force: Option<crate::shared::TimeInForce>,
        deposit_source: Option<crate::shared::DepositSource>,
    ) -> Result<SubmitOrderRequest, SdkError> {
        if self.signature == [0u8; 64] {
            return Err(SdkError::UnsignedOrder);
        }

        Ok(SubmitOrderRequest {
            maker: self.maker.to_string(),
            salt: self.salt,
            market_pubkey: self.market.to_string(),
            base_token: self.base_mint.to_string(),
            quote_token: self.quote_mint.to_string(),
            side: self.side as u32,
            amount_in: self.amount_in,
            amount_out: self.amount_out,
            expiration: self.expiration,
            signature: hex::encode(self.signature),
            orderbook_id: orderbook_id.into(),
            time_in_force,
            deposit_source,
        })
    }

    /// Derive the orderbook ID for this order.
    ///
    /// Format: `{base_token[0:8]}_{quote_token[0:8]}`
    pub fn derive_orderbook_id(&self) -> String {
        crate::shared::derive_orderbook_id(
            &self.base_mint.to_string(),
            &self.quote_mint.to_string(),
        )
        .to_string()
    }
}

// ============================================================================
// Order (33 bytes)
// ============================================================================

/// Compact order format for on-chain transaction data.
///
/// No `maker`, `market`, or mint fields: the program reads them from the
/// instruction's accounts when it rebuilds the signed preimage.
///
/// Layout (33 bytes, integers little-endian):
/// - [0..8]   salt (8 bytes, u64)
/// - [8]      side (1 byte)
/// - [9..17]  amount_in (8 bytes)
/// - [17..25] amount_out (8 bytes)
/// - [25..33] expiration (8 bytes)
#[derive(Debug, Clone)]
pub struct Order {
    /// Order identity salt
    pub salt: u64,
    /// Order side (0 = Bid, 1 = Ask)
    pub side: OrderSide,
    /// Amount the maker gives, in raw atoms of the given mint
    pub amount_in: u64,
    /// Amount the maker receives, in raw atoms of the received mint
    pub amount_out: u64,
    /// Expiration as Unix seconds (0 = no expiration)
    pub expiration: i64,
}

impl Order {
    /// Order size in bytes
    pub const LEN: usize = ORDER_SIZE;

    /// Serialize to bytes (33 bytes).
    pub fn serialize(&self) -> [u8; ORDER_SIZE] {
        let mut data = [0u8; ORDER_SIZE];

        data[0..8].copy_from_slice(&self.salt.to_le_bytes());
        data[8] = self.side as u8;
        data[9..17].copy_from_slice(&self.amount_in.to_le_bytes());
        data[17..25].copy_from_slice(&self.amount_out.to_le_bytes());
        data[25..33].copy_from_slice(&self.expiration.to_le_bytes());

        data
    }

    /// Deserialize from exactly 33 bytes.
    pub fn deserialize(data: &[u8]) -> SdkResult<Self> {
        if data.len() != ORDER_SIZE {
            return Err(SdkError::InvalidDataLength {
                expected: ORDER_SIZE,
                actual: data.len(),
            });
        }

        let mut salt_bytes = [0u8; 8];
        salt_bytes.copy_from_slice(&data[0..8]);

        let mut amount_in_bytes = [0u8; 8];
        amount_in_bytes.copy_from_slice(&data[9..17]);

        let mut amount_out_bytes = [0u8; 8];
        amount_out_bytes.copy_from_slice(&data[17..25]);

        let mut expiration_bytes = [0u8; 8];
        expiration_bytes.copy_from_slice(&data[25..33]);

        Ok(Self {
            salt: u64::from_le_bytes(salt_bytes),
            side: OrderSide::try_from(data[8])?,
            amount_in: u64::from_le_bytes(amount_in_bytes),
            amount_out: u64::from_le_bytes(amount_out_bytes),
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
    ) -> OrderPayload {
        OrderPayload {
            salt: self.salt,
            maker,
            market,
            base_mint,
            quote_mint,
            side: self.side,
            amount_in: self.amount_in,
            amount_out: self.amount_out,
            expiration: self.expiration,
            signature,
        }
    }
}

// ============================================================================
// Order Validation Helpers
// ============================================================================

/// Check if an order is expired at `current_time` (Unix seconds).
///
/// Matches the program: an order expires only when its expiration is nonzero
/// and strictly earlier than the current time, so it is still valid during the
/// second equal to its expiration.
pub fn is_order_expired(order: &OrderPayload, current_time: i64) -> bool {
    order.expiration != 0 && order.expiration < current_time
}

/// Check if two orders can cross (prices are compatible).
///
/// Returns true if the buyer's price >= seller's price.
pub fn orders_can_cross(buy_order: &OrderPayload, sell_order: &OrderPayload) -> bool {
    if buy_order.side != OrderSide::Bid || sell_order.side != OrderSide::Ask {
        return false;
    }

    if buy_order.amount_in == 0
        || buy_order.amount_out == 0
        || sell_order.amount_in == 0
        || sell_order.amount_out == 0
    {
        return false;
    }

    // Buyer gives quote, receives base
    // Seller gives base, receives quote
    // Cross condition: buyer's price >= seller's price
    // buyer_price = buyer.amount_in / buyer.amount_out (quote per base)
    // seller_price = seller.amount_out / seller.amount_in (quote per base)
    // Cross: buyer.amount_in / buyer.amount_out >= seller.amount_out / seller.amount_in
    // Rearrange: buyer.amount_in * seller.amount_in >= buyer.amount_out * seller.amount_out

    let buyer_cross = (buy_order.amount_in as u128) * (sell_order.amount_in as u128);
    let seller_cross = (buy_order.amount_out as u128) * (sell_order.amount_out as u128);

    buyer_cross >= seller_cross
}

/// Calculate the minimum taker fill for a maker fill, in raw atoms.
///
/// `maker_fill_amount` is what the maker gives, in atoms of the maker's given
/// mint. The result is the least the taker must give the maker, in atoms of the
/// maker's received mint, rounded up as the program requires:
/// `taker_fill >= ceil(maker_fill * maker.amount_out / maker.amount_in)`.
pub fn calculate_taker_fill(maker_order: &OrderPayload, maker_fill_amount: u64) -> SdkResult<u64> {
    if maker_order.amount_in == 0 {
        return Err(SdkError::Overflow);
    }

    let numerator = u128::from(maker_fill_amount)
        .checked_mul(u128::from(maker_order.amount_out))
        .ok_or(SdkError::Overflow)?;
    let result = numerator.div_ceil(u128::from(maker_order.amount_in));

    u64::try_from(result).map_err(|_| SdkError::Overflow)
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
// Cancel Order Signing Helpers
// ============================================================================

/// Build the message bytes for cancelling an order.
///
/// The message is the order hash hex string as UTF-8 bytes (same protocol as order signing).
pub fn cancel_order_message(order_hash: &str) -> Vec<u8> {
    order_hash.as_bytes().to_vec()
}

/// Build the message bytes for cancelling a trigger order.
///
/// The message is the trigger_order_id as UTF-8 bytes.
#[cfg(feature = "trigger_orders")]
pub fn cancel_trigger_order_message(trigger_order_id: &str) -> Vec<u8> {
    trigger_order_id.as_bytes().to_vec()
}

/// Build the message string for cancelling all orders.
///
/// Format: `"cancel_all:{pubkey}:{orderbook_id}:{timestamp}:{salt}"`
pub fn cancel_all_message(
    user_pubkey: &str,
    orderbook_id: &str,
    timestamp: i64,
    salt: &str,
) -> String {
    format!(
        "cancel_all:{}:{}:{}:{}",
        user_pubkey, orderbook_id, timestamp, salt
    )
}

/// Generate a random salt for order uniqueness.
///
/// The salt is the order's only identity field, so it spans the full u64 range.
pub fn generate_salt() -> u64 {
    rand::random::<u64>()
}

/// Generate a random UUID v4 salt for cancel-all replay protection.
pub fn generate_cancel_all_salt() -> String {
    let mut bytes = rand::random::<[u8; 16]>();
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{}-{}-{}-{}-{}",
        hex::encode(&bytes[0..4]),
        hex::encode(&bytes[4..6]),
        hex::encode(&bytes[6..8]),
        hex::encode(&bytes[8..10]),
        hex::encode(&bytes[10..16]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signing_rules() -> OrderbookRules {
        serde_json::from_str(
            r#"{
                "orderbook_id":"test",
                "base_decimals":0,
                "quote_decimals":0,
                "price_decimals":6,
                "trading_rules":{
                    "base_size_decimals":0,
                    "max_price_decimals":6,
                    "max_price_significant_figures":5,
                    "integer_prices_always_allowed":true,
                    "price_quantum":"0.000001",
                    "price_quantum_raw":"1",
                    "base_size_quantum":"1",
                    "base_size_quantum_raw":"1"
                }
            }"#,
        )
        .unwrap()
    }

    /// One program-contract signing vector. Amounts are raw atoms, expiration
    /// is Unix seconds, and every hex field is the backend's expected bytes.
    struct SigningVector {
        name: &'static str,
        salt: u64,
        side: OrderSide,
        maker_amount: u64,
        taker_amount: u64,
        expiration: i64,
        preimage_hex: &'static str,
        order_id_hex: &'static str,
        signature_hex: &'static str,
        compact_hex: &'static str,
    }

    // Backend signing vectors (fixtures/program-contract/v1/signing.json),
    // generated independently of this SDK with Python struct, Keccak and
    // Ed25519. Every vector is signed by seed 00..1f, whose public key is the
    // maker, on market [0x11; 32], base mint [0x22; 32] and quote mint [0x33; 32].
    #[cfg(feature = "native-auth")]
    const SIGNING_SEED_HEX: &str =
        "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    const SIGNING_MAKER_HEX: &str =
        "03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8";
    const SIGNING_VECTORS: [SigningVector; 4] = [
        SigningVector {
            name: "bid",
            salt: 72_623_859_790_382_856,
            side: OrderSide::Bid,
            maker_amount: 60_000_000,
            taker_amount: 100_000_000,
            expiration: 0,
            preimage_hex: concat!(
                "0807060504030201",
                "03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8",
                "1111111111111111111111111111111111111111111111111111111111111111",
                "2222222222222222222222222222222222222222222222222222222222222222",
                "3333333333333333333333333333333333333333333333333333333333333333",
                "00008793030000000000e1f505000000000000000000000000",
            ),
            order_id_hex: "497bbc21bdf8d3fb1187f0eda8f7a0f124dc97511e1f5068bbb7180b6b4a9bdc",
            signature_hex: concat!(
                "28ee8645e9a4f1cd3b28f910531d2d7b29b36038f873f5e1517058c02cf4cc4b",
                "ec8400596d0e87766c3a527e4f2d04b4d0b592be656b91160e48b3b5c6a7e800",
            ),
            compact_hex: "080706050403020100008793030000000000e1f505000000000000000000000000",
        },
        SigningVector {
            name: "ask",
            salt: 9_833_440_827_789_222_417,
            side: OrderSide::Ask,
            maker_amount: 100_000_000,
            taker_amount: 60_000_000,
            expiration: 1_700_000_000,
            preimage_hex: concat!(
                "1122334455667788",
                "03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8",
                "1111111111111111111111111111111111111111111111111111111111111111",
                "2222222222222222222222222222222222222222222222222222222222222222",
                "3333333333333333333333333333333333333333333333333333333333333333",
                "0100e1f50500000000008793030000000000f1536500000000",
            ),
            order_id_hex: "ee414532884ed99be69c82c9a99d4916ccf13bb17c955d2e979c43df6697fdfc",
            signature_hex: concat!(
                "f7df8ad52408140f9418b0a9c3a5925ffe41d58f68899001769748eaa84a4f55",
                "f5afc47c8a2d241f38b63141012f8970fecc159dfad855c624c4def66cd2c80a",
            ),
            compact_hex: "11223344556677880100e1f50500000000008793030000000000f1536500000000",
        },
        SigningVector {
            name: "zero_salt",
            salt: 0,
            side: OrderSide::Bid,
            maker_amount: 1,
            taker_amount: 1,
            expiration: 0,
            preimage_hex: concat!(
                "0000000000000000",
                "03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8",
                "1111111111111111111111111111111111111111111111111111111111111111",
                "2222222222222222222222222222222222222222222222222222222222222222",
                "3333333333333333333333333333333333333333333333333333333333333333",
                "00010000000000000001000000000000000000000000000000",
            ),
            order_id_hex: "bddbbc63b3eca9b0c8ab96920b44d44239042684fa523f4d9e3e35d23228a77c",
            signature_hex: concat!(
                "2a0e4bea081930c109b666e29b979bdcfcaddb1b169a2bed1f1df60ac9522fcd",
                "fee68652f00541151af5cc2ca9c288a79e6dab71904ee3f35efd4fb4caf43506",
            ),
            compact_hex: "000000000000000000010000000000000001000000000000000000000000000000",
        },
        SigningVector {
            name: "max_salt",
            salt: u64::MAX,
            side: OrderSide::Ask,
            maker_amount: u64::MAX,
            taker_amount: u64::MAX,
            expiration: i64::MAX,
            preimage_hex: concat!(
                "ffffffffffffffff",
                "03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8",
                "1111111111111111111111111111111111111111111111111111111111111111",
                "2222222222222222222222222222222222222222222222222222222222222222",
                "3333333333333333333333333333333333333333333333333333333333333333",
                "01ffffffffffffffffffffffffffffffffffffffffffffff7f",
            ),
            order_id_hex: "f51ef152121aeb7380e5fd0e0f5574382fc9e1bda5cecff7b98d8ff938ad0dc2",
            signature_hex: concat!(
                "a03310e35a4dbae73b111de3ffb233ffdf6b358fb2023ee5cd88989a8f8e92c7",
                "6eee9d37042eb3028e31cd6f17878c927b8a3da5598f2736e8ddd69a84c77a01",
            ),
            compact_hex: "ffffffffffffffff01ffffffffffffffffffffffffffffffffffffffffffffff7f",
        },
    ];

    #[test]
    fn signing_vectors_match_program_contract() {
        let names: Vec<&str> = SIGNING_VECTORS.iter().map(|v| v.name).collect();
        assert_eq!(names, ["bid", "ask", "zero_salt", "max_salt"]);
        let maker: [u8; 32] = hex::decode(SIGNING_MAKER_HEX).unwrap().try_into().unwrap();

        for vector in &SIGNING_VECTORS {
            let name = vector.name;
            let unsigned = OrderPayload {
                salt: vector.salt,
                maker: Pubkey::new_from_array(maker),
                market: Pubkey::new_from_array([0x11; 32]),
                base_mint: Pubkey::new_from_array([0x22; 32]),
                quote_mint: Pubkey::new_from_array([0x33; 32]),
                side: vector.side,
                amount_in: vector.maker_amount,
                amount_out: vector.taker_amount,
                expiration: vector.expiration,
                signature: [0; 64],
            };

            assert_eq!(
                hex::encode(unsigned.preimage()),
                vector.preimage_hex,
                "{name}: preimage"
            );
            // The fixture's signed message (`message_ascii`) equals `order_id_hex`
            // in every vector: makers sign the order id's 64 lowercase hex ASCII
            // bytes, which `verify_signature` checks below.
            assert_eq!(
                unsigned.hash_hex(),
                vector.order_id_hex,
                "{name}: order id and signed message"
            );
            assert_eq!(
                hex::encode(unsigned.to_order().serialize()),
                vector.compact_hex,
                "{name}: compact order"
            );

            let mut signed = unsigned.clone();
            signed
                .signature
                .copy_from_slice(&hex::decode(vector.signature_hex).unwrap());
            signed.verify_signature().unwrap();
            let signed_bytes = signed.serialize();
            // The fixture's 225-byte `signed_order_hex` is the preimage
            // followed by the signature.
            assert_eq!(
                hex::encode(signed_bytes),
                format!("{}{}", vector.preimage_hex, vector.signature_hex),
                "{name}: signed order"
            );

            let decoded = OrderPayload::deserialize(&signed_bytes).unwrap();
            assert_eq!(decoded.serialize(), signed_bytes, "{name}: signed decode");
            let compact = Order::deserialize(&hex::decode(vector.compact_hex).unwrap()).unwrap();
            let expanded = compact.to_signed(
                signed.maker,
                signed.market,
                signed.base_mint,
                signed.quote_mint,
                signed.signature,
            );
            assert_eq!(expanded.serialize(), signed_bytes, "{name}: compact decode");

            #[cfg(feature = "native-auth")]
            {
                let seed: [u8; 32] = hex::decode(SIGNING_SEED_HEX).unwrap().try_into().unwrap();
                let keypair = solana_keypair::Keypair::new_from_array(seed);
                assert_eq!(keypair.pubkey(), unsigned.maker, "{name}: maker key");
                let mut resigned = unsigned.clone();
                resigned.sign(&keypair, &signing_rules()).unwrap();
                assert_eq!(
                    resigned.signature_hex(),
                    vector.signature_hex,
                    "{name}: signature"
                );
            }
        }
    }

    #[test]
    fn serialized_order_layouts_require_exact_lengths() {
        assert_eq!(OrderPayload::HASH_SIZE, 161);
        assert_eq!(OrderPayload::LEN, 225);
        assert_eq!(Order::LEN, 33);
        for length in [224, 226] {
            assert!(matches!(
                OrderPayload::deserialize(&vec![0; length]),
                Err(SdkError::InvalidDataLength {
                    expected: 225,
                    actual,
                }) if actual == length
            ));
        }
        for length in [32, 34] {
            assert!(matches!(
                Order::deserialize(&vec![0; length]),
                Err(SdkError::InvalidDataLength {
                    expected: 33,
                    actual,
                }) if actual == length
            ));
        }
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn signing_preflights_amounts_against_rules() {
        let keypair = Keypair::new();
        let mut order = OrderPayload {
            salt: u64::MAX,
            maker: keypair.pubkey(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            amount_in: 1,
            amount_out: 3_000,
            expiration: 0,
            signature: [0; 64],
        };
        assert!(matches!(
            order.sign(&keypair, &signing_rules()),
            Err(SdkError::Scaling(
                crate::shared::scaling::ScalingError::PriceNotExactlyRepresentable
            ))
        ));
        order.amount_in = 0;
        assert!(matches!(
            order.sign(&keypair, &signing_rules()),
            Err(SdkError::Scaling(
                crate::shared::scaling::ScalingError::OrderFieldOutOfRange { field: "amount_in" }
            ))
        ));
        assert!(!order.is_signed());
    }

    #[test]
    fn test_order_payload_serialization_roundtrip() {
        let order = OrderPayload {
            salt: 12345,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            amount_in: 1000000,
            amount_out: 500000,
            expiration: 1234567890,
            signature: [0u8; 64],
        };

        let serialized = order.serialize();
        let deserialized = OrderPayload::deserialize(&serialized).unwrap();

        assert_eq!(order.salt, deserialized.salt);
        assert_eq!(order.maker, deserialized.maker);
        assert_eq!(order.market, deserialized.market);
        assert_eq!(order.base_mint, deserialized.base_mint);
        assert_eq!(order.quote_mint, deserialized.quote_mint);
        assert_eq!(order.side, deserialized.side);
        assert_eq!(order.amount_in, deserialized.amount_in);
        assert_eq!(order.amount_out, deserialized.amount_out);
        assert_eq!(order.expiration, deserialized.expiration);
    }

    #[test]
    fn test_order_serialization_roundtrip() {
        let order = Order {
            salt: 12345,
            side: OrderSide::Ask,
            amount_in: 1000000,
            amount_out: 500000,
            expiration: 1234567890,
        };

        let serialized = order.serialize();
        let deserialized = Order::deserialize(&serialized).unwrap();

        assert_eq!(order.salt, deserialized.salt);
        assert_eq!(order.side, deserialized.side);
        assert_eq!(order.amount_in, deserialized.amount_in);
        assert_eq!(order.amount_out, deserialized.amount_out);
        assert_eq!(order.expiration, deserialized.expiration);
    }

    #[test]
    fn test_order_size() {
        assert_eq!(ORDER_SIZE, 33);
        let order = Order {
            salt: 1,
            side: OrderSide::Bid,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
        };
        assert_eq!(order.serialize().len(), 33);
    }

    #[test]
    fn test_order_hash_consistency() {
        let order = OrderPayload {
            salt: 1,
            maker: Pubkey::new_from_array([1u8; 32]),
            market: Pubkey::new_from_array([2u8; 32]),
            base_mint: Pubkey::new_from_array([3u8; 32]),
            quote_mint: Pubkey::new_from_array([4u8; 32]),
            side: OrderSide::Bid,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: [0u8; 64],
        };

        let hash1 = order.hash();
        let hash2 = order.hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_signed_order_to_order_roundtrip() {
        let signed = OrderPayload {
            salt: 42,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            amount_in: 1000,
            amount_out: 500,
            expiration: 12345,
            signature: [7u8; 64],
        };

        let order = signed.to_order();
        assert_eq!(order.salt, 42);
        assert_eq!(order.side, OrderSide::Bid);
        assert_eq!(order.amount_in, 1000);
        assert_eq!(order.amount_out, 500);
        assert_eq!(order.expiration, 12345);

        let back = order.to_signed(
            signed.maker,
            signed.market,
            signed.base_mint,
            signed.quote_mint,
            signed.signature,
        );
        assert_eq!(back.salt, 42);
        assert_eq!(back.maker, signed.maker);
        assert_eq!(back.amount_in, 1000);
    }

    #[test]
    fn test_orders_can_cross() {
        let buy_order = OrderPayload {
            salt: 1,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            amount_in: 100, // 100 quote
            amount_out: 50, // for 50 base (price = 2 quote/base)
            expiration: 0,
            signature: [0u8; 64],
        };

        let sell_order = OrderPayload {
            salt: 2,
            maker: Pubkey::new_unique(),
            market: buy_order.market,
            base_mint: buy_order.base_mint,
            quote_mint: buy_order.quote_mint,
            side: OrderSide::Ask,
            amount_in: 50,  // 50 base
            amount_out: 90, // for 90 quote (price = 1.8 quote/base)
            expiration: 0,
            signature: [0u8; 64],
        };

        // Buyer pays 2 quote/base, seller wants 1.8 quote/base - should cross
        assert!(orders_can_cross(&buy_order, &sell_order));
    }

    #[test]
    fn test_orders_cannot_cross() {
        let buy_order = OrderPayload {
            salt: 1,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            amount_in: 50,  // 50 quote
            amount_out: 50, // for 50 base (price = 1 quote/base)
            expiration: 0,
            signature: [0u8; 64],
        };

        let sell_order = OrderPayload {
            salt: 2,
            maker: Pubkey::new_unique(),
            market: buy_order.market,
            base_mint: buy_order.base_mint,
            quote_mint: buy_order.quote_mint,
            side: OrderSide::Ask,
            amount_in: 50,   // 50 base
            amount_out: 100, // for 100 quote (price = 2 quote/base)
            expiration: 0,
            signature: [0u8; 64],
        };

        // Buyer pays 1 quote/base, seller wants 2 quote/base - should not cross
        assert!(!orders_can_cross(&buy_order, &sell_order));
    }

    #[test]
    fn test_calculate_taker_fill() {
        let maker_order = OrderPayload {
            salt: 1,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Ask,
            amount_in: 100,  // gives 100 base
            amount_out: 200, // wants 200 quote
            expiration: 0,
            signature: [0u8; 64],
        };

        // If filling 50 amount_in, taker should get 50 * 200 / 100 = 100
        let taker_fill = calculate_taker_fill(&maker_order, 50).unwrap();
        assert_eq!(taker_fill, 100);
        // A fractional minimum rounds up, as the program's price check requires:
        // ceil(1 * 200 / 100) = 2 and ceil(3 * 200 / 101) = ceil(5.94) = 6.
        assert_eq!(calculate_taker_fill(&maker_order, 1).unwrap(), 2);
        let odd_maker = OrderPayload {
            amount_in: 101,
            ..maker_order.clone()
        };
        assert_eq!(calculate_taker_fill(&odd_maker, 3).unwrap(), 6);
        let overflowing = OrderPayload {
            amount_in: 1,
            amount_out: u64::MAX,
            ..maker_order
        };
        assert!(matches!(
            calculate_taker_fill(&overflowing, 2),
            Err(SdkError::Overflow)
        ));
    }

    #[test]
    fn order_expires_only_after_its_expiration_second() {
        let mut order = OrderPayload {
            salt: 0,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: [0u8; 64],
        };
        assert!(!is_order_expired(&order, i64::MAX));

        order.expiration = 1_700_000_000;
        assert!(!is_order_expired(&order, 1_699_999_999));
        assert!(!is_order_expired(&order, 1_700_000_000));
        assert!(is_order_expired(&order, 1_700_000_001));
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_to_submit_request() {
        use solana_keypair::Keypair;
        use solana_signer::Signer;

        let keypair = Keypair::new();
        let maker = keypair.pubkey();
        let market = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let mut order = OrderPayload {
            salt: 42,
            maker,
            market,
            base_mint,
            quote_mint,
            side: OrderSide::Bid,
            amount_in: 1_000_000,
            amount_out: 500_000,
            expiration: 1234567890,
            signature: [0u8; 64],
        };

        order.sign(&keypair, &signing_rules()).unwrap();

        let request = order
            .to_submit_request("test_orderbook", None, None)
            .unwrap();

        assert_eq!(request.maker, maker.to_string());
        assert_eq!(request.salt, 42);
        assert_eq!(request.market_pubkey, market.to_string());
        assert_eq!(request.base_token, base_mint.to_string());
        assert_eq!(request.quote_token, quote_mint.to_string());
        assert_eq!(request.side, 0); // Bid
        assert_eq!(request.amount_in, 1_000_000);
        assert_eq!(request.amount_out, 500_000);
        assert_eq!(request.expiration, 1234567890);
        assert_eq!(request.orderbook_id, "test_orderbook");
        assert_eq!(request.signature.len(), 128); // 64 bytes = 128 hex chars
    }

    #[test]
    fn test_derive_orderbook_id() {
        let order = OrderPayload {
            salt: 1,
            maker: Pubkey::new_from_array([1u8; 32]),
            market: Pubkey::new_from_array([2u8; 32]),
            base_mint: Pubkey::new_from_array([3u8; 32]),
            quote_mint: Pubkey::new_from_array([4u8; 32]),
            side: OrderSide::Bid,
            amount_in: 100,
            amount_out: 50,
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
    #[cfg(feature = "native-auth")]
    fn test_is_signed() {
        use solana_keypair::Keypair;
        use solana_signer::Signer;

        let keypair = Keypair::new();
        let mut order = OrderPayload {
            salt: 1,
            maker: keypair.pubkey(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: [0u8; 64],
        };

        assert!(!order.is_signed());

        order.sign(&keypair, &signing_rules()).unwrap();

        assert!(order.is_signed());
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_signature_and_hash_hex() {
        use solana_keypair::Keypair;
        use solana_signer::Signer;

        let keypair = Keypair::new();
        let mut order = OrderPayload {
            salt: 1,
            maker: keypair.pubkey(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: [0u8; 64],
        };

        order.sign(&keypair, &signing_rules()).unwrap();

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
    fn test_to_submit_request_errors_unsigned() {
        let order = OrderPayload {
            salt: 1,
            maker: Pubkey::new_unique(),
            market: Pubkey::new_unique(),
            base_mint: Pubkey::new_unique(),
            quote_mint: Pubkey::new_unique(),
            side: OrderSide::Bid,
            amount_in: 100,
            amount_out: 50,
            expiration: 0,
            signature: [0u8; 64],
        };

        let result = order.to_submit_request("test_orderbook", None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be signed"),);
    }

    #[test]
    #[cfg(feature = "trigger_orders")]
    fn test_cancel_trigger_order_message() {
        let id = "trigger-order-uuid-123";
        let message = cancel_trigger_order_message(id);
        assert_eq!(message, id.as_bytes());
    }

    #[test]
    fn test_cancel_order_message() {
        let hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let message = cancel_order_message(hash);
        assert_eq!(message, hash.as_bytes());
    }

    #[test]
    fn test_cancel_all_message() {
        let pubkey = "SomePubkey123";
        let orderbook_id = "test_orderbook";
        let timestamp = 1700000000i64;
        let salt = "550e8400-e29b-41d4-a716-446655440000";
        let message = cancel_all_message(pubkey, orderbook_id, timestamp, salt);
        assert_eq!(
            message,
            "cancel_all:SomePubkey123:test_orderbook:1700000000:550e8400-e29b-41d4-a716-446655440000"
        );
    }

    #[test]
    fn test_generate_cancel_all_salt() {
        let salt = generate_cancel_all_salt();
        assert_eq!(salt.len(), 36);
        assert_eq!(salt.chars().filter(|c| *c == '-').count(), 4);
        assert_eq!(salt.chars().nth(14), Some('4'));
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_cancel_body_signed() {
        use crate::domain::order::CancelBody;
        use solana_keypair::Keypair;
        use solana_signer::Signer;

        let keypair = Keypair::new();
        let order_hash = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        let maker = crate::shared::PubkeyStr::from_pubkey(keypair.pubkey());

        let body = CancelBody::signed(order_hash.to_string(), maker, &keypair);
        assert_eq!(body.signature.len(), 128);
        assert_eq!(body.order_hash, order_hash);

        let sig_bytes = hex::decode(&body.signature).unwrap();
        let sig = Signature::try_from(sig_bytes.as_slice()).unwrap();
        assert!(sig.verify(keypair.pubkey().as_ref(), order_hash.as_bytes()));
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_cancel_all_body_signed() {
        use crate::domain::order::CancelAllBody;
        use solana_keypair::Keypair;
        use solana_signer::Signer;

        let keypair = Keypair::new();
        let pubkey_str = crate::shared::PubkeyStr::from_pubkey(keypair.pubkey());
        let orderbook_id = crate::shared::OrderBookId::from("");
        let timestamp = 1700000000i64;
        let salt = "550e8400-e29b-41d4-a716-446655440000".to_string();

        let body = CancelAllBody::signed(
            pubkey_str.clone(),
            orderbook_id.clone(),
            timestamp,
            salt.clone(),
            &keypair,
        );
        assert_eq!(body.signature.len(), 128);
        assert_eq!(body.salt, salt);

        let message =
            cancel_all_message(pubkey_str.as_str(), orderbook_id.as_str(), timestamp, &salt);
        let sig_bytes = hex::decode(&body.signature).unwrap();
        let sig = Signature::try_from(sig_bytes.as_slice()).unwrap();
        assert!(sig.verify(keypair.pubkey().as_ref(), message.as_bytes()));
    }
}
