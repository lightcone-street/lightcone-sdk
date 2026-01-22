"""Tests for account deserialization."""

import struct

import pytest
from solders.pubkey import Pubkey

from src.program import (
    EXCHANGE_DISCRIMINATOR,
    MARKET_DISCRIMINATOR,
    ORDER_STATUS_DISCRIMINATOR,
    POSITION_DISCRIMINATOR,
    USER_NONCE_DISCRIMINATOR,
    MarketStatus,
    InvalidAccountDataError,
    InvalidDiscriminatorError,
    deserialize_exchange,
    deserialize_market,
    deserialize_order_status,
    deserialize_position,
    deserialize_user_nonce,
)


def build_exchange_data(
    authority: Pubkey,
    operator: Pubkey,
    market_count: int,
    paused: bool,
    bump: int,
) -> bytes:
    """Build Exchange account data for testing."""
    data = bytearray()
    data.extend(EXCHANGE_DISCRIMINATOR)
    data.extend(bytes(authority))
    data.extend(bytes(operator))
    data.extend(struct.pack("<Q", market_count))
    data.append(1 if paused else 0)
    data.append(bump)
    data.extend(bytes(6))  # padding
    return bytes(data)


def build_market_data(
    market_id: int,
    num_outcomes: int,
    status: MarketStatus,
    winning_outcome: int,
    has_winning: bool,
    bump: int,
    oracle: Pubkey,
    question_id: bytes,
    condition_id: bytes,
) -> bytes:
    """Build Market account data for testing."""
    data = bytearray()
    data.extend(MARKET_DISCRIMINATOR)
    data.extend(struct.pack("<Q", market_id))
    data.append(num_outcomes)
    data.append(status)
    data.append(winning_outcome)
    data.append(1 if has_winning else 0)
    data.append(bump)
    data.extend(bytes(3))  # padding
    data.extend(bytes(oracle))
    data.extend(question_id)
    data.extend(condition_id)
    return bytes(data)


def build_position_data(owner: Pubkey, market: Pubkey, bump: int) -> bytes:
    """Build Position account data for testing."""
    data = bytearray()
    data.extend(POSITION_DISCRIMINATOR)
    data.extend(bytes(owner))
    data.extend(bytes(market))
    data.append(bump)
    data.extend(bytes(7))  # padding
    return bytes(data)


def build_order_status_data(remaining: int, is_cancelled: bool) -> bytes:
    """Build OrderStatus account data for testing."""
    data = bytearray()
    data.extend(ORDER_STATUS_DISCRIMINATOR)
    data.extend(struct.pack("<Q", remaining))
    data.append(1 if is_cancelled else 0)
    data.extend(bytes(7))  # padding
    return bytes(data)


def build_user_nonce_data(nonce: int) -> bytes:
    """Build UserNonce account data for testing."""
    data = bytearray()
    data.extend(USER_NONCE_DISCRIMINATOR)
    data.extend(struct.pack("<Q", nonce))
    return bytes(data)


class TestDeserializeExchange:
    def test_deserialize_valid_data(self):
        authority = Pubkey.new_unique()
        operator = Pubkey.new_unique()
        data = build_exchange_data(
            authority=authority,
            operator=operator,
            market_count=42,
            paused=False,
            bump=255,
        )

        exchange = deserialize_exchange(data)

        assert exchange.authority == authority
        assert exchange.operator == operator
        assert exchange.market_count == 42
        assert exchange.paused is False
        assert exchange.bump == 255

    def test_deserialize_paused_exchange(self):
        data = build_exchange_data(
            authority=Pubkey.new_unique(),
            operator=Pubkey.new_unique(),
            market_count=0,
            paused=True,
            bump=254,
        )

        exchange = deserialize_exchange(data)
        assert exchange.paused is True

    def test_invalid_discriminator(self):
        data = b"invalid!" + bytes(80)

        with pytest.raises(InvalidDiscriminatorError):
            deserialize_exchange(data)

    def test_data_too_short(self):
        data = EXCHANGE_DISCRIMINATOR + bytes(10)

        with pytest.raises(InvalidAccountDataError):
            deserialize_exchange(data)


class TestDeserializeMarket:
    def test_deserialize_active_market(self):
        oracle = Pubkey.new_unique()
        question_id = bytes(range(32))
        condition_id = bytes(range(32, 64))

        data = build_market_data(
            market_id=5,
            num_outcomes=2,
            status=MarketStatus.ACTIVE,
            winning_outcome=255,
            has_winning=False,
            bump=253,
            oracle=oracle,
            question_id=question_id,
            condition_id=condition_id,
        )

        market = deserialize_market(data)

        assert market.market_id == 5
        assert market.num_outcomes == 2
        assert market.status == MarketStatus.ACTIVE
        assert market.winning_outcome is None
        assert market.bump == 253
        assert market.oracle == oracle
        assert market.question_id == question_id
        assert market.condition_id == condition_id

    def test_deserialize_resolved_market(self):
        data = build_market_data(
            market_id=10,
            num_outcomes=3,
            status=MarketStatus.RESOLVED,
            winning_outcome=1,
            has_winning=True,
            bump=252,
            oracle=Pubkey.new_unique(),
            question_id=bytes(32),
            condition_id=bytes(32),
        )

        market = deserialize_market(data)

        assert market.status == MarketStatus.RESOLVED
        assert market.winning_outcome == 1

    def test_invalid_discriminator(self):
        data = b"baddisc!" + bytes(112)

        with pytest.raises(InvalidDiscriminatorError):
            deserialize_market(data)


class TestDeserializePosition:
    def test_deserialize_valid_data(self):
        owner = Pubkey.new_unique()
        market = Pubkey.new_unique()
        data = build_position_data(owner, market, 251)

        position = deserialize_position(data)

        assert position.owner == owner
        assert position.market == market
        assert position.bump == 251

    def test_invalid_discriminator(self):
        data = b"notposit" + bytes(72)

        with pytest.raises(InvalidDiscriminatorError):
            deserialize_position(data)


class TestDeserializeOrderStatus:
    def test_deserialize_active_order(self):
        data = build_order_status_data(1000000, False)

        order_status = deserialize_order_status(data)

        assert order_status.remaining == 1000000
        assert order_status.is_cancelled is False

    def test_deserialize_cancelled_order(self):
        data = build_order_status_data(500000, True)

        order_status = deserialize_order_status(data)

        assert order_status.remaining == 500000
        assert order_status.is_cancelled is True

    def test_invalid_discriminator(self):
        data = b"notorder" + bytes(16)

        with pytest.raises(InvalidDiscriminatorError):
            deserialize_order_status(data)


class TestDeserializeUserNonce:
    def test_deserialize_valid_data(self):
        data = build_user_nonce_data(42)

        user_nonce = deserialize_user_nonce(data)

        assert user_nonce.nonce == 42

    def test_deserialize_zero_nonce(self):
        data = build_user_nonce_data(0)

        user_nonce = deserialize_user_nonce(data)

        assert user_nonce.nonce == 0

    def test_deserialize_large_nonce(self):
        data = build_user_nonce_data(2**64 - 1)

        user_nonce = deserialize_user_nonce(data)

        assert user_nonce.nonce == 2**64 - 1

    def test_invalid_discriminator(self):
        data = b"badnonce" + bytes(8)

        with pytest.raises(InvalidDiscriminatorError):
            deserialize_user_nonce(data)
