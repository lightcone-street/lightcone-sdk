"""Account deserialization for the Lightcone SDK."""

from .constants import (
    EXCHANGE_DISCRIMINATOR,
    EXCHANGE_SIZE,
    GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR,
    GLOBAL_DEPOSIT_TOKEN_SIZE,
    MARKET_DISCRIMINATOR,
    MARKET_SIZE,
    ORDER_STATUS_DISCRIMINATOR,
    ORDER_STATUS_SIZE,
    ORDERBOOK_DISCRIMINATOR,
    ORDERBOOK_SIZE,
    POSITION_DISCRIMINATOR,
    POSITION_SIZE,
    USER_NONCE_DISCRIMINATOR,
    USER_NONCE_SIZE,
)
from .errors import InvalidAccountDataError, InvalidDiscriminatorError
from .types import (
    Exchange,
    GlobalDepositToken,
    Market,
    MarketStatus,
    Orderbook,
    OrderStatus,
    Position,
    UserNonce,
)
from .utils import decode_bool, decode_pubkey, decode_u8, decode_u16, decode_u64


def _validate_discriminator(data: bytes, expected: bytes, name: str) -> None:
    """Validate account discriminator."""
    if len(data) < 8:
        raise InvalidAccountDataError(f"{name} data too short: {len(data)} bytes")
    actual = data[:8]
    if actual != expected:
        raise InvalidDiscriminatorError(expected, actual)


def deserialize_exchange(data: bytes) -> Exchange:
    """Deserialize an Exchange account.

    Layout (120 bytes):
    - [0..8]: discriminator
    - [8..40]: authority (Pubkey)
    - [40..72]: operator (Pubkey)
    - [72..104]: manager (Pubkey)
    - [104..112]: market_count (u64 LE)
    - [112]: paused (bool)
    - [113]: bump (u8)
    - [114..116]: deposit_token_count (u16 LE)
    - [116..120]: padding
    """
    _validate_discriminator(data, EXCHANGE_DISCRIMINATOR, "Exchange")

    if len(data) < EXCHANGE_SIZE:
        raise InvalidAccountDataError(
            f"Exchange data too short: {len(data)} bytes (expected {EXCHANGE_SIZE})"
        )

    return Exchange(
        authority=decode_pubkey(data, 8),
        operator=decode_pubkey(data, 40),
        manager=decode_pubkey(data, 72),
        market_count=decode_u64(data, 104),
        paused=decode_bool(data, 112),
        bump=decode_u8(data, 113),
        deposit_token_count=decode_u16(data, 114),
    )


def deserialize_market(data: bytes) -> Market:
    """Deserialize a Market account.

    Layout (120 bytes):
    - [0..8]: discriminator
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

    return Market(
        market_id=decode_u64(data, 8),
        num_outcomes=decode_u8(data, 16),
        status=MarketStatus(decode_u8(data, 17)),
        winning_outcome=decode_u8(data, 18),
        has_winning_outcome=decode_bool(data, 19),
        bump=decode_u8(data, 20),
        oracle=decode_pubkey(data, 24),
        question_id=data[56:88],
        condition_id=data[88:120],
    )


def deserialize_position(data: bytes) -> Position:
    """Deserialize a Position account.

    Layout (80 bytes):
    - [0..8]: discriminator
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

    Layout (32 bytes):
    - [0..8]: discriminator
    - [8..16]: remaining (u64 LE)
    - [16..24]: base_remaining (u64 LE)
    - [24]: is_cancelled (bool)
    - [25..32]: padding (7 bytes)
    """
    _validate_discriminator(data, ORDER_STATUS_DISCRIMINATOR, "OrderStatus")

    if len(data) < ORDER_STATUS_SIZE:
        raise InvalidAccountDataError(
            f"OrderStatus data too short: {len(data)} bytes (expected {ORDER_STATUS_SIZE})"
        )

    return OrderStatus(
        remaining=decode_u64(data, 8),
        base_remaining=decode_u64(data, 16),
        is_cancelled=decode_bool(data, 24),
    )


def deserialize_user_nonce(data: bytes) -> UserNonce:
    """Deserialize a UserNonce account.

    Layout (16 bytes):
    - [0..8]: discriminator
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


def deserialize_orderbook(data: bytes) -> Orderbook:
    """Deserialize an Orderbook account.

    Layout (144 bytes):
    - [0..8]: discriminator
    - [8..40]: market (Pubkey)
    - [40..72]: mint_a (Pubkey)
    - [72..104]: mint_b (Pubkey)
    - [104..136]: lookup_table (Pubkey)
    - [136]: base_index (u8)
    - [137]: bump (u8)
    - [138..144]: padding (6 bytes)
    """
    _validate_discriminator(data, ORDERBOOK_DISCRIMINATOR, "Orderbook")

    if len(data) < ORDERBOOK_SIZE:
        raise InvalidAccountDataError(
            f"Orderbook data too short: {len(data)} bytes (expected {ORDERBOOK_SIZE})"
        )

    return Orderbook(
        market=decode_pubkey(data, 8),
        mint_a=decode_pubkey(data, 40),
        mint_b=decode_pubkey(data, 72),
        lookup_table=decode_pubkey(data, 104),
        base_index=decode_u8(data, 136),
        bump=decode_u8(data, 137),
    )


def is_exchange_account(data: bytes) -> bool:
    """Check if data has the Exchange discriminator."""
    return len(data) >= 8 and data[:8] == EXCHANGE_DISCRIMINATOR


def is_market_account(data: bytes) -> bool:
    """Check if data has the Market discriminator."""
    return len(data) >= 8 and data[:8] == MARKET_DISCRIMINATOR


def is_position_account(data: bytes) -> bool:
    """Check if data has the Position discriminator."""
    return len(data) >= 8 and data[:8] == POSITION_DISCRIMINATOR


def is_order_status_account(data: bytes) -> bool:
    """Check if data has the OrderStatus discriminator."""
    return len(data) >= 8 and data[:8] == ORDER_STATUS_DISCRIMINATOR


def is_user_nonce_account(data: bytes) -> bool:
    """Check if data has the UserNonce discriminator."""
    return len(data) >= 8 and data[:8] == USER_NONCE_DISCRIMINATOR


def is_orderbook_account(data: bytes) -> bool:
    """Check if data has the Orderbook discriminator."""
    return len(data) >= 8 and data[:8] == ORDERBOOK_DISCRIMINATOR


def is_global_deposit_token(data: bytes) -> bool:
    """Check if data has the GlobalDepositToken discriminator."""
    return len(data) >= 8 and data[:8] == GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR


def deserialize_global_deposit_token(data: bytes) -> GlobalDepositToken:
    """Deserialize a GlobalDepositToken account.

    Layout (48 bytes):
    - [0..8]: discriminator
    - [8..40]: mint (Pubkey)
    - [40]: active (bool)
    - [41]: bump (u8)
    - [42..44]: index (u16 LE)
    - [44..48]: padding
    """
    _validate_discriminator(data, GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR, "GlobalDepositToken")

    if len(data) < GLOBAL_DEPOSIT_TOKEN_SIZE:
        raise InvalidAccountDataError(
            f"GlobalDepositToken data too short: {len(data)} bytes (expected {GLOBAL_DEPOSIT_TOKEN_SIZE})"
        )

    return GlobalDepositToken(
        mint=decode_pubkey(data, 8),
        active=decode_bool(data, 40),
        bump=decode_u8(data, 41),
        index=decode_u16(data, 42),
    )
