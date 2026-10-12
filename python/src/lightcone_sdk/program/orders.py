"""Order creation, hashing, signing, and serialization for the Lightcone SDK.

Compatibility imports use explicit aliases to retain ordinary public module attributes.
"""

import os
import time
import uuid

import nacl.exceptions
from nacl.signing import SigningKey, VerifyKey
from solders.keypair import Keypair
from solders.pubkey import Pubkey as Pubkey
from solders.signature import Signature

from ..shared.scaling import (
    OrderbookRules,
    ScalingError,
    validate_raw_amounts,
    validate_signed_fields,
)
from .constants import COMPACT_ORDER_SIZE as COMPACT_ORDER_SIZE
from .constants import FULL_ORDER_SIZE as FULL_ORDER_SIZE
from .constants import (
    ORDER_BASE_MINT_OFFSET,
    ORDER_EXPIRATION_OFFSET,
    ORDER_MAKER_AMOUNT_OFFSET,
    ORDER_MAKER_OFFSET,
    ORDER_MARKET_OFFSET,
    ORDER_QUOTE_MINT_OFFSET,
    ORDER_SALT_OFFSET,
    ORDER_SIDE_OFFSET,
    ORDER_SIGNATURE_OFFSET,
    ORDER_SIZE,
    ORDER_TAKER_AMOUNT_OFFSET,
    SIGNATURE_SIZE,
    SIGNED_ORDER_SIZE,
)
from .constants import ORDER_HASH_SIZE as ORDER_HASH_SIZE
from .errors import (
    ArithmeticOverflowError,
    InvalidDataLengthError,
    InvalidOrderError,
    InvalidSideError,
    InvalidSignatureError,
    SerializationError,
    UnsignedOrderError,
)
from .types import (
    AskOrderParams,
    BidOrderParams,
    Order,
    OrderSide,
    SignedOrder,
)
from .utils import (
    decode_i64,
    decode_pubkey,
    decode_u8,
    decode_u64,
    encode_i64,
    encode_u8,
    encode_u64,
    keccak256,
    orders_cross,
)
from .utils import derive_condition_id as derive_condition_id

# Backward compatibility alias
FullOrder = SignedOrder

# Signed 64-bit integer range (order expiration).
MIN_I64 = -(2**63)
MAX_I64 = 2**63 - 1
# Unsigned 64-bit integer maximum (salt, amounts, fills).
MAX_U64 = 2**64 - 1


def generate_salt() -> int:
    """Draw a uniformly random salt from the full u64 range.

    The salt is the order's only identity, so it spans the full u64 range.
    """
    return int.from_bytes(os.urandom(8), "little")


def _is_int(value: object) -> bool:
    """True for an ``int`` (including ``IntEnum``), never a ``bool``."""
    return isinstance(value, int) and not isinstance(value, bool)


def _decode_side(value: object) -> OrderSide:
    """Validate a side as BID (0) or ASK (1)."""
    if not _is_int(value) or value not in (OrderSide.BID, OrderSide.ASK):
        raise InvalidSideError(value)  # type: ignore[arg-type]
    return OrderSide(value)


def _encode_u64(value: object) -> bytes:
    if not _is_int(value):
        raise SerializationError(f"u64 value out of range: {value!r}")
    return encode_u64(value)  # type: ignore[arg-type]


def _encode_i64(value: object) -> bytes:
    if not _is_int(value):
        raise SerializationError(f"i64 value out of range: {value!r}")
    return encode_i64(value)  # type: ignore[arg-type]


def create_bid_order(params: BidOrderParams) -> SignedOrder:
    """Create a bid order (buyer wants base tokens, gives quote tokens).

    The signature field is left empty (64 zero bytes).
    """
    return SignedOrder(
        salt=params.salt,
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
        salt=params.salt,
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
    """Build the ``ORDER_PREIMAGE_SIZE`` (161) byte signing preimage (excludes the signature).

    Layout (integers little-endian):
    salt (8, u64) | maker (32) | market (32) | base_mint (32) | quote_mint (32) |
    side (1) | amount_in (8, u64) | amount_out (8, u64) | expiration (8, i64)

    Raises ``SerializationError`` for a value outside its integer range (Python
    integers never wrap) and ``InvalidSideError`` for a side other than BID or ASK.
    """
    return (
        _encode_u64(order.salt)
        + bytes(order.maker)
        + bytes(order.market)
        + bytes(order.base_mint)
        + bytes(order.quote_mint)
        + encode_u8(_decode_side(order.side))
        + _encode_u64(order.amount_in)
        + _encode_u64(order.amount_out)
        + _encode_i64(order.expiration)
    )


# Rust-compatible name for the signing preimage.
order_preimage = serialize_order_for_hashing


def hash_order(order: SignedOrder) -> bytes:
    """Keccak-256 of the 161-byte preimage: the order ID and order-status PDA seed.

    Returns a 32-byte hash.
    """
    data = serialize_order_for_hashing(order)
    return keccak256(data)


def hash_order_hex(order: SignedOrder) -> str:
    """The order hash as 64 lowercase hex characters (the message a wallet signs)."""
    return hash_order(order).hex()


def get_order_message(order: SignedOrder) -> bytes:
    """The exact bytes a wallet signs: the ASCII hex of the order hash."""
    return hash_order_hex(order).encode("ascii")


def _signing_key(keypair: Keypair) -> SigningKey:
    # solders Keypair stores the full 64-byte secret (seed + public key).
    return SigningKey(bytes(keypair)[:32])


def sign_order(order: SignedOrder, keypair: Keypair, rules: OrderbookRules) -> bytes:
    """Sign an order with a keypair after preflighting it against ``rules``.

    Signs the 64-character lowercase hex of the order hash with the keypair's
    Ed25519 key. Updates the order's signature in place only on success and
    returns the signature.
    """
    side = _decode_side(order.side)
    validate_signed_fields(order.amount_in, order.amount_out)
    validate_raw_amounts(order.amount_in, order.amount_out, int(side), rules)
    message = get_order_message(order)
    signature = _signing_key(keypair).sign(message).signature
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

    message = get_order_message(order)

    try:
        # Get verify key from maker pubkey
        verify_key = VerifyKey(bytes(order.maker))
        verify_key.verify(message, order.signature)
        return True
    except nacl.exceptions.BadSignatureError:
        return False
    except (nacl.exceptions.ValueError, ValueError) as e:
        raise InvalidOrderError(f"Invalid maker public key: {e}") from e


def serialize_full_order(order: SignedOrder) -> bytes:
    """Serialize a signed order to exactly 225 bytes: the preimage, then the signature."""
    if len(order.signature) != SIGNATURE_SIZE:
        raise InvalidDataLengthError(SIGNATURE_SIZE, len(order.signature))
    return serialize_order_for_hashing(order) + bytes(order.signature)


def deserialize_full_order(data: bytes) -> SignedOrder:
    """Deserialize a signed order from exactly 225 bytes."""
    if len(data) != SIGNED_ORDER_SIZE:
        raise InvalidDataLengthError(SIGNED_ORDER_SIZE, len(data))

    return SignedOrder(
        salt=decode_u64(data, ORDER_SALT_OFFSET),
        maker=decode_pubkey(data, ORDER_MAKER_OFFSET),
        market=decode_pubkey(data, ORDER_MARKET_OFFSET),
        base_mint=decode_pubkey(data, ORDER_BASE_MINT_OFFSET),
        quote_mint=decode_pubkey(data, ORDER_QUOTE_MINT_OFFSET),
        side=_decode_side(decode_u8(data, ORDER_SIDE_OFFSET)),
        amount_in=decode_u64(data, ORDER_MAKER_AMOUNT_OFFSET),
        amount_out=decode_u64(data, ORDER_TAKER_AMOUNT_OFFSET),
        expiration=decode_i64(data, ORDER_EXPIRATION_OFFSET),
        signature=bytes(
            data[ORDER_SIGNATURE_OFFSET : ORDER_SIGNATURE_OFFSET + SIGNATURE_SIZE]
        ),
    )


def to_order(order: SignedOrder) -> Order:
    """Convert a signed order to a compact order (33 bytes, no maker, market, or mints)."""
    return Order(
        salt=order.salt,
        side=order.side,
        amount_in=order.amount_in,
        amount_out=order.amount_out,
        expiration=order.expiration,
    )


# Backward compatibility alias
to_compact_order = to_order


def order_to_signed(
    order: Order,
    maker: Pubkey,
    market: Pubkey,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    signature: bytes,
) -> SignedOrder:
    """Rebuild a signed order from a compact order and its account context."""
    return SignedOrder(
        salt=order.salt,
        maker=maker,
        market=market,
        base_mint=base_mint,
        quote_mint=quote_mint,
        side=order.side,
        amount_in=order.amount_in,
        amount_out=order.amount_out,
        expiration=order.expiration,
        signature=signature,
    )


def serialize_order(order: Order) -> bytes:
    """Serialize a compact order to exactly 33 bytes.

    Layout (little-endian): salt (8, u64) | side (1) | amount_in (8, u64) |
    amount_out (8, u64) | expiration (8, i64)
    """
    return (
        _encode_u64(order.salt)
        + encode_u8(_decode_side(order.side))
        + _encode_u64(order.amount_in)
        + _encode_u64(order.amount_out)
        + _encode_i64(order.expiration)
    )


# Backward compatibility alias
serialize_compact_order = serialize_order


def deserialize_order(data: bytes) -> Order:
    """Deserialize a compact order from exactly 33 bytes."""
    if len(data) != ORDER_SIZE:
        raise InvalidDataLengthError(ORDER_SIZE, len(data))

    return Order(
        salt=decode_u64(data, 0),
        side=_decode_side(decode_u8(data, 8)),
        amount_in=decode_u64(data, 9),
        amount_out=decode_u64(data, 17),
        expiration=decode_i64(data, 25),
    )


# Backward compatibility alias
deserialize_compact_order = deserialize_order


def create_signed_bid_order(
    params: BidOrderParams, keypair: Keypair, rules: OrderbookRules
) -> SignedOrder:
    """Create and sign a bid order in one call."""
    order = create_bid_order(params)
    sign_order(order, keypair, rules)
    return order


def create_signed_ask_order(
    params: AskOrderParams, keypair: Keypair, rules: OrderbookRules
) -> SignedOrder:
    """Create and sign an ask order in one call."""
    order = create_ask_order(params)
    sign_order(order, keypair, rules)
    return order


def validate_order(order: SignedOrder, check_expiration: bool = False) -> None:
    """Validate an order's fields.

    Args:
        order: The order to validate
        check_expiration: If True, also check that the order hasn't expired

    Raises InvalidOrderError if any field is invalid.
    """
    if not _is_int(order.salt) or not 0 <= order.salt <= MAX_U64:
        raise InvalidOrderError(f"salt is outside the u64 range: {order.salt!r}")
    try:
        validate_signed_fields(order.amount_in, order.amount_out)
    except ScalingError as error:
        raise InvalidOrderError(str(error)) from error
    if not _is_int(order.side) or order.side not in (OrderSide.BID, OrderSide.ASK):
        raise InvalidOrderError(f"Invalid side: {order.side!r}")
    if not _is_int(order.expiration) or not MIN_I64 <= order.expiration <= MAX_I64:
        raise InvalidOrderError(
            f"expiration is outside the i64 range: {order.expiration!r}"
        )

    if check_expiration and is_order_expired(order, int(time.time())):
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

    The message is the order hash hex string as UTF-8 bytes (same protocol as
    order signing; the backend verifies the message's UTF-8 bytes).
    """
    return order_hash.encode("utf-8")


def sign_cancel_order(order_hash: str, keypair: Keypair) -> str:
    """Sign a cancel order request.

    Returns the signature as a 128-char hex string.
    """
    message = cancel_order_message(order_hash)
    return _signing_key(keypair).sign(message).signature.hex()


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
    # The backend verifies the message's UTF-8 bytes; the salt may be any text.
    message = cancel_all_message(user_pubkey, orderbook_id, timestamp, salt)
    return _signing_key(keypair).sign(message.encode("utf-8")).signature.hex()


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
    """Calculate the minimum taker fill for a maker fill, in raw atoms.

    ``maker_fill_amount`` is what the maker gives, in atoms of the maker's
    given mint (u64). The result is the least the taker must give the maker,
    in atoms of the maker's received mint, rounded up as the program requires:
    ``taker_fill >= ceil(maker_fill * maker.amount_out / maker.amount_in)``.

    Raises:
        ArithmeticOverflowError: For a zero ``maker_order.amount_in``, a maker
            amount or fill that is not a u64 ``int``, or a result above u64
            (as in Rust).
    """
    for amount in (maker_order.amount_in, maker_order.amount_out, maker_fill_amount):
        if not _is_int(amount) or not 0 <= amount <= MAX_U64:
            raise ArithmeticOverflowError()
    if maker_order.amount_in == 0:
        raise ArithmeticOverflowError()

    numerator = maker_fill_amount * maker_order.amount_out
    result = -(-numerator // maker_order.amount_in)
    if result > MAX_U64:
        raise ArithmeticOverflowError()

    return result


def to_submit_request(
    order: SignedOrder,
    orderbook_id: str,
    time_in_force=None,
    deposit_source=None,
):
    """Encode a signed order as the REST submission request.

    Keeps the exact integer fields; rejects an unsigned order and any field
    the order layout cannot encode (u64 salt, i64 expiration, BID or ASK).

    Args:
        order: A signed SignedOrder
        orderbook_id: The orderbook identifier
        time_in_force: Optional TimeInForce value
        deposit_source: Optional DepositSource value

    Returns:
        SubmitOrderRequest ready for API submission

    Raises:
        UnsignedOrderError: If the order is not signed
    """
    if len(order.signature) != SIGNATURE_SIZE:
        raise InvalidDataLengthError(SIGNATURE_SIZE, len(order.signature))
    if not is_signed(order):
        raise UnsignedOrderError()
    validate_signed_fields(order.amount_in, order.amount_out)
    serialize_order_for_hashing(order)

    # Import here to avoid circular dependency
    from ..shared.types import SubmitOrderRequest

    return SubmitOrderRequest(
        maker=str(order.maker),
        salt=order.salt,
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
        deposit_source=deposit_source,
    )


def is_order_expired(order: SignedOrder, current_time: int) -> bool:
    """Check if an order is expired at ``current_time`` (Unix seconds).

    An order remains valid during its expiration second; an expiration of 0
    means no expiration.
    """
    if order.expiration == 0:
        return False
    return order.expiration < current_time


def apply_signature(order: SignedOrder, sig_bs58: str, rules: OrderbookRules) -> None:
    """Apply an external base58 Ed25519 signature to an order in place.

    Preflights the order against ``rules`` first. A signature that is not
    canonical base58 for exactly 64 bytes raises ``InvalidSignatureError``.
    """
    side = _decode_side(order.side)
    validate_signed_fields(order.amount_in, order.amount_out)
    validate_raw_amounts(order.amount_in, order.amount_out, int(side), rules)
    # The order must be encodable: u64 salt, i64 expiration, BID or ASK.
    serialize_order_for_hashing(order)
    try:
        signature = Signature.from_string(sig_bs58)
    except (TypeError, ValueError):
        raise InvalidSignatureError() from None
    order.signature = bytes(signature)


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
