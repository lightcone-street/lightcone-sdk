"""Tests for types module."""

from solders.pubkey import Pubkey

from lightcone_sdk import (
    Exchange,
    FullOrder,
    Market,
    MarketStatus,
    OrderSide,
    OrderStatus,
    OutcomeMetadata,
    Position,
    ScalarResolutionParams,
    SettleMarketParams,
    UserNonce,
    scalar_to_payout_numerators,
    winner_takes_all_payout_numerators,
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
        manager = Pubkey.new_unique()

        exchange = Exchange(
            authority=authority,
            operator=operator,
            manager=manager,
            market_count=5,
            paused=False,
            bump=255,
        )

        assert exchange.authority == authority
        assert exchange.operator == operator
        assert exchange.manager == manager
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
            bump=254,
            oracle=oracle,
            question_id=question_id,
            condition_id=condition_id,
            payout_numerators=(0, 0, 0, 0, 0, 0),
            payout_denominator=0,
        )

        assert market.market_id == 42
        assert market.num_outcomes == 2
        assert market.status == MarketStatus.ACTIVE
        assert market.bump == 254
        assert market.oracle == oracle
        assert market.payout_numerators == (0, 0, 0, 0, 0, 0)
        assert market.payout_denominator == 0


class TestMarketResolution:
    def test_winner_takes_all_payout_numerators(self):
        assert winner_takes_all_payout_numerators(2, 4) == [0, 0, 1, 0]

    def test_settle_market_params_winner_takes_all(self):
        oracle = Pubkey.new_unique()
        params = SettleMarketParams.winner_takes_all(
            oracle=oracle,
            market_id=7,
            winning_outcome=1,
            num_outcomes=3,
        )

        assert params.oracle == oracle
        assert params.market_id == 7
        assert params.payout_numerators == [0, 1, 0]

    def test_scalar_to_payout_numerators(self):
        assert scalar_to_payout_numerators(
            ScalarResolutionParams(
                min_value=0,
                max_value=100,
                resolved_value=25,
                lower_outcome_index=0,
                upper_outcome_index=1,
                num_outcomes=2,
            )
        ) == [3, 1]
        assert scalar_to_payout_numerators(
            ScalarResolutionParams(
                min_value=0,
                max_value=100,
                resolved_value=-5,
                lower_outcome_index=0,
                upper_outcome_index=1,
                num_outcomes=2,
            )
        ) == [1, 0]
        assert scalar_to_payout_numerators(
            ScalarResolutionParams(
                min_value=0,
                max_value=100,
                resolved_value=120,
                lower_outcome_index=0,
                upper_outcome_index=1,
                num_outcomes=2,
            )
        ) == [0, 1]
        assert scalar_to_payout_numerators(
            ScalarResolutionParams(
                min_value=-10_000,
                max_value=40_000,
                resolved_value=15_250,
                lower_outcome_index=0,
                upper_outcome_index=1,
                num_outcomes=2,
            )
        ) == [99, 101]


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
            base_remaining=750000,
            is_cancelled=False,
        )

        assert status.remaining == 1000000
        assert status.base_remaining == 750000
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
            amount_in=1000000,
            amount_out=500000,
            expiration=1700000000,
        )

        assert order.nonce == 1
        assert order.maker == maker
        assert order.market == market
        assert order.base_mint == base_mint
        assert order.quote_mint == quote_mint
        assert order.side == OrderSide.BID
        assert order.amount_in == 1000000
        assert order.amount_out == 500000
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
