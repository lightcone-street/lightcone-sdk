"""Exact-order construction and signed-range regression tests."""

import asyncio
from copy import deepcopy
from dataclasses import replace
from types import SimpleNamespace

import pytest
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from lightcone_sdk.error import DeserializationError
from lightcone_sdk.domain.orderbook.client import Orderbooks
from lightcone_sdk.domain.order.client import Orders as OrderClient
from lightcone_sdk.domain.orderbook.wire import DecimalsResponse, OrderbookDepthResponse
from lightcone_sdk.program.orders import (
    generate_salt,
    hash_order_hex,
    serialize_order_for_hashing,
    sign_order,
)
from lightcone_sdk.program.errors import InvalidOrderError
from lightcone_sdk.program.types import OrderSide, SignedOrder
from lightcone_sdk.program.envelope import LimitOrderEnvelope
from lightcone_sdk.shared.scaling import (
    I64_MAX,
    ScalingError,
    scale_price_size,
    validate_raw_amounts,
    validate_signed_fields,
)
from lightcone_sdk.shared.rejection import RejectionCode
from lightcone_sdk.shared.types import SubmitOrderRequest


RULES_WIRE = {
    "orderbook_id": "11111111111111111111111111111111",
    "base_decimals": 8,
    "quote_decimals": 6,
    "price_decimals": 4,
    "trading_rules": {
        "base_size_decimals": 5,
        "max_price_decimals": 1,
        "max_price_significant_figures": 5,
        "integer_prices_always_allowed": True,
        "price_quantum": "0.1000",
        "price_quantum_raw": "1000",
        "base_size_quantum": "0.00001000",
        "base_size_quantum_raw": "1000",
    },
}
RULES = DecimalsResponse.from_dict(RULES_WIRE).to_rules()

VALID_ORDERS = (
    (OrderSide.BID, "12.3", "1.23456", 15_185_088, 123_456_000),
    (OrderSide.ASK, "12.3", "1.23456", 123_456_000, 15_185_088),
    (OrderSide.BID, "150250", "1", 150_250_000_000, 100_000_000),
)
INVALID_ORDERS = (
    ("12.34", "1.23456", "INVALID_PRICE_DECIMALS"),
    ("12.3", "1.234567", "INVALID_SIZE_DECIMALS"),
    ("150250.1", "1", "INVALID_PRICE_SIGNIFICANT_FIGURES"),
)

SIGNING_CASE = {
    "seed_hex": "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
    "maker": "FAe4sisG95oZ42w7buUn5qEE4TAnfTTFPiguZUHmhiF",
    "market": "4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi",
    "base_mint": "8qbHbw2BbbTHBW1sbeqakYXVKRQM8Ne7pLK7m6CVfeR",
    "quote_mint": "CktRuQ2mttgRGkXJtyksdKHjUdc2C4TgDzyB98oEzy8",
    "hash_hex": "17228fe4bdf93c14714367454e948206bb4f001917d59e132e4aaad097819eac",
    "signature_hex": "1e68fe672f919085ed34333c86facf9ad816ae30ab23d3d6ccef0aeb4c40b161f841c8f68355c9720252fc9ab4d859e64416d81ffa170e5a6cd17dfe41038808",
}


def test_valid_and_invalid_precision_cases():
    for side, price, size, amount_in, amount_out in VALID_ORDERS:
        scaled = scale_price_size(price, size, int(side), RULES)
        assert scaled.amount_in == amount_in
        assert scaled.amount_out == amount_out

    for price, size, code in INVALID_ORDERS:
        with pytest.raises(ScalingError) as error:
            scale_price_size(price, size, int(OrderSide.BID), RULES)
        assert error.value.code == code


def test_raw_ratio_ranges_and_salts():
    with pytest.raises(ScalingError, match="PRICE_NOT_EXACTLY_REPRESENTABLE"):
        validate_raw_amounts(1, 3_000, int(OrderSide.BID), RULES)
    validate_signed_fields(I64_MAX, I64_MAX, I64_MAX, 2**32 - 1)
    with pytest.raises(ScalingError):
        validate_signed_fields(I64_MAX + 1, 1, 0, 0)
    assert all(0 <= generate_salt() <= I64_MAX for _ in range(10_000))


def test_known_signing_contract_is_unchanged():
    keypair = Keypair.from_seed(bytes.fromhex(SIGNING_CASE["seed_hex"]))
    order = SignedOrder(
        nonce=42,
        salt=123,
        maker=keypair.pubkey(),
        market=Pubkey.from_string(SIGNING_CASE["market"]),
        base_mint=Pubkey.from_string(SIGNING_CASE["base_mint"]),
        quote_mint=Pubkey.from_string(SIGNING_CASE["quote_mint"]),
        side=OrderSide.BID,
        amount_in=15_185_088,
        amount_out=123_456_000,
        expiration=0,
    )
    assert str(keypair.pubkey()) == SIGNING_CASE["maker"]
    assert len(serialize_order_for_hashing(order)) == 169
    assert hash_order_hex(order) == SIGNING_CASE["hash_hex"]
    sign_order(order, keypair, RULES)
    assert order.signature.hex() == SIGNING_CASE["signature_hex"]
    order.salt = I64_MAX + 1
    with pytest.raises(InvalidOrderError):
        sign_order(order, keypair, RULES)
    order.salt = 0
    order.amount_in = 1
    order.amount_out = 3_000
    with pytest.raises(ScalingError, match="PRICE_NOT_EXACTLY_REPRESENTABLE"):
        sign_order(order, keypair, RULES)


def test_high_level_signed_order_path_requires_and_applies_rules():
    keypair = Keypair.from_seed(bytes([9]) * 32)
    pair = SimpleNamespace(
        orderbook_id=RULES_WIRE["orderbook_id"],
        market_pubkey=str(Pubkey.from_bytes(bytes([1]) * 32)),
        base=SimpleNamespace(pubkey=str(Pubkey.from_bytes(bytes([2]) * 32))),
        quote=SimpleNamespace(pubkey=str(Pubkey.from_bytes(bytes([3]) * 32))),
    )
    request = (
        LimitOrderEnvelope()
        .nonce(1)
        .salt(0)
        .maker(keypair.pubkey())
        .bid()
        .price("12.3")
        .size("1.23456")
        .sign(keypair, pair, RULES)
    )
    assert request.amount_in == 15_185_088
    assert request.amount_out == 123_456_000

    with pytest.raises(ScalingError, match="cannot be used"):
        (
            LimitOrderEnvelope()
            .nonce(1)
            .salt(0)
            .maker(keypair.pubkey())
            .bid()
            .price("12.3")
            .size("1.23456")
            .sign(keypair, pair, replace(RULES, orderbook_id="another-orderbook"))
        )

    with pytest.raises(ScalingError, match="PRICE_NOT_EXACTLY_REPRESENTABLE"):
        (
            LimitOrderEnvelope()
            .nonce(1)
            .salt(0)
            .maker(keypair.pubkey())
            .bid()
            .amount_in(1)
            .amount_out(3_000)
            .sign(keypair, pair, RULES)
        )


def test_depth_metadata_defaults_and_rules_cache_deduplicates():
    depth = OrderbookDepthResponse.from_dict(
        {
            "orderbook_id": "ob",
            "best_bid": None,
            "best_ask": None,
            "bids": [],
            "asks": [],
            "price_quantum": "0.1000",
            "trading_rules": RULES_WIRE["trading_rules"],
            "revision": 1842,
            "captured_at_ms": 1785776400123,
            "decimals": {"price": 4, "size": 8},
        }
    )
    assert depth.bids_truncated is False
    assert depth.asks_truncated is False
    assert depth.revision == 1842
    assert depth.captured_at_ms == 1785776400123

    class Http:
        calls = 0

        async def get(self, path, **_kwargs):
            self.calls += 1
            await asyncio.sleep(0)
            return RULES_WIRE

    class Client:
        _http = Http()

    books = Orderbooks(Client())

    async def discover():
        first, second = await asyncio.gather(books.decimals("ob"), books.decimals("ob"))
        assert first is second

    asyncio.run(discover())
    assert Client._http.calls == 1


def test_raw_quantums_must_be_json_strings():
    invalid = deepcopy(RULES_WIRE)
    invalid["trading_rules"]["price_quantum_raw"] = 1000
    with pytest.raises(DeserializationError, match="decimal strings"):
        DecimalsResponse.from_dict(invalid)


def test_direct_submit_preflights_before_http():
    class Http:
        calls = 0

        async def post(self, *_args, **_kwargs):
            self.calls += 1
            return {}

    class RuleClient:
        async def decimals(self, _orderbook_id):
            return RULES

    class Client:
        _http = Http()

        def orderbooks(self):
            return RuleClient()

    request = SubmitOrderRequest(
        maker="maker",
        nonce=0,
        salt=0,
        market_pubkey="market",
        base_token="base",
        quote_token="quote",
        side=int(OrderSide.BID),
        amount_in=1,
        amount_out=3_000,
        expiration=0,
        signature="signature",
        orderbook_id="ob",
    )

    async def submit():
        with pytest.raises(ScalingError, match="PRICE_NOT_EXACTLY_REPRESENTABLE"):
            await OrderClient(Client()).submit(request)

    asyncio.run(submit())
    assert Client._http.calls == 0


def test_new_rejection_codes_are_stable_known_values():
    for code in (
        "TRADING_RULES_UNAVAILABLE",
        "ORDER_FIELD_OUT_OF_RANGE",
        "PRICE_NOT_EXACTLY_REPRESENTABLE",
        "PRICE_OUT_OF_RANGE",
        "INVALID_PRICE_DECIMALS",
        "INVALID_PRICE_SIGNIFICANT_FIGURES",
        "INVALID_SIZE_DECIMALS",
        "TRIGGER_PRICE_OUT_OF_RANGE",
    ):
        parsed = RejectionCode.from_wire(code)
        assert parsed is not None and parsed.is_known()
        assert parsed.wire_name() == code
