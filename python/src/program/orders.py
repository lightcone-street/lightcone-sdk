"""Order creation, hashing, signing, and serialization for the Lightcone SDK."""

import time

import nacl.exceptions
from nacl.signing import SigningKey, VerifyKey
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from .constants import (
    COMPACT_ORDER_SIZE,
    FULL_ORDER_SIZE,
    ORDER_BASE_MINT_OFFSET,
    ORDER_EXPIRATION_OFFSET,
    ORDER_HASH_SIZE,
    ORDER_MAKER_AMOUNT_OFFSET,
    ORDER_MAKER_OFFSET,
    ORDER_MARKET_OFFSET,
    ORDER_NONCE_OFFSET,
    ORDER_QUOTE_MINT_OFFSET,
    ORDER_SIDE_OFFSET,
    ORDER_SIGNATURE_OFFSET,
    ORDER_TAKER_AMOUNT_OFFSET,
    SIGNATURE_SIZE,
)
from .errors import InvalidOrderError, InvalidSignatureError
from .types import (
    AskOrderParams,
    BidOrderParams,
    CompactOrder,
    FullOrder,
    OrderSide,
)
from .utils import (
    decode_i64,
    decode_pubkey,
    decode_u64,
    decode_u8,
    encode_i64,
    encode_u64,
    encode_u8,
    keccak256,
)


def create_bid_order(params: BidOrderParams) -> FullOrder:
    """Create a bid order (buyer wants base tokens, gives quote tokens).

    The signature field is left empty (64 zero bytes).
    """
    return FullOrder(
        nonce=params.nonce,
        maker=params.maker,
        market=params.market,
        base_mint=params.base_mint,
        quote_mint=params.quote_mint,
        side=OrderSide.BID,
        maker_amount=params.maker_amount,
        taker_amount=params.taker_amount,
        expiration=params.expiration,
        signature=bytes(SIGNATURE_SIZE),
    )


def create_ask_order(params: AskOrderParams) -> FullOrder:
    """Create an ask order (seller offers base tokens, receives quote tokens).

    The signature field is left empty (64 zero bytes).
    """
    return FullOrder(
        nonce=params.nonce,
        maker=params.maker,
        market=params.market,
        base_mint=params.base_mint,
        quote_mint=params.quote_mint,
        side=OrderSide.ASK,
        maker_amount=params.maker_amount,
        taker_amount=params.taker_amount,
        expiration=params.expiration,
        signature=bytes(SIGNATURE_SIZE),
    )


def serialize_order_for_hashing(order: FullOrder) -> bytes:
    """Serialize an order for hashing (excludes signature).

    Layout (161 bytes):
    - nonce (8) | maker (32) | market (32) | base_mint (32) | quote_mint (32) |
      side (1) | maker_amount (8) | taker_amount (8) | expiration (8)
    """
    return (
        encode_u64(order.nonce)
        + bytes(order.maker)
        + bytes(order.market)
        + bytes(order.base_mint)
        + bytes(order.quote_mint)
        + encode_u8(order.side)
        + encode_u64(order.maker_amount)
        + encode_u64(order.taker_amount)
        + encode_i64(order.expiration)
    )


def hash_order(order: FullOrder) -> bytes:
    """Compute the keccak256 hash of an order.

    Returns a 32-byte hash.
    """
    data = serialize_order_for_hashing(order)
    return keccak256(data)


def sign_order(order: FullOrder, keypair: Keypair) -> bytes:
    """Sign an order with a keypair.

    Signs the keccak256 hash of the order with the keypair's Ed25519 private key.
    Updates the order's signature in place and returns the signature.
    """
    order_hash = hash_order(order)

    # Extract the 32-byte seed from the keypair's secret key
    # solders Keypair stores the full 64-byte secret (seed + public key)
    secret_bytes = bytes(keypair)
    seed = secret_bytes[:32]

    # Create a nacl signing key and sign the hash
    signing_key = SigningKey(seed)
    signed = signing_key.sign(order_hash)
    signature = signed.signature

    # Update the order's signature
    order.signature = signature
    return signature


def verify_order_signature(order: FullOrder) -> bool:
    """Verify the Ed25519 signature on an order.

    Returns True if the signature is valid, False otherwise.

    Raises:
        InvalidOrderError: If the maker public key is invalid
    """
    if len(order.signature) != SIGNATURE_SIZE:
        return False

    order_hash = hash_order(order)

    try:
        # Get verify key from maker pubkey
        verify_key = VerifyKey(bytes(order.maker))
        verify_key.verify(order_hash, order.signature)
        return True
    except nacl.exceptions.BadSignatureError:
        return False
    except (nacl.exceptions.ValueError, ValueError) as e:
        raise InvalidOrderError(f"Invalid maker public key: {e}")


def serialize_full_order(order: FullOrder) -> bytes:
    """Serialize a full order to bytes.

    Layout (225 bytes):
    - nonce (8) | maker (32) | market (32) | base_mint (32) | quote_mint (32) |
      side (1) | maker_amount (8) | taker_amount (8) | expiration (8) | signature (64)
    """
    return serialize_order_for_hashing(order) + order.signature


def deserialize_full_order(data: bytes) -> FullOrder:
    """Deserialize a full order from bytes."""
    if len(data) < FULL_ORDER_SIZE:
        raise InvalidOrderError(
            f"Data too short: {len(data)} bytes (expected {FULL_ORDER_SIZE})"
        )

    return FullOrder(
        nonce=decode_u64(data, ORDER_NONCE_OFFSET),
        maker=decode_pubkey(data, ORDER_MAKER_OFFSET),
        market=decode_pubkey(data, ORDER_MARKET_OFFSET),
        base_mint=decode_pubkey(data, ORDER_BASE_MINT_OFFSET),
        quote_mint=decode_pubkey(data, ORDER_QUOTE_MINT_OFFSET),
        side=OrderSide(decode_u8(data, ORDER_SIDE_OFFSET)),
        maker_amount=decode_u64(data, ORDER_MAKER_AMOUNT_OFFSET),
        taker_amount=decode_u64(data, ORDER_TAKER_AMOUNT_OFFSET),
        expiration=decode_i64(data, ORDER_EXPIRATION_OFFSET),
        signature=data[ORDER_SIGNATURE_OFFSET : ORDER_SIGNATURE_OFFSET + SIGNATURE_SIZE],
    )


def to_compact_order(order: FullOrder) -> CompactOrder:
    """Convert a full order to a compact order (excludes market/mints)."""
    return CompactOrder(
        nonce=order.nonce,
        maker=order.maker,
        side=order.side,
        maker_amount=order.maker_amount,
        taker_amount=order.taker_amount,
        expiration=order.expiration,
    )


def serialize_compact_order(order: CompactOrder) -> bytes:
    """Serialize a compact order to bytes.

    Layout (65 bytes):
    - nonce (8) | maker (32) | side (1) | maker_amount (8) | taker_amount (8) | expiration (8)
    """
    return (
        encode_u64(order.nonce)
        + bytes(order.maker)
        + encode_u8(order.side)
        + encode_u64(order.maker_amount)
        + encode_u64(order.taker_amount)
        + encode_i64(order.expiration)
    )


def deserialize_compact_order(data: bytes) -> CompactOrder:
    """Deserialize a compact order from bytes."""
    if len(data) < COMPACT_ORDER_SIZE:
        raise InvalidOrderError(
            f"Data too short: {len(data)} bytes (expected {COMPACT_ORDER_SIZE})"
        )

    return CompactOrder(
        nonce=decode_u64(data, 0),
        maker=decode_pubkey(data, 8),
        side=OrderSide(decode_u8(data, 40)),
        maker_amount=decode_u64(data, 41),
        taker_amount=decode_u64(data, 49),
        expiration=decode_i64(data, 57),
    )


def create_signed_bid_order(params: BidOrderParams, keypair: Keypair) -> FullOrder:
    """Create and sign a bid order in one call."""
    order = create_bid_order(params)
    sign_order(order, keypair)
    return order


def create_signed_ask_order(params: AskOrderParams, keypair: Keypair) -> FullOrder:
    """Create and sign an ask order in one call."""
    order = create_ask_order(params)
    sign_order(order, keypair)
    return order


def validate_order(order: FullOrder, check_expiration: bool = False) -> None:
    """Validate an order's fields.

    Args:
        order: The order to validate
        check_expiration: If True, also check that the order hasn't expired

    Raises InvalidOrderError if any field is invalid.
    """
    # Validate amounts
    if order.maker_amount == 0:
        raise InvalidOrderError("maker_amount cannot be zero")
    if order.taker_amount == 0:
        raise InvalidOrderError("taker_amount cannot be zero")

    # Validate u64 bounds
    max_u64 = 2**64 - 1
    if order.maker_amount > max_u64:
        raise InvalidOrderError(f"maker_amount exceeds u64 max: {order.maker_amount}")
    if order.taker_amount > max_u64:
        raise InvalidOrderError(f"taker_amount exceeds u64 max: {order.taker_amount}")

    # Validate side
    if order.side not in (OrderSide.BID, OrderSide.ASK):
        raise InvalidOrderError(f"Invalid side: {order.side}")

    # Validate expiration (if set and check enabled, must be in the future)
    if check_expiration and order.expiration != 0 and order.expiration < int(time.time()):
        raise InvalidOrderError(f"Order already expired: expiration={order.expiration}")

    # Validate maker is not zero pubkey
    if bytes(order.maker) == bytes(32):
        raise InvalidOrderError("maker cannot be zero pubkey")


def validate_signed_order(order: FullOrder) -> None:
    """Validate an order including its signature.

    Raises InvalidOrderError or InvalidSignatureError if invalid.
    """
    validate_order(order)

    if not verify_order_signature(order):
        raise InvalidSignatureError("Order signature verification failed")
