"""Tests for types module."""

import pytest
from solders.pubkey import Pubkey

from src import (
    Exchange,
    FullOrder,
    Market,
    MarketStatus,
    OrderSide,
    OrderStatus,
    OutcomeMetadata,
    Position,
    UserNonce,
)


class TestMarketStatus:
    def test_pending_value(self):
        assert MarketStatus.PENDING == 0

    def test_active_value(self):
        assert MarketStatus.ACTIVE == 1

    def test_resolved_value(self):
        assert MarketStatus.RESOLVED == 2

    def test_cancelled_value(self):
        assert MarketStatus.CANCELLED == 3

    def test_from_int(self):
        assert MarketStatus(0) == MarketStatus.PENDING
        assert MarketStatus(1) == MarketStatus.ACTIVE
        assert MarketStatus(2) == MarketStatus.RESOLVED
        assert MarketStatus(3) == MarketStatus.CANCELLED


class TestOrderSide:
    def test_bid_value(self):
        assert OrderSide.BID == 0

    def test_ask_value(self):
        assert OrderSide.ASK == 1

    def test_from_int(self):
        assert OrderSide(0) == OrderSide.BID
        assert OrderSide(1) == OrderSide.ASK


class TestExchange:
    def test_create_exchange(self):
        authority = Pubkey.new_unique()
        operator = Pubkey.new_unique()

        exchange = Exchange(
            authority=authority,
            operator=operator,
            market_count=5,
            paused=False,
            bump=255,
        )

        assert exchange.authority == authority
        assert exchange.operator == operator
        assert exchange.market_count == 5
        assert exchange.paused is False
        assert exchange.bump == 255


class TestMarket:
    def test_create_market(self):
        oracle = Pubkey.new_unique()
        question_id = bytes(32)
        condition_id = bytes(32)

        market = Market(
            market_id=42,
            num_outcomes=2,
            status=MarketStatus.ACTIVE,
            winning_outcome=None,
            bump=254,
            oracle=oracle,
            question_id=question_id,
            condition_id=condition_id,
        )

        assert market.market_id == 42
        assert market.num_outcomes == 2
        assert market.status == MarketStatus.ACTIVE
        assert market.winning_outcome is None
        assert market.bump == 254
        assert market.oracle == oracle


class TestPosition:
    def test_create_position(self):
        owner = Pubkey.new_unique()
        market = Pubkey.new_unique()

        position = Position(
            owner=owner,
            market=market,
            bump=253,
        )

        assert position.owner == owner
        assert position.market == market
        assert position.bump == 253


class TestOrderStatus:
    def test_create_order_status(self):
        status = OrderStatus(
            remaining=1000000,
            is_cancelled=False,
        )

        assert status.remaining == 1000000
        assert status.is_cancelled is False


class TestUserNonce:
    def test_create_user_nonce(self):
        nonce = UserNonce(nonce=42)
        assert nonce.nonce == 42


class TestFullOrder:
    def test_create_full_order(self):
        maker = Pubkey.new_unique()
        market = Pubkey.new_unique()
        base_mint = Pubkey.new_unique()
        quote_mint = Pubkey.new_unique()

        order = FullOrder(
            nonce=1,
            maker=maker,
            market=market,
            base_mint=base_mint,
            quote_mint=quote_mint,
            side=OrderSide.BID,
            maker_amount=1000000,
            taker_amount=500000,
            expiration=1700000000,
        )

        assert order.nonce == 1
        assert order.maker == maker
        assert order.market == market
        assert order.base_mint == base_mint
        assert order.quote_mint == quote_mint
        assert order.side == OrderSide.BID
        assert order.maker_amount == 1000000
        assert order.taker_amount == 500000
        assert order.expiration == 1700000000
        assert len(order.signature) == 64
        assert order.signature == bytes(64)


class TestOutcomeMetadata:
    def test_create_outcome_metadata(self):
        metadata = OutcomeMetadata(
            name="Yes",
            symbol="YES",
            uri="https://example.com/yes.json",
        )

        assert metadata.name == "Yes"
        assert metadata.symbol == "YES"
        assert metadata.uri == "https://example.com/yes.json"
