"""Nonce-free order identity: signing vectors, encodings, preflight, and envelopes."""

import asyncio
import json
from types import SimpleNamespace

import base58
import pytest
from nacl.signing import SigningKey, VerifyKey
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from lightcone_sdk.error import ValidationError
from lightcone_sdk.program.builder import OrderBuilder
from lightcone_sdk.program.constants import (
    ORDER_PREIMAGE_SIZE,
    ORDER_SIZE,
    SIGNED_ORDER_SIZE,
)
from lightcone_sdk.program.envelope import LimitOrderEnvelope
from lightcone_sdk.program.errors import (
    ArithmeticOverflowError,
    InvalidDataLengthError,
    InvalidSideError,
    InvalidSignatureError,
    MissingFieldError,
    SerializationError,
    UnsignedOrderError,
)
from lightcone_sdk.program.orders import (
    apply_signature,
    calculate_taker_fill,
    cancel_all_message,
    cancel_order_message,
    deserialize_full_order,
    deserialize_order,
    generate_salt,
    get_order_message,
    hash_order_hex,
    is_order_expired,
    order_preimage,
    order_to_signed,
    serialize_full_order,
    serialize_order,
    sign_cancel_all,
    sign_cancel_order,
    sign_order,
    to_order,
    to_submit_request,
    verify_order_signature,
)
from lightcone_sdk.program.types import OrderSide, SignedOrder
from lightcone_sdk.shared.scaling import (
    I64_MAX,
    U64_MAX,
    OrderbookRules,
    ScalingError,
    TradingRules,
    scale_price_size,
    validate_raw_amounts,
    validate_signed_fields,
)
from lightcone_sdk.shared.signing import ExternalSigner, SigningStrategy
from lightcone_sdk.shared.types import DepositSource, SubmitOrderRequest, TimeInForce


def unit_rules(orderbook_id: str = "unit") -> OrderbookRules:
    """Unit rules: whole atoms, six price decimals, every positive ratio on the grid."""
    return OrderbookRules(
        orderbook_id=orderbook_id,
        base_decimals=0,
        quote_decimals=0,
        price_decimals=6,
        trading_rules=TradingRules(
            base_size_decimals=0,
            max_price_decimals=6,
            max_price_significant_figures=5,
            integer_prices_always_allowed=True,
            price_quantum="0.000001",
            price_quantum_raw=1,
            base_size_quantum="1",
            base_size_quantum_raw=1,
        ),
    )


# Backend signing vectors (fixtures/program-contract/v1/signing.json),
# generated independently of the SDKs with Python struct, Keccak and Ed25519,
# and pinned by the Rust and TypeScript SDKs too. Every vector is signed by
# seed 00..1f, whose public key is the maker, on market [0x11; 32], base mint
# [0x22; 32] and quote mint [0x33; 32]. Amounts are raw atoms; expiration is
# Unix seconds.
SIGNING_SEED_HEX = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
SIGNING_MAKER_HEX = "03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8"
PREIMAGE_ACCOUNTS = SIGNING_MAKER_HEX + "11" * 32 + "22" * 32 + "33" * 32

SIGNING_VECTORS = [
    {
        "name": "bid",
        "salt": 72_623_859_790_382_856,
        "side": OrderSide.BID,
        "maker_amount": 60_000_000,
        "taker_amount": 100_000_000,
        "expiration": 0,
        "preimage_hex": "0807060504030201"
        + PREIMAGE_ACCOUNTS
        + "00008793030000000000e1f505000000000000000000000000",
        "order_id_hex": "497bbc21bdf8d3fb1187f0eda8f7a0f124dc97511e1f5068bbb7180b6b4a9bdc",
        "signature_hex": "28ee8645e9a4f1cd3b28f910531d2d7b29b36038f873f5e1517058c02cf4cc4b"
        "ec8400596d0e87766c3a527e4f2d04b4d0b592be656b91160e48b3b5c6a7e800",
        "compact_hex": "080706050403020100008793030000000000e1f505000000000000000000000000",
    },
    {
        "name": "ask",
        "salt": 9_833_440_827_789_222_417,
        "side": OrderSide.ASK,
        "maker_amount": 100_000_000,
        "taker_amount": 60_000_000,
        "expiration": 1_700_000_000,
        "preimage_hex": "1122334455667788"
        + PREIMAGE_ACCOUNTS
        + "0100e1f50500000000008793030000000000f1536500000000",
        "order_id_hex": "ee414532884ed99be69c82c9a99d4916ccf13bb17c955d2e979c43df6697fdfc",
        "signature_hex": "f7df8ad52408140f9418b0a9c3a5925ffe41d58f68899001769748eaa84a4f55"
        "f5afc47c8a2d241f38b63141012f8970fecc159dfad855c624c4def66cd2c80a",
        "compact_hex": "11223344556677880100e1f50500000000008793030000000000f1536500000000",
    },
    {
        "name": "zero_salt",
        "salt": 0,
        "side": OrderSide.BID,
        "maker_amount": 1,
        "taker_amount": 1,
        "expiration": 0,
        "preimage_hex": "0000000000000000"
        + PREIMAGE_ACCOUNTS
        + "00010000000000000001000000000000000000000000000000",
        "order_id_hex": "bddbbc63b3eca9b0c8ab96920b44d44239042684fa523f4d9e3e35d23228a77c",
        "signature_hex": "2a0e4bea081930c109b666e29b979bdcfcaddb1b169a2bed1f1df60ac9522fcd"
        "fee68652f00541151af5cc2ca9c288a79e6dab71904ee3f35efd4fb4caf43506",
        "compact_hex": "000000000000000000010000000000000001000000000000000000000000000000",
    },
    {
        "name": "max_salt",
        "salt": U64_MAX,
        "side": OrderSide.ASK,
        "maker_amount": U64_MAX,
        "taker_amount": U64_MAX,
        "expiration": I64_MAX,
        "preimage_hex": "ffffffffffffffff"
        + PREIMAGE_ACCOUNTS
        + "01ffffffffffffffffffffffffffffffffffffffffffffff7f",
        "order_id_hex": "f51ef152121aeb7380e5fd0e0f5574382fc9e1bda5cecff7b98d8ff938ad0dc2",
        "signature_hex": "a03310e35a4dbae73b111de3ffb233ffdf6b358fb2023ee5cd88989a8f8e92c7"
        "6eee9d37042eb3028e31cd6f17878c927b8a3da5598f2736e8ddd69a84c77a01",
        "compact_hex": "ffffffffffffffff01ffffffffffffffffffffffffffffffffffffffffffffff7f",
    },
]


def unsigned_order(**overrides) -> SignedOrder:
    fields = {
        "salt": 1,
        "maker": Pubkey.from_bytes(bytes([7]) * 32),
        "market": Pubkey.from_bytes(bytes([8]) * 32),
        "base_mint": Pubkey.from_bytes(bytes([9]) * 32),
        "quote_mint": Pubkey.from_bytes(bytes([10]) * 32),
        "side": OrderSide.BID,
        "amount_in": 100,
        "amount_out": 50,
        "expiration": 0,
    }
    fields.update(overrides)
    return SignedOrder(**fields)


def test_orderbook(orderbook_id: str = "unit") -> SimpleNamespace:
    return SimpleNamespace(
        orderbook_id=orderbook_id,
        market_pubkey=str(Pubkey.from_bytes(bytes([1]) * 32)),
        base=SimpleNamespace(pubkey=str(Pubkey.from_bytes(bytes([2]) * 32))),
        quote=SimpleNamespace(pubkey=str(Pubkey.from_bytes(bytes([3]) * 32))),
    )


test_orderbook.__test__ = False  # type: ignore[attr-defined]


class TestNonceFreeOrderIdentity:
    def test_matches_the_backend_signing_vectors(self) -> None:
        keypair = Keypair.from_seed(bytes.fromhex(SIGNING_SEED_HEX))
        assert bytes(keypair.pubkey()).hex() == SIGNING_MAKER_HEX

        for vector in SIGNING_VECTORS:
            name = vector["name"]
            unsigned = SignedOrder(
                salt=vector["salt"],
                maker=keypair.pubkey(),
                market=Pubkey.from_bytes(bytes([0x11]) * 32),
                base_mint=Pubkey.from_bytes(bytes([0x22]) * 32),
                quote_mint=Pubkey.from_bytes(bytes([0x33]) * 32),
                side=vector["side"],
                amount_in=vector["maker_amount"],
                amount_out=vector["taker_amount"],
                expiration=vector["expiration"],
            )
            assert order_preimage(unsigned).hex() == vector["preimage_hex"], name
            assert len(order_preimage(unsigned)) == ORDER_PREIMAGE_SIZE
            # Makers sign the order id's 64 lowercase hex ASCII bytes.
            assert hash_order_hex(unsigned) == vector["order_id_hex"], name
            assert get_order_message(unsigned) == vector["order_id_hex"].encode()

            signature = bytes.fromhex(vector["signature_hex"])
            signed = SignedOrder(**{**unsigned.__dict__, "signature": signature})
            assert serialize_order(to_order(signed)).hex() == vector["compact_hex"]
            assert verify_order_signature(signed), name

            # The 225-byte signed order is the preimage followed by the signature.
            signed_bytes = serialize_full_order(signed)
            assert len(signed_bytes) == SIGNED_ORDER_SIZE
            assert (
                signed_bytes.hex() == vector["preimage_hex"] + vector["signature_hex"]
            )
            assert serialize_full_order(deserialize_full_order(signed_bytes)) == (
                signed_bytes
            )
            expanded = order_to_signed(
                deserialize_order(bytes.fromhex(vector["compact_hex"])),
                signed.maker,
                signed.market,
                signed.base_mint,
                signed.quote_mint,
                signed.signature,
            )
            assert serialize_full_order(expanded) == signed_bytes, name

            # Ed25519 is deterministic, so re-signing reproduces the vector.
            sign_order(unsigned, keypair, unit_rules())
            assert unsigned.signature.hex() == vector["signature_hex"], name

    def test_decodes_serialized_orders_only_at_their_exact_lengths(self) -> None:
        assert (ORDER_PREIMAGE_SIZE, SIGNED_ORDER_SIZE, ORDER_SIZE) == (161, 225, 33)
        for length in (224, 226):
            with pytest.raises(InvalidDataLengthError):
                deserialize_full_order(bytes(length))
        for length in (32, 34):
            with pytest.raises(InvalidDataLengthError):
                deserialize_order(bytes(length))
        bad_side = bytearray(ORDER_SIZE)
        bad_side[8] = 2
        with pytest.raises(InvalidSideError):
            deserialize_order(bytes(bad_side))

    def test_encodes_and_decodes_only_bid_or_ask_sides(self) -> None:
        signed = unsigned_order(signature=bytes([1]) * 64)
        bad_side = bytearray(serialize_full_order(signed))
        bad_side[136] = 2
        with pytest.raises(InvalidSideError):
            deserialize_full_order(bytes(bad_side))
        for side in (2, 256, True, False):
            wrapping = unsigned_order(side=side)
            with pytest.raises(InvalidSideError):
                order_preimage(wrapping)
            with pytest.raises(InvalidSideError):
                serialize_order(to_order(wrapping))

    def test_rejects_non_integer_and_boolean_fields(self) -> None:
        for field in ("salt", "amount_in", "amount_out"):
            with pytest.raises(SerializationError, match="u64 value out of range"):
                order_preimage(unsigned_order(**{field: True}))
        with pytest.raises(SerializationError, match="i64 value out of range"):
            order_preimage(unsigned_order(expiration=False))
        for amount_in, amount_out in ((True, 1), (1, False)):
            with pytest.raises(ScalingError, match="ORDER_FIELD_OUT_OF_RANGE"):
                validate_signed_fields(amount_in, amount_out)

    def test_preflights_signed_amounts_against_rules_before_signing(self) -> None:
        keypair = Keypair()
        order = unsigned_order(
            maker=keypair.pubkey(), salt=U64_MAX, amount_in=1, amount_out=3_000
        )
        with pytest.raises(ScalingError) as error:
            sign_order(order, keypair, unit_rules())
        assert error.value.code == "PRICE_NOT_EXACTLY_REPRESENTABLE"
        order.amount_in = 0
        with pytest.raises(ScalingError) as error:
            sign_order(order, keypair, unit_rules())
        assert error.value.code == "ORDER_FIELD_OUT_OF_RANGE"
        assert "amount_in" in str(error.value)
        assert order.signature == bytes(64)

    def test_rejects_values_the_encoding_would_otherwise_wrap(self) -> None:
        with pytest.raises(SerializationError, match="u64 value out of range"):
            order_preimage(unsigned_order(salt=U64_MAX + 1))
        with pytest.raises(SerializationError, match="u64 value out of range"):
            order_preimage(unsigned_order(salt=-1))
        with pytest.raises(SerializationError, match="i64 value out of range"):
            order_preimage(unsigned_order(expiration=I64_MAX + 1))

    def test_draws_salts_from_the_full_u64_range(self) -> None:
        salts = [generate_salt() for _ in range(256)]
        assert all(0 <= salt <= U64_MAX for salt in salts)
        assert any(salt > I64_MAX for salt in salts), "256 draws never exceeded i64"

    def test_rounds_the_minimum_taker_fill_up(self) -> None:
        maker = unsigned_order(side=OrderSide.ASK, amount_in=100, amount_out=200)
        assert calculate_taker_fill(maker, 50) == 100
        # ceil(1 * 200 / 100) = 2 and ceil(3 * 200 / 101) = ceil(5.94) = 6.
        assert calculate_taker_fill(maker, 1) == 2
        maker.amount_in = 101
        assert calculate_taker_fill(maker, 3) == 6
        with pytest.raises(ArithmeticOverflowError):
            calculate_taker_fill(unsigned_order(amount_in=1, amount_out=U64_MAX), 2)
        for fill in (-5, U64_MAX + 1, True):
            with pytest.raises(ArithmeticOverflowError):
                calculate_taker_fill(maker, fill)
        # A zero amount_in overflows, as in Rust.
        with pytest.raises(ArithmeticOverflowError):
            calculate_taker_fill(unsigned_order(amount_in=0), 1)

    def test_expires_an_order_only_after_its_expiration_second(self) -> None:
        assert is_order_expired(unsigned_order(expiration=0), I64_MAX) is False
        expiring = unsigned_order(expiration=1_700_000_000)
        assert is_order_expired(expiring, 1_699_999_999) is False
        assert is_order_expired(expiring, 1_700_000_000) is False
        assert is_order_expired(expiring, 1_700_000_001) is True


class TestSignedAmountRanges:
    def test_accepts_any_nonzero_u64_amount(self) -> None:
        validate_signed_fields(U64_MAX, U64_MAX)
        validate_signed_fields(1, 1)
        with pytest.raises(ScalingError, match="ORDER_FIELD_OUT_OF_RANGE: amount_in"):
            validate_signed_fields(0, 1)
        with pytest.raises(ScalingError, match="ORDER_FIELD_OUT_OF_RANGE: amount_out"):
            validate_signed_fields(1, 0)
        with pytest.raises(ScalingError, match="ORDER_FIELD_OUT_OF_RANGE: amount_in"):
            validate_signed_fields(U64_MAX + 1, 1)

    def test_admits_amounts_above_i64_max(self) -> None:
        raw = validate_raw_amounts(U64_MAX, U64_MAX, int(OrderSide.ASK), unit_rules())
        assert raw.base_atoms == U64_MAX
        assert raw.price_raw == 1_000_000

        above_i64 = str(I64_MAX + 1)
        scaled = scale_price_size("1", above_i64, int(OrderSide.BID), unit_rules())
        assert scaled.amount_in == 1 << 63
        assert scaled.amount_out == 1 << 63
        with pytest.raises(
            ScalingError, match="ORDER_FIELD_OUT_OF_RANGE: quote amount"
        ):
            scale_price_size("2", above_i64, int(OrderSide.BID), unit_rules())


class TestSubmitRequest:
    def sample_request(self) -> SubmitOrderRequest:
        return SubmitOrderRequest(
            maker="maker",
            salt=U64_MAX,
            market_pubkey="market",
            base_token="base",
            quote_token="quote",
            side=1,
            amount_in=100,
            amount_out=50,
            expiration=1_700_000_000,
            signature="sig",
            orderbook_id="ob",
        )

    def test_serializes_only_the_backends_fields(self) -> None:
        assert json.dumps(self.sample_request().to_dict(), separators=(",", ":")) == (
            '{"maker":"maker","salt":18446744073709551615,"market_pubkey":"market",'
            '"base_token":"base","quote_token":"quote","side":1,"amount_in":100,'
            '"amount_out":50,"expiration":1700000000,"signature":"sig","orderbook_id":"ob"}'
        )

    def test_serializes_time_in_force_and_the_conditional_deposit_source(self) -> None:
        request = self.sample_request()
        request.time_in_force = TimeInForce.IOC
        request.deposit_source = DepositSource.MARKET
        body = request.to_dict()
        assert body["tif"] == "IOC"
        assert body["deposit_source"] == "conditional"
        assert len(body) == 13
        assert "nonce" not in body

    def test_encodes_a_signed_order_without_a_nonce_and_rejects_an_unsigned_one(
        self,
    ) -> None:
        keypair = Keypair()
        order = unsigned_order(maker=keypair.pubkey())
        sign_order(order, keypair, unit_rules())
        request = to_submit_request(order, "unit", time_in_force=TimeInForce.FOK)
        assert request.salt == order.salt
        assert request.signature == order.signature.hex()
        assert "nonce" not in request.to_dict()
        with pytest.raises(UnsignedOrderError):
            to_submit_request(unsigned_order(maker=keypair.pubkey()), "unit")
        order.salt = -1
        with pytest.raises(SerializationError, match="u64 value out of range"):
            to_submit_request(order, "unit")


def envelope(maker: Pubkey) -> LimitOrderEnvelope:
    return (
        LimitOrderEnvelope()
        .maker(maker)
        .market(Pubkey.from_bytes(bytes([4]) * 32))
        .base_mint(Pubkey.from_bytes(bytes([5]) * 32))
        .quote_mint(Pubkey.from_bytes(bytes([6]) * 32))
        .bid()
        .amount_in(1_000_000)
        .amount_out(500_000)
    )


class _Signer(ExternalSigner):
    def __init__(self, keypair: Keypair):
        self.keypair = keypair
        self.messages: list[bytes] = []

    async def sign_message(self, message: bytes) -> bytes:
        self.messages.append(message)
        return SigningKey(bytes(self.keypair)[:32]).sign(message).signature

    async def sign_transaction(self, tx_bytes: bytes) -> bytes:  # pragma: no cover
        raise NotImplementedError


class _Client:
    def __init__(self, strategy: SigningStrategy):
        self.strategy = strategy
        self.submitted: list[SubmitOrderRequest] = []

    def _require_signing_strategy(self) -> SigningStrategy:
        return self.strategy

    def orderbooks(self):
        async def decimals(_orderbook_id: str) -> OrderbookRules:
            return unit_rules()

        return SimpleNamespace(decimals=decimals)

    def orders(self):
        async def submit(request: SubmitOrderRequest):
            self.submitted.append(request)
            return "submitted"

        return SimpleNamespace(submit=submit)


class TestOrderEnvelope:
    def test_draws_an_unset_salt_once_and_reuses_it(self) -> None:
        order = envelope(Keypair().pubkey())
        assert order.get_salt is None
        first = order.payload()
        assert order.get_salt == first.salt
        second = order.payload()
        assert second.salt == first.salt
        assert hash_order_hex(second) == hash_order_hex(first)
        assert order.salt(7).payload().salt == 7

    def test_submits_the_hash_a_wallet_adapter_signed(self) -> None:
        keypair = Keypair()
        order = envelope(keypair.pubkey())
        # An external wallet signs the payload's hash before finalize().
        payload = order.payload()
        signature = SigningKey(bytes(keypair)[:32]).sign(get_order_message(payload))
        request = order.finalize(
            base58.b58encode(signature.signature).decode(),
            test_orderbook(),
            unit_rules(),
        )
        assert request.salt == payload.salt
        assert request.signature == signature.signature.hex()
        payload.signature = signature.signature
        assert verify_order_signature(payload)

    @pytest.mark.parametrize(
        "signature",
        [
            "1111",
            "0x" + "ab" * 64,
            base58.b58encode(bytes(64)).decode() + " ",
            "",
            "l" * 88,
        ],
    )
    def test_rejects_a_malformed_external_signature(self, signature: str) -> None:
        with pytest.raises(InvalidSignatureError):
            envelope(Keypair().pubkey()).finalize(
                signature, test_orderbook(), unit_rules()
            )

    def test_rejects_an_unencodable_salt_or_expiration_before_submission(self) -> None:
        signature = base58.b58encode(bytes([1]) * 64).decode()
        maker = Keypair().pubkey()
        with pytest.raises(SerializationError, match="u64 value out of range"):
            envelope(maker).salt(-1).finalize(signature, test_orderbook(), unit_rules())
        with pytest.raises(SerializationError, match="i64 value out of range"):
            envelope(maker).expiration(I64_MAX + 1).finalize(
                signature, test_orderbook(), unit_rules()
            )

    def test_reports_zero_amounts_as_missing_as_the_rust_sdk_does(self) -> None:
        maker = Keypair().pubkey()
        with pytest.raises(MissingFieldError, match="amount_in must be greater than 0"):
            envelope(maker).amount_in(0).amount_out(5).payload()
        with pytest.raises(
            MissingFieldError, match="amount_out must be greater than 0"
        ):
            envelope(maker).amount_in(5).amount_out(0).payload()

    def test_requires_a_side(self) -> None:
        order = (
            LimitOrderEnvelope().maker(Keypair().pubkey()).amount_in(5).amount_out(5)
        )
        with pytest.raises(
            MissingFieldError, match=r"side \(call .bid\(\) or .ask\(\)\)"
        ):
            order.sign(Keypair(), test_orderbook(), unit_rules())
        with pytest.raises(InvalidSideError):
            order.side(True)

    def test_derives_price_and_size_again_on_every_build(self) -> None:
        keypair = Keypair()
        order = (
            LimitOrderEnvelope().maker(keypair.pubkey()).bid().price("0.5").size("10")
        )
        first = order.sign(keypair, test_orderbook(), unit_rules())
        assert (first.amount_in, first.amount_out) == (5, 10)
        # Signing again unchanged resubmits the identical order.
        assert order.sign(keypair, test_orderbook(), unit_rules()) == first
        repriced = order.price("0.6").sign(keypair, test_orderbook(), unit_rules())
        assert (repriced.amount_in, repriced.amount_out) == (6, 10)
        ask = order.ask().sign(keypair, test_orderbook(), unit_rules())
        assert (ask.amount_in, ask.amount_out) == (10, 6)
        # Raw amounts replace derived ones and are kept as supplied.
        raw = (
            order.amount_in(7)
            .amount_out(14)
            .sign(keypair, test_orderbook(), unit_rules())
        )
        assert (raw.amount_in, raw.amount_out) == (7, 14)
        assert raw.salt == first.salt

    def test_never_returns_stale_derived_amounts(self) -> None:
        keypair = Keypair()
        order = (
            LimitOrderEnvelope().maker(keypair.pubkey()).bid().price("0.5").size("10")
        )
        order.sign(keypair, test_orderbook(), unit_rules())
        assert order.payload().amount_in == 5
        # A later price change clears the derived amounts until the next build,
        # as Rust's payload() requires amounts set by a build.
        order.price("0.6")
        with pytest.raises(MissingFieldError, match="amount_in"):
            order.payload()

    def test_serializes_the_conditional_deposit_source(self) -> None:
        keypair = Keypair()
        request = (
            envelope(keypair.pubkey())
            .salt(0)
            .deposit_source(DepositSource.MARKET)
            .time_in_force(TimeInForce.FOK)
            .sign(keypair, test_orderbook(), unit_rules())
        )
        body = request.to_dict()
        assert body["deposit_source"] == "conditional"
        assert body["tif"] == "FOK"
        assert "nonce" not in body

    def test_submit_signs_with_a_wallet_adapter(self) -> None:
        keypair = Keypair()
        signer = _Signer(keypair)
        client = _Client(SigningStrategy.wallet_adapter(signer))
        order = envelope(keypair.pubkey())
        assert asyncio.run(order.submit(client, test_orderbook())) == "submitted"
        [request] = client.submitted
        payload = order.payload()
        assert signer.messages == [get_order_message(payload)]
        VerifyKey(bytes(keypair.pubkey())).verify(
            get_order_message(payload), bytes.fromhex(request.signature)
        )
        assert request.salt == payload.salt

    def test_submit_signs_natively(self) -> None:
        keypair = Keypair()
        client = _Client(SigningStrategy.native(keypair))
        assert asyncio.run(envelope(keypair.pubkey()).submit(client, test_orderbook()))
        [request] = client.submitted
        assert request.amount_in == 1_000_000

    def test_submit_rejects_privy_order_signing(self) -> None:
        client = _Client(SigningStrategy.privy("wallet-id"))
        with pytest.raises(
            ValidationError, match="Privy order signing is not supported"
        ):
            asyncio.run(envelope(Keypair().pubkey()).submit(client, test_orderbook()))
        assert client.submitted == []


class TestOrderBuilder:
    def builder(self, maker: Pubkey) -> OrderBuilder:
        return (
            OrderBuilder()
            .maker(maker)
            .market(Pubkey.from_bytes(bytes([4]) * 32))
            .base_mint(Pubkey.from_bytes(bytes([5]) * 32))
            .quote_mint(Pubkey.from_bytes(bytes([6]) * 32))
        )

    def test_prices_with_the_side_set_at_build_time(self) -> None:
        maker = Keypair().pubkey()
        price_first = (
            self.builder(maker).salt(1).price("0.5", "10", unit_rules()).ask().build()
        )
        side_first = (
            self.builder(maker).salt(1).ask().price("0.5", "10", unit_rules()).build()
        )
        assert (price_first.amount_in, price_first.amount_out) == (10, 5)
        assert hash_order_hex(price_first) == hash_order_hex(side_first)

    def test_fixes_its_salt_on_the_first_build(self) -> None:
        keypair = Keypair()
        order = self.builder(keypair.pubkey()).bid().price("0.5", "10", unit_rules())
        unsigned = order.build()
        signed = order.build_and_sign(keypair)
        assert signed.salt == unsigned.salt
        assert hash_order_hex(signed) == hash_order_hex(unsigned)
        assert verify_order_signature(signed)

    def test_lets_raw_amounts_replace_price_and_size(self) -> None:
        order = (
            self.builder(Keypair().pubkey())
            .price("0.5", "10", unit_rules())
            .amount_in(6)
            .amount_out(10)
            .build()
        )
        assert (order.amount_in, order.amount_out) == (6, 10)

    def test_reports_missing_fields_and_sends_the_deposit_source(self) -> None:
        with pytest.raises(MissingFieldError, match="maker"):
            OrderBuilder().build(unit_rules())
        keypair = Keypair()
        request = (
            self.builder(keypair.pubkey())
            .bid()
            .amount_in(5)
            .amount_out(10)
            .deposit_source(DepositSource.MARKET)
            .to_submit_request(keypair, "unit", unit_rules())
        )
        assert request.to_dict()["deposit_source"] == "conditional"


class TestCancelSigning:
    def test_signs_the_utf8_messages_the_backend_verifies(self) -> None:
        keypair = Keypair()
        wallet = str(keypair.pubkey())
        verify = VerifyKey(bytes(keypair.pubkey()))
        for salt in ("0b9f6c1e-1d3a-4c55-9a7e-2f4d8e6b1c3a", "salt-é", "日本"):
            signature = sign_cancel_all(wallet, "ob", 1_700_000_000, salt, keypair)
            message = cancel_all_message(wallet, "ob", 1_700_000_000, salt)
            verify.verify(message.encode("utf-8"), bytes.fromhex(signature))
        order_hash = "ab" * 32
        signature = sign_cancel_order(order_hash, keypair)
        assert cancel_order_message(order_hash) == order_hash.encode("utf-8")
        verify.verify(order_hash.encode("utf-8"), bytes.fromhex(signature))


def test_apply_signature_mutates_only_on_success() -> None:
    order = unsigned_order()
    with pytest.raises(InvalidSignatureError):
        apply_signature(order, "not-base58-0OIl", unit_rules())
    assert order.signature == bytes(64)
    apply_signature(order, base58.b58encode(bytes([2]) * 64).decode(), unit_rules())
    assert order.signature == bytes([2]) * 64
