"""Order creation, hashing, signing, and serialization for the Lightcone SDK."""

import time

import nacl.exceptions
from nacl.signing import SigningKey, VerifyKey
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from .constants import (
    ORDER_SIZE,
    SIGNED_ORDER_SIZE,
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
    # Backward compat
    FULL_ORDER_SIZE,
    COMPACT_ORDER_SIZE,
)
from .errors import InvalidOrderError, InvalidSignatureError
from .types import (
    AskOrderParams,
    BidOrderParams,
    Order,
    FullOrder,
    OrderSide,
)
from .utils import (
    decode_i64,
    decode_pubkey,
    decode_u32,
    decode_u64,
    decode_u8,
    encode_i64,
    encode_u32,
    encode_u64,
    encode_u8,
    keccak256,
)

# Maximum value for a u64 integer
MAX_U64 = 2**64 - 1
# Maximum value for a u32 integer
MAX_U32 = 2**32 - 1


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
    - nonce (8, u32 value widened to u64 LE) | maker (32) | market (32) |
      base_mint (32) | quote_mint (32) | side (1) | maker_amount (8) |
      taker_amount (8) | expiration (8)
    """
    if order.nonce > MAX_U32:
        raise InvalidOrderError(f"nonce exceeds u32 max: {order.nonce}")
    return (
        encode_u64(order.nonce)  # Widen u32 to u64 for wire compatibility
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


def hash_order_hex(order: FullOrder) -> str:
    """Compute the keccak256 hash of an order and return as a 64-char hex string."""
    return hash_order(order).hex()


def sign_order(order: FullOrder, keypair: Keypair) -> bytes:
    """Sign an order with a keypair.

    Signs the hex-encoded keccak256 hash of the order (64-char ASCII string)
    with the keypair's Ed25519 private key. Updates the order's signature in
    place and returns the signature.
    """
    order_hash_hex = hash_order_hex(order)
    message = order_hash_hex.encode("ascii")

    # Extract the 32-byte seed from the keypair's secret key
    # solders Keypair stores the full 64-byte secret (seed + public key)
    secret_bytes = bytes(keypair)
    seed = secret_bytes[:32]

    # Create a nacl signing key and sign the hex-encoded hash
    signing_key = SigningKey(seed)
    signed = signing_key.sign(message)
    signature = signed.signature

    # Update the order's signature
    order.signature = signature
    return signature


def verify_order_signature(order: FullOrder) -> bool:
    """Verify the Ed25519 signature on an order.

    Returns True if the signature is valid, False otherwise.
    Verifies against the hex-encoded keccak256 hash (64-char ASCII string).

    Raises:
        InvalidOrderError: If the maker public key is invalid
    """
    if len(order.signature) != SIGNATURE_SIZE:
        return False

    order_hash_hex = hash_order_hex(order)
    message = order_hash_hex.encode("ascii")

    try:
        # Get verify key from maker pubkey
        verify_key = VerifyKey(bytes(order.maker))
        verify_key.verify(message, order.signature)
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
    if len(data) < SIGNED_ORDER_SIZE:
        raise InvalidOrderError(
            f"Data too short: {len(data)} bytes (expected {SIGNED_ORDER_SIZE})"
        )

    nonce_u64 = decode_u64(data, ORDER_NONCE_OFFSET)
    if nonce_u64 > MAX_U32:
        raise InvalidOrderError(f"nonce exceeds u32 max: {nonce_u64}")

    return FullOrder(
        nonce=nonce_u64,
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


def to_order(order: FullOrder) -> Order:
    """Convert a full order to a compact order (29 bytes, no maker, u32 nonce)."""
    return Order(
        nonce=order.nonce,
        side=order.side,
        maker_amount=order.maker_amount,
        taker_amount=order.taker_amount,
        expiration=order.expiration,
    )


# Backward compatibility alias
to_compact_order = to_order


def serialize_order(order: Order) -> bytes:
    """Serialize a compact order to bytes.

    Layout (29 bytes):
    - nonce (4, u32) | side (1) | maker_amount (8) | taker_amount (8) | expiration (8)
    """
    return (
        encode_u32(order.nonce)
        + encode_u8(order.side)
        + encode_u64(order.maker_amount)
        + encode_u64(order.taker_amount)
        + encode_i64(order.expiration)
    )


# Backward compatibility alias
serialize_compact_order = serialize_order


def deserialize_order(data: bytes) -> Order:
    """Deserialize a compact order from bytes."""
    if len(data) < ORDER_SIZE:
        raise InvalidOrderError(
            f"Data too short: {len(data)} bytes (expected {ORDER_SIZE})"
        )

    return Order(
        nonce=decode_u32(data, 0),
        side=OrderSide(decode_u8(data, 4)),
        maker_amount=decode_u64(data, 5),
        taker_amount=decode_u64(data, 13),
        expiration=decode_i64(data, 21),
    )


# Backward compatibility alias
deserialize_compact_order = deserialize_order


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
    # Validate nonce range (u32)
    if order.nonce > MAX_U32:
        raise InvalidOrderError(f"nonce exceeds u32 max: {order.nonce}")

    # Validate amounts
    if order.maker_amount == 0:
        raise InvalidOrderError("maker_amount cannot be zero")
    if order.taker_amount == 0:
        raise InvalidOrderError("taker_amount cannot be zero")

    # Validate u64 bounds
    if order.maker_amount > MAX_U64:
        raise InvalidOrderError(f"maker_amount exceeds u64 max: {order.maker_amount}")
    if order.taker_amount > MAX_U64:
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


# =========================================================================
# Cancel Order Signing Helpers
# =========================================================================


def cancel_order_message(order_hash: str) -> bytes:
    """Build the message bytes for cancelling an order.

    The message is the order hash hex string as ASCII bytes
    (same protocol as order signing).
    """
    return order_hash.encode("ascii")


def sign_cancel_order(order_hash: str, keypair: Keypair) -> str:
    """Sign a cancel order request.

    Returns the signature as a 128-char hex string.
    """
    message = cancel_order_message(order_hash)

    secret_bytes = bytes(keypair)
    seed = secret_bytes[:32]

    signing_key = SigningKey(seed)
    signed = signing_key.sign(message)
    return signed.signature.hex()


def cancel_all_message(user_pubkey: str, timestamp: int) -> str:
    """Build the message string for cancelling all orders.

    Format: "cancel_all:{pubkey}:{timestamp}"
    """
    return f"cancel_all:{user_pubkey}:{timestamp}"


def sign_cancel_all(user_pubkey: str, timestamp: int, keypair: Keypair) -> str:
    """Sign a cancel-all orders request.

    Returns the signature as a 128-char hex string.
    """
    message = cancel_all_message(user_pubkey, timestamp)
    message_bytes = message.encode("ascii")

    secret_bytes = bytes(keypair)
    seed = secret_bytes[:32]

    signing_key = SigningKey(seed)
    signed = signing_key.sign(message_bytes)
    return signed.signature.hex()


# =========================================================================
# Bridge Methods (order <-> API)
# =========================================================================


def signature_hex(order: FullOrder) -> str:
    """Return the order signature as a 128-char hex string."""
    return order.signature.hex()


def is_signed(order: FullOrder) -> bool:
    """Check if an order has a non-zero signature."""
    return order.signature != bytes(SIGNATURE_SIZE)


def to_submit_request(order: FullOrder, orderbook_id: str):
    """Convert a signed FullOrder to a SubmitOrderRequest.

    Args:
        order: A signed FullOrder
        orderbook_id: The orderbook identifier

    Returns:
        SubmitOrderRequest ready for API submission

    Raises:
        InvalidOrderError: If the order is not signed
    """
    if not is_signed(order):
        raise InvalidOrderError("Order must be signed before submitting")

    # Import here to avoid circular dependency
    from ..api.types.order import SubmitOrderRequest

    return SubmitOrderRequest(
        maker=str(order.maker),
        nonce=order.nonce,
        market_pubkey=str(order.market),
        base_token=str(order.base_mint),
        quote_token=str(order.quote_mint),
        side=int(order.side),
        maker_amount=order.maker_amount,
        taker_amount=order.taker_amount,
        expiration=order.expiration,
        signature=signature_hex(order),
        orderbook_id=orderbook_id,
    )
