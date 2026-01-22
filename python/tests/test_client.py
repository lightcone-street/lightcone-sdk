"""Tests for the client module."""

import pytest
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from src import (
    PROGRAM_ID,
    AskOrderParams,
    BidOrderParams,
    LightconePinocchioClient,
    OrderSide,
    hash_order,
    verify_order_signature,
)


class MockConnection:
    """Mock Solana connection for testing."""

    async def get_account_info(self, pubkey):
        return MockResponse(None)

    async def get_latest_blockhash(self):
        from solders.hash import Hash

        return MockResponse(MockBlockhash(Hash.default()))


class MockResponse:
    def __init__(self, value):
        self.value = value


class MockBlockhash:
    def __init__(self, blockhash):
        self.blockhash = blockhash


@pytest.fixture
def client():
    return LightconePinocchioClient(MockConnection())


class TestClientInit:
    def test_default_program_id(self):
        client = LightconePinocchioClient(MockConnection())
        assert client.program_id == PROGRAM_ID

    def test_custom_program_id(self):
        custom_id = Pubkey.new_unique()
        client = LightconePinocchioClient(MockConnection(), custom_id)
        assert client.program_id == custom_id


class TestClientOrderHelpers:
    def test_create_bid_order(self, client):
        params = BidOrderParams(
            nonce=1,
            maker=Pubkey.new_unique(),
            market=Pubkey.new_unique(),
            base_mint=Pubkey.new_unique(),
            quote_mint=Pubkey.new_unique(),
            maker_amount=1000000,
            taker_amount=500000,
            expiration=1700000000,
        )

        order = client.create_bid_order(params)

        assert order.side == OrderSide.BID
        assert order.nonce == params.nonce
        assert order.maker_amount == params.maker_amount

    def test_create_ask_order(self, client):
        params = AskOrderParams(
            nonce=1,
            maker=Pubkey.new_unique(),
            market=Pubkey.new_unique(),
            base_mint=Pubkey.new_unique(),
            quote_mint=Pubkey.new_unique(),
            maker_amount=500000,
            taker_amount=1000000,
            expiration=1700000000,
        )

        order = client.create_ask_order(params)

        assert order.side == OrderSide.ASK

    def test_create_signed_bid_order(self, client):
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

        order = client.create_signed_bid_order(params, keypair)

        assert order.signature != bytes(64)
        assert verify_order_signature(order)

    def test_create_signed_ask_order(self, client):
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

        order = client.create_signed_ask_order(params, keypair)

        assert verify_order_signature(order)

    def test_hash_order(self, client):
        params = BidOrderParams(
            nonce=1,
            maker=Pubkey.new_unique(),
            market=Pubkey.new_unique(),
            base_mint=Pubkey.new_unique(),
            quote_mint=Pubkey.new_unique(),
            maker_amount=1000000,
            taker_amount=500000,
            expiration=1700000000,
        )
        order = client.create_bid_order(params)

        order_hash = client.hash_order(order)

        assert len(order_hash) == 32
        assert order_hash == hash_order(order)


class TestClientUtilityMethods:
    def test_derive_condition_id(self, client):
        oracle = Pubkey.new_unique()
        question_id = bytes(32)

        condition_id = client.derive_condition_id(oracle, question_id, 2)

        assert len(condition_id) == 32

    def test_get_conditional_mints(self, client):
        market = Pubkey.new_unique()
        deposit_mint = Pubkey.new_unique()

        mints = client.get_conditional_mints(market, deposit_mint, 3)

        assert len(mints) == 3
        assert all(isinstance(m, Pubkey) for m in mints)

    def test_get_exchange_address(self, client):
        address = client.get_exchange_address()
        assert isinstance(address, Pubkey)

    def test_get_market_address(self, client):
        address = client.get_market_address(0)
        assert isinstance(address, Pubkey)

        # Different market IDs should produce different addresses
        address1 = client.get_market_address(1)
        assert address != address1

    def test_get_position_address(self, client):
        owner = Pubkey.new_unique()
        market = Pubkey.new_unique()

        address = client.get_position_address(owner, market)

        assert isinstance(address, Pubkey)

    def test_get_order_status_address(self, client):
        order_hash = bytes(32)

        address = client.get_order_status_address(order_hash)

        assert isinstance(address, Pubkey)

    def test_get_user_nonce_address(self, client):
        user = Pubkey.new_unique()

        address = client.get_user_nonce_address(user)

        assert isinstance(address, Pubkey)
