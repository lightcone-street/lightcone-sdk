"""Exact-order construction and signed-range regression tests."""

import asyncio
from copy import deepcopy
from dataclasses import replace
from types import SimpleNamespace

import pytest
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from lightcone_sdk.domain.order.client import Orders as OrderClient
from lightcone_sdk.domain.orderbook.client import Orderbooks
from lightcone_sdk.domain.orderbook.wire import DecimalsResponse, OrderbookDepthResponse
from lightcone_sdk.error import DeserializationError
from lightcone_sdk.program.envelope import LimitOrderEnvelope
from lightcone_sdk.program.errors import SerializationError
from lightcone_sdk.program.orders import generate_salt, sign_order
from lightcone_sdk.program.types import OrderSide, SignedOrder
from lightcone_sdk.shared.rejection import RejectionCode
from lightcone_sdk.shared.scaling import (
    I64_MAX,
    U64_MAX,
    ScalingError,
    scale_price_size,
    validate_raw_amounts,
    validate_signed_fields,
)
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
    validate_signed_fields(U64_MAX, U64_MAX)
    validate_signed_fields(1, 1)
    for amount_in, amount_out in ((0, 1), (1, 0), (U64_MAX + 1, 1), (True, 1)):
        with pytest.raises(ScalingError, match="ORDER_FIELD_OUT_OF_RANGE"):
            validate_signed_fields(amount_in, amount_out)
    salts = [generate_salt() for _ in range(256)]
    assert all(0 <= salt <= U64_MAX for salt in salts)
    assert any(salt > I64_MAX for salt in salts), "256 draws never exceeded i64::MAX"


def test_signing_rejects_unencodable_salts_and_inexact_ratios():
    keypair = Keypair.from_seed(bytes(range(32)))
    order = SignedOrder(
        salt=U64_MAX + 1,
        maker=keypair.pubkey(),
        market=Pubkey.from_bytes(bytes([1]) * 32),
        base_mint=Pubkey.from_bytes(bytes([2]) * 32),
        quote_mint=Pubkey.from_bytes(bytes([3]) * 32),
        side=OrderSide.BID,
        amount_in=15_185_088,
        amount_out=123_456_000,
        expiration=0,
    )
    with pytest.raises(SerializationError, match="u64 value out of range"):
        sign_order(order, keypair, RULES)
    assert order.signature == bytes(64)
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
        orderbook_id = RULES_WIRE["orderbook_id"]
        first, second = await asyncio.gather(
            books.decimals(orderbook_id), books.decimals(orderbook_id)
        )
        assert first is second

    asyncio.run(discover())
    assert Client._http.calls == 1


def test_mismatched_rules_are_rejected_and_not_cached():
    wrong = deepcopy(RULES_WIRE)
    wrong["orderbook_id"] = "wrong"
    correct = deepcopy(RULES_WIRE)
    correct["orderbook_id"] = "ob"

    class Http:
        calls = 0
        responses = [wrong, correct]

        async def get(self, _path, **_kwargs):
            self.calls += 1
            return self.responses.pop(0)

    class Client:
        _http = Http()

    books = Orderbooks(Client())

    async def discover():
        with pytest.raises(ScalingError, match="cannot be used for 'ob'"):
            await books.decimals("ob")

        rules = await books.decimals("ob")
        assert rules.orderbook_id == "ob"
        assert await books.decimals("ob") is rules

    asyncio.run(discover())
    assert Client._http.calls == 2


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
    ):
        parsed = RejectionCode.from_wire(code)
        assert parsed is not None and parsed.is_known()
        assert parsed.wire_name() == code
