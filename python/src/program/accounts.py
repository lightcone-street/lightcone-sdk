"""Account deserialization for the Lightcone SDK."""

from typing import Optional

from .constants import (
    EXCHANGE_DISCRIMINATOR,
    EXCHANGE_SIZE,
    MARKET_DISCRIMINATOR,
    MARKET_SIZE,
    NO_WINNING_OUTCOME,
    ORDER_STATUS_DISCRIMINATOR,
    ORDER_STATUS_SIZE,
    POSITION_DISCRIMINATOR,
    POSITION_SIZE,
    USER_NONCE_DISCRIMINATOR,
    USER_NONCE_SIZE,
)
from .errors import InvalidAccountDataError, InvalidDiscriminatorError
from .types import Exchange, Market, MarketStatus, OrderStatus, Position, UserNonce
from .utils import decode_bool, decode_pubkey, decode_u64, decode_u8


def _validate_discriminator(data: bytes, expected: bytes, name: str) -> None:
    """Validate account discriminator."""
    if len(data) < 8:
        raise InvalidAccountDataError(f"{name} data too short: {len(data)} bytes")
    actual = data[:8]
    if actual != expected:
        raise InvalidDiscriminatorError(expected, actual)


def deserialize_exchange(data: bytes) -> Exchange:
    """Deserialize an Exchange account.

    Layout (88 bytes):
    - [0..8]: discriminator ("exchange")
    - [8..40]: authority (Pubkey)
    - [40..72]: operator (Pubkey)
    - [72..80]: market_count (u64 LE)
    - [80]: paused (bool)
    - [81]: bump (u8)
    - [82..88]: padding
    """
    _validate_discriminator(data, EXCHANGE_DISCRIMINATOR, "Exchange")

    if len(data) < EXCHANGE_SIZE:
        raise InvalidAccountDataError(
            f"Exchange data too short: {len(data)} bytes (expected {EXCHANGE_SIZE})"
        )

    return Exchange(
        authority=decode_pubkey(data, 8),
        operator=decode_pubkey(data, 40),
        market_count=decode_u64(data, 72),
        paused=decode_bool(data, 80),
        bump=decode_u8(data, 81),
    )


def deserialize_market(data: bytes) -> Market:
    """Deserialize a Market account.

    Layout (120 bytes):
    - [0..8]: discriminator ("market\\x00\\x00")
    - [8..16]: market_id (u64 LE)
    - [16]: num_outcomes (u8)
    - [17]: status (u8: 0=Pending, 1=Active, 2=Resolved, 3=Cancelled)
    - [18]: winning_outcome (u8, 255 if not resolved)
    - [19]: has_winning_outcome (bool)
    - [20]: bump (u8)
    - [21..24]: padding (3 bytes)
    - [24..56]: oracle (Pubkey)
    - [56..88]: question_id (32 bytes)
    - [88..120]: condition_id (32 bytes)
    """
    _validate_discriminator(data, MARKET_DISCRIMINATOR, "Market")

    if len(data) < MARKET_SIZE:
        raise InvalidAccountDataError(
            f"Market data too short: {len(data)} bytes (expected {MARKET_SIZE})"
        )

    winning_outcome_raw = decode_u8(data, 18)
    has_winning_outcome = decode_bool(data, 19)

    winning_outcome: Optional[int] = None
    if has_winning_outcome and winning_outcome_raw != NO_WINNING_OUTCOME:
        winning_outcome = winning_outcome_raw

    return Market(
        market_id=decode_u64(data, 8),
        num_outcomes=decode_u8(data, 16),
        status=MarketStatus(decode_u8(data, 17)),
        winning_outcome=winning_outcome,
        bump=decode_u8(data, 20),
        oracle=decode_pubkey(data, 24),
        question_id=data[56:88],
        condition_id=data[88:120],
    )


def deserialize_position(data: bytes) -> Position:
    """Deserialize a Position account.

    Layout (80 bytes):
    - [0..8]: discriminator ("position")
    - [8..40]: owner (Pubkey)
    - [40..72]: market (Pubkey)
    - [72]: bump (u8)
    - [73..80]: padding (7 bytes)
    """
    _validate_discriminator(data, POSITION_DISCRIMINATOR, "Position")

    if len(data) < POSITION_SIZE:
        raise InvalidAccountDataError(
            f"Position data too short: {len(data)} bytes (expected {POSITION_SIZE})"
        )

    return Position(
        owner=decode_pubkey(data, 8),
        market=decode_pubkey(data, 40),
        bump=decode_u8(data, 72),
    )


def deserialize_order_status(data: bytes) -> OrderStatus:
    """Deserialize an OrderStatus account.

    Layout (24 bytes):
    - [0..8]: discriminator ("ordstat\\x00")
    - [8..16]: remaining (u64 LE)
    - [16]: is_cancelled (bool)
    - [17..24]: padding (7 bytes)
    """
    _validate_discriminator(data, ORDER_STATUS_DISCRIMINATOR, "OrderStatus")

    if len(data) < ORDER_STATUS_SIZE:
        raise InvalidAccountDataError(
            f"OrderStatus data too short: {len(data)} bytes (expected {ORDER_STATUS_SIZE})"
        )

    return OrderStatus(
        remaining=decode_u64(data, 8),
        is_cancelled=decode_bool(data, 16),
    )


def deserialize_user_nonce(data: bytes) -> UserNonce:
    """Deserialize a UserNonce account.

    Layout (16 bytes):
    - [0..8]: discriminator ("usrnonce")
    - [8..16]: nonce (u64 LE)
    """
    _validate_discriminator(data, USER_NONCE_DISCRIMINATOR, "UserNonce")

    if len(data) < USER_NONCE_SIZE:
        raise InvalidAccountDataError(
            f"UserNonce data too short: {len(data)} bytes (expected {USER_NONCE_SIZE})"
        )

    return UserNonce(
        nonce=decode_u64(data, 8),
    )
