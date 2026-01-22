"""Tests for order operations."""

import pytest
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from src.program import (
    COMPACT_ORDER_SIZE,
    FULL_ORDER_SIZE,
    AskOrderParams,
    BidOrderParams,
    CompactOrder,
    FullOrder,
    OrderSide,
    InvalidOrderError,
    InvalidSignatureError,
    create_ask_order,
    create_bid_order,
    create_signed_ask_order,
    create_signed_bid_order,
    deserialize_compact_order,
    deserialize_full_order,
    hash_order,
    serialize_compact_order,
    serialize_full_order,
    sign_order,
    to_compact_order,
    validate_order,
    validate_signed_order,
    verify_order_signature,
)


@pytest.fixture
def sample_bid_params():
    return BidOrderParams(
        nonce=1,
        maker=Pubkey.new_unique(),
        market=Pubkey.new_unique(),
        base_mint=Pubkey.new_unique(),
        quote_mint=Pubkey.new_unique(),
        maker_amount=1000000,
        taker_amount=500000,
        expiration=1700000000,
    )


@pytest.fixture
def sample_ask_params():
    return AskOrderParams(
        nonce=2,
        maker=Pubkey.new_unique(),
        market=Pubkey.new_unique(),
        base_mint=Pubkey.new_unique(),
        quote_mint=Pubkey.new_unique(),
        maker_amount=500000,
        taker_amount=1000000,
        expiration=1700000000,
    )


class TestCreateBidOrder:
    def test_creates_bid_order(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)

        assert order.nonce == sample_bid_params.nonce
        assert order.maker == sample_bid_params.maker
        assert order.market == sample_bid_params.market
        assert order.base_mint == sample_bid_params.base_mint
        assert order.quote_mint == sample_bid_params.quote_mint
        assert order.side == OrderSide.BID
        assert order.maker_amount == sample_bid_params.maker_amount
        assert order.taker_amount == sample_bid_params.taker_amount
        assert order.expiration == sample_bid_params.expiration

    def test_signature_is_empty(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        assert order.signature == bytes(64)


class TestCreateAskOrder:
    def test_creates_ask_order(self, sample_ask_params):
        order = create_ask_order(sample_ask_params)

        assert order.side == OrderSide.ASK
        assert order.maker_amount == sample_ask_params.maker_amount
        assert order.taker_amount == sample_ask_params.taker_amount


class TestHashOrder:
    def test_produces_32_byte_hash(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        order_hash = hash_order(order)

        assert len(order_hash) == 32

    def test_same_order_produces_same_hash(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)

        hash1 = hash_order(order)
        hash2 = hash_order(order)

        assert hash1 == hash2

    def test_different_orders_produce_different_hashes(self):
        params1 = BidOrderParams(
            nonce=1,
            maker=Pubkey.new_unique(),
            market=Pubkey.new_unique(),
            base_mint=Pubkey.new_unique(),
            quote_mint=Pubkey.new_unique(),
            maker_amount=1000000,
            taker_amount=500000,
            expiration=1700000000,
        )
        params2 = BidOrderParams(
            nonce=2,  # Different nonce
            maker=params1.maker,
            market=params1.market,
            base_mint=params1.base_mint,
            quote_mint=params1.quote_mint,
            maker_amount=params1.maker_amount,
            taker_amount=params1.taker_amount,
            expiration=params1.expiration,
        )

        order1 = create_bid_order(params1)
        order2 = create_bid_order(params2)

        assert hash_order(order1) != hash_order(order2)

    def test_signature_does_not_affect_hash(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        hash_before = hash_order(order)

        order.signature = bytes([1] * 64)
        hash_after = hash_order(order)

        assert hash_before == hash_after


class TestSignOrder:
    def test_signs_order(self):
        keypair = Keypair()
        params = BidOrderParams(
            nonce=1,
            maker=keypair.pubkey(),
            market=Pubkey.new_unique(),
            base_mint=Pubkey.new_unique(),
            quote_mint=Pubkey.new_unique(),
            maker_amount=1000000,
            taker_amount=500000,
            expiration=1700000000,
        )
        order = create_bid_order(params)

        signature = sign_order(order, keypair)

        assert len(signature) == 64
        assert signature != bytes(64)
        assert order.signature == signature

    def test_signed_order_verifies(self):
        keypair = Keypair()
        params = BidOrderParams(
            nonce=1,
            maker=keypair.pubkey(),
            market=Pubkey.new_unique(),
            base_mint=Pubkey.new_unique(),
            quote_mint=Pubkey.new_unique(),
            maker_amount=1000000,
            taker_amount=500000,
            expiration=1700000000,
        )
        order = create_bid_order(params)
        sign_order(order, keypair)

        assert verify_order_signature(order) is True


class TestVerifyOrderSignature:
    def test_valid_signature_verifies(self):
        keypair = Keypair()
        params = BidOrderParams(
            nonce=1,
            maker=keypair.pubkey(),
            market=Pubkey.new_unique(),
            base_mint=Pubkey.new_unique(),
            quote_mint=Pubkey.new_unique(),
            maker_amount=1000000,
            taker_amount=500000,
            expiration=1700000000,
        )
        order = create_signed_bid_order(params, keypair)

        assert verify_order_signature(order) is True

    def test_invalid_signature_fails(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        order.signature = bytes([1] * 64)  # Invalid signature

        assert verify_order_signature(order) is False

    def test_tampered_order_fails(self):
        keypair = Keypair()
        params = BidOrderParams(
            nonce=1,
            maker=keypair.pubkey(),
            market=Pubkey.new_unique(),
            base_mint=Pubkey.new_unique(),
            quote_mint=Pubkey.new_unique(),
            maker_amount=1000000,
            taker_amount=500000,
            expiration=1700000000,
        )
        order = create_signed_bid_order(params, keypair)

        # Tamper with the order
        order.maker_amount = 2000000

        assert verify_order_signature(order) is False


class TestSerializeFullOrder:
    def test_produces_correct_size(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        data = serialize_full_order(order)

        assert len(data) == FULL_ORDER_SIZE

    def test_round_trip(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        data = serialize_full_order(order)
        restored = deserialize_full_order(data)

        assert restored.nonce == order.nonce
        assert restored.maker == order.maker
        assert restored.market == order.market
        assert restored.base_mint == order.base_mint
        assert restored.quote_mint == order.quote_mint
        assert restored.side == order.side
        assert restored.maker_amount == order.maker_amount
        assert restored.taker_amount == order.taker_amount
        assert restored.expiration == order.expiration
        assert restored.signature == order.signature


class TestSerializeCompactOrder:
    def test_produces_correct_size(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        compact = to_compact_order(order)
        data = serialize_compact_order(compact)

        assert len(data) == COMPACT_ORDER_SIZE

    def test_round_trip(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        compact = to_compact_order(order)
        data = serialize_compact_order(compact)
        restored = deserialize_compact_order(data)

        assert restored.nonce == compact.nonce
        assert restored.maker == compact.maker
        assert restored.side == compact.side
        assert restored.maker_amount == compact.maker_amount
        assert restored.taker_amount == compact.taker_amount
        assert restored.expiration == compact.expiration


class TestToCompactOrder:
    def test_converts_correctly(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        compact = to_compact_order(order)

        assert compact.nonce == order.nonce
        assert compact.maker == order.maker
        assert compact.side == order.side
        assert compact.maker_amount == order.maker_amount
        assert compact.taker_amount == order.taker_amount
        assert compact.expiration == order.expiration


class TestValidateOrder:
    def test_valid_order_passes(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        # Should not raise
        validate_order(order)

    def test_zero_maker_amount_fails(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        order.maker_amount = 0

        with pytest.raises(InvalidOrderError, match="maker_amount"):
            validate_order(order)

    def test_zero_taker_amount_fails(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        order.taker_amount = 0

        with pytest.raises(InvalidOrderError, match="taker_amount"):
            validate_order(order)


class TestValidateSignedOrder:
    def test_valid_signed_order_passes(self):
        keypair = Keypair()
        params = BidOrderParams(
            nonce=1,
            maker=keypair.pubkey(),
            market=Pubkey.new_unique(),
            base_mint=Pubkey.new_unique(),
            quote_mint=Pubkey.new_unique(),
            maker_amount=1000000,
            taker_amount=500000,
            expiration=1700000000,
        )
        order = create_signed_bid_order(params, keypair)

        # Should not raise
        validate_signed_order(order)

    def test_invalid_signature_fails(self, sample_bid_params):
        order = create_bid_order(sample_bid_params)
        order.signature = bytes([1] * 64)

        with pytest.raises(InvalidSignatureError):
            validate_signed_order(order)


class TestCreateSignedBidOrder:
    def test_creates_and_signs(self):
        keypair = Keypair()
        params = BidOrderParams(
            nonce=1,
            maker=keypair.pubkey(),
            market=Pubkey.new_unique(),
            base_mint=Pubkey.new_unique(),
            quote_mint=Pubkey.new_unique(),
            maker_amount=1000000,
            taker_amount=500000,
            expiration=1700000000,
        )

        order = create_signed_bid_order(params, keypair)

        assert order.side == OrderSide.BID
        assert order.signature != bytes(64)
        assert verify_order_signature(order) is True


class TestCreateSignedAskOrder:
    def test_creates_and_signs(self):
        keypair = Keypair()
        params = AskOrderParams(
            nonce=1,
            maker=keypair.pubkey(),
            market=Pubkey.new_unique(),
            base_mint=Pubkey.new_unique(),
            quote_mint=Pubkey.new_unique(),
            maker_amount=500000,
            taker_amount=1000000,
            expiration=1700000000,
        )

        order = create_signed_ask_order(params, keypair)

        assert order.side == OrderSide.ASK
        assert verify_order_signature(order) is True
