"""Order creation, hashing, signing, and serialization for the Lightcone SDK."""

import time
import uuid

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
    SignedOrder,
    OrderSide,
)
from .utils import (
    decode_i64,
    decode_pubkey,
    decode_u32,
    decode_u64,
    decode_u8,
    derive_condition_id,
    encode_i64,
    encode_u32,
    encode_u64,
    encode_u8,
    keccak256,
    orders_cross,
)

# Backward compatibility alias
FullOrder = SignedOrder

# Maximum value for a u64 integer
MAX_U64 = 2**64 - 1
# Maximum value for a u32 integer
MAX_U32 = 2**32 - 1


def create_bid_order(params: BidOrderParams) -> SignedOrder:
    """Create a bid order (buyer wants base tokens, gives quote tokens).

    The signature field is left empty (64 zero bytes).
    """
    return SignedOrder(
        nonce=params.nonce,
        maker=params.maker,
        market=params.market,
        base_mint=params.base_mint,
        quote_mint=params.quote_mint,
        side=OrderSide.BID,
        amount_in=params.amount_in,
        amount_out=params.amount_out,
        expiration=params.expiration,
        signature=bytes(SIGNATURE_SIZE),
    )


def create_ask_order(params: AskOrderParams) -> SignedOrder:
    """Create an ask order (seller offers base tokens, receives quote tokens).

    The signature field is left empty (64 zero bytes).
    """
    return SignedOrder(
        nonce=params.nonce,
        maker=params.maker,
        market=params.market,
        base_mint=params.base_mint,
        quote_mint=params.quote_mint,
        side=OrderSide.ASK,
        amount_in=params.amount_in,
        amount_out=params.amount_out,
        expiration=params.expiration,
        signature=bytes(SIGNATURE_SIZE),
    )


def serialize_order_for_hashing(order: SignedOrder) -> bytes:
    """Serialize an order for hashing (excludes signature).

    Layout (161 bytes):
    - nonce (8, u32 value widened to u64 LE) | maker (32) | market (32) |
      base_mint (32) | quote_mint (32) | side (1) | amount_in (8) |
      amount_out (8) | expiration (8)
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
        + encode_u64(order.amount_in)
        + encode_u64(order.amount_out)
        + encode_i64(order.expiration)
    )


def hash_order(order: SignedOrder) -> bytes:
    """Compute the keccak256 hash of an order.

    Returns a 32-byte hash.
    """
    data = serialize_order_for_hashing(order)
    return keccak256(data)


def hash_order_hex(order: SignedOrder) -> str:
    """Compute the keccak256 hash of an order and return as a 64-char hex string."""
    return hash_order(order).hex()


def sign_order(order: SignedOrder, keypair: Keypair) -> bytes:
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


def verify_order_signature(order: SignedOrder) -> bool:
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


def serialize_full_order(order: SignedOrder) -> bytes:
    """Serialize a full order to bytes.

    Layout (225 bytes):
    - nonce (8) | maker (32) | market (32) | base_mint (32) | quote_mint (32) |
      side (1) | amount_in (8) | amount_out (8) | expiration (8) | signature (64)
    """
    return serialize_order_for_hashing(order) + order.signature


def deserialize_full_order(data: bytes) -> SignedOrder:
    """Deserialize a full order from bytes."""
    if len(data) < SIGNED_ORDER_SIZE:
        raise InvalidOrderError(
            f"Data too short: {len(data)} bytes (expected {SIGNED_ORDER_SIZE})"
        )

    nonce_u64 = decode_u64(data, ORDER_NONCE_OFFSET)
    if nonce_u64 > MAX_U32:
        raise InvalidOrderError(f"nonce exceeds u32 max: {nonce_u64}")

    return SignedOrder(
        nonce=nonce_u64,
        maker=decode_pubkey(data, ORDER_MAKER_OFFSET),
        market=decode_pubkey(data, ORDER_MARKET_OFFSET),
        base_mint=decode_pubkey(data, ORDER_BASE_MINT_OFFSET),
        quote_mint=decode_pubkey(data, ORDER_QUOTE_MINT_OFFSET),
        side=OrderSide(decode_u8(data, ORDER_SIDE_OFFSET)),
        amount_in=decode_u64(data, ORDER_MAKER_AMOUNT_OFFSET),
        amount_out=decode_u64(data, ORDER_TAKER_AMOUNT_OFFSET),
        expiration=decode_i64(data, ORDER_EXPIRATION_OFFSET),
        signature=data[ORDER_SIGNATURE_OFFSET : ORDER_SIGNATURE_OFFSET + SIGNATURE_SIZE],
    )


def to_order(order: SignedOrder) -> Order:
    """Convert a full order to a compact order (29 bytes, no maker, u32 nonce)."""
    return Order(
        nonce=order.nonce,
        side=order.side,
        amount_in=order.amount_in,
        amount_out=order.amount_out,
        expiration=order.expiration,
    )


# Backward compatibility alias
to_compact_order = to_order


def serialize_order(order: Order) -> bytes:
    """Serialize a compact order to bytes.

    Layout (29 bytes):
    - nonce (4, u32) | side (1) | amount_in (8) | amount_out (8) | expiration (8)
    """
    return (
        encode_u32(order.nonce)
        + encode_u8(order.side)
        + encode_u64(order.amount_in)
        + encode_u64(order.amount_out)
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
        amount_in=decode_u64(data, 5),
        amount_out=decode_u64(data, 13),
        expiration=decode_i64(data, 21),
    )


# Backward compatibility alias
deserialize_compact_order = deserialize_order


def create_signed_bid_order(params: BidOrderParams, keypair: Keypair) -> SignedOrder:
    """Create and sign a bid order in one call."""
    order = create_bid_order(params)
    sign_order(order, keypair)
    return order


def create_signed_ask_order(params: AskOrderParams, keypair: Keypair) -> SignedOrder:
    """Create and sign an ask order in one call."""
    order = create_ask_order(params)
    sign_order(order, keypair)
    return order


def validate_order(order: SignedOrder, check_expiration: bool = False) -> None:
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
    if order.amount_in == 0:
        raise InvalidOrderError("amount_in cannot be zero")
    if order.amount_out == 0:
        raise InvalidOrderError("amount_out cannot be zero")

    # Validate u64 bounds
    if order.amount_in > MAX_U64:
        raise InvalidOrderError(f"amount_in exceeds u64 max: {order.amount_in}")
    if order.amount_out > MAX_U64:
        raise InvalidOrderError(f"amount_out exceeds u64 max: {order.amount_out}")

    # Validate side
    if order.side not in (OrderSide.BID, OrderSide.ASK):
        raise InvalidOrderError(f"Invalid side: {order.side}")

    # Validate expiration (if set and check enabled, must be in the future)
    if check_expiration and order.expiration != 0 and order.expiration < int(time.time()):
        raise InvalidOrderError(f"Order already expired: expiration={order.expiration}")

    # Validate maker is not zero pubkey
    if bytes(order.maker) == bytes(32):
        raise InvalidOrderError("maker cannot be zero pubkey")


def validate_signed_order(order: SignedOrder) -> None:
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


def cancel_trigger_order_message(trigger_order_id: str) -> bytes:
    """Build the message bytes for cancelling a trigger order.

    The message is the trigger_order_id as ASCII bytes.
    """
    return trigger_order_id.encode("ascii")


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


def cancel_all_message(
    user_pubkey: str, orderbook_id: str, timestamp: int, salt: str
) -> str:
    """Build the message string for cancelling all orders.

    Format: "cancel_all:{pubkey}:{orderbook_id}:{timestamp}:{salt}"
    """
    return f"cancel_all:{user_pubkey}:{orderbook_id}:{timestamp}:{salt}"


def generate_cancel_all_salt() -> str:
    """Generate a random UUID v4 salt for cancel-all replay protection."""
    return str(uuid.uuid4())


def sign_cancel_all(
    user_pubkey: str,
    orderbook_id: str,
    timestamp: int,
    salt: str,
    keypair: Keypair,
) -> str:
    """Sign a cancel-all orders request.

    Returns the signature as a 128-char hex string.
    """
    message = cancel_all_message(user_pubkey, orderbook_id, timestamp, salt)
    message_bytes = message.encode("ascii")

    secret_bytes = bytes(keypair)
    seed = secret_bytes[:32]

    signing_key = SigningKey(seed)
    signed = signing_key.sign(message_bytes)
    return signed.signature.hex()


# =========================================================================
# Bridge Methods (order <-> API)
# =========================================================================


def signature_hex(order: SignedOrder) -> str:
    """Return the order signature as a 128-char hex string."""
    return order.signature.hex()


def is_signed(order: SignedOrder) -> bool:
    """Check if an order has a non-zero signature."""
    return order.signature != bytes(SIGNATURE_SIZE)


def calculate_taker_fill(maker_order: SignedOrder, maker_fill_amount: int) -> int:
    """Calculate the taker fill amount given a maker fill amount.

    Returns the taker fill amount based on maker's price ratio.

    Raises:
        InvalidOrderError: If maker_order.amount_in is zero or overflow occurs
    """
    if maker_order.amount_in == 0:
        raise InvalidOrderError("maker_order.amount_in cannot be zero")

    result = (maker_fill_amount * maker_order.amount_out) // maker_order.amount_in
    if result > MAX_U64:
        raise InvalidOrderError("taker fill calculation overflow")

    return result


def to_submit_request(
    order: SignedOrder,
    orderbook_id: str,
    time_in_force=None,
    trigger_price=None,
    trigger_type=None,
    deposit_source=None,
):
    """Convert a signed SignedOrder to a SubmitOrderRequest.

    Args:
        order: A signed SignedOrder
        orderbook_id: The orderbook identifier
        time_in_force: Optional TimeInForce value
        trigger_price: Optional trigger price (float)
        trigger_type: Optional TriggerType value
        deposit_source: Optional DepositSource value

    Returns:
        SubmitOrderRequest ready for API submission

    Raises:
        InvalidOrderError: If the order is not signed
    """
    if not is_signed(order):
        raise InvalidOrderError("Order must be signed before submitting")

    # Import here to avoid circular dependency
    from ..shared.types import SubmitOrderRequest

    return SubmitOrderRequest(
        maker=str(order.maker),
        nonce=order.nonce,
        market_pubkey=str(order.market),
        base_token=str(order.base_mint),
        quote_token=str(order.quote_mint),
        side=int(order.side),
        amount_in=order.amount_in,
        amount_out=order.amount_out,
        expiration=order.expiration,
        signature=signature_hex(order),
        orderbook_id=orderbook_id,
        time_in_force=time_in_force,
        trigger_price=trigger_price,
        trigger_type=trigger_type,
        deposit_source=deposit_source,
    )


def is_order_expired(order: SignedOrder, current_time: int) -> bool:
    """Check if an order is expired. Expiration of 0 means no expiration."""
    if order.expiration == 0:
        return False
    return current_time >= order.expiration


def apply_signature(order: SignedOrder, sig_bs58: str) -> None:
    """Apply a base58-encoded signature to an order in place."""
    import base58
    sig_bytes = base58.b58decode(sig_bs58)
    if len(sig_bytes) != SIGNATURE_SIZE:
        raise InvalidSignatureError(
            f"Expected {SIGNATURE_SIZE} bytes, got {len(sig_bytes)}"
        )
    order.signature = sig_bytes


def derive_orderbook_id(order: SignedOrder) -> str:
    """Derive orderbook ID from order's base/quote mints.

    Format: "{base_mint[:8]}_{quote_mint[:8]}"
    """
    base = str(order.base_mint)[:8]
    quote = str(order.quote_mint)[:8]
    return f"{base}_{quote}"


def orders_can_cross(buy_order: SignedOrder, sell_order: SignedOrder) -> bool:
    """Check if two orders can cross (prices are compatible).

    Returns True if the buyer's price >= seller's price.
    Validates sides and non-zero amounts before comparing.
    """
    if buy_order.side != OrderSide.BID or sell_order.side != OrderSide.ASK:
        return False

    if (
        buy_order.amount_in == 0
        or buy_order.amount_out == 0
        or sell_order.amount_in == 0
        or sell_order.amount_out == 0
    ):
        return False

    return orders_cross(
        bid_maker_amount=buy_order.amount_in,
        bid_taker_amount=buy_order.amount_out,
        ask_maker_amount=sell_order.amount_in,
        ask_taker_amount=sell_order.amount_out,
    )
