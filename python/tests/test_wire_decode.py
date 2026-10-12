"""Strict committed wire decoders, DecimalText, and exact price helpers."""

import json
from datetime import UTC, datetime
from decimal import Decimal, localcontext
from enum import Enum

import pytest

from lightcone_sdk.error import DeserializationError
from lightcone_sdk.shared import CommitInfo, DepositSource, FundingSource, Side
from lightcone_sdk.shared.decimal_text import DecimalText, wire_decimal_text
from lightcone_sdk.shared.price import add_decimals, quote_per_base
from lightcone_sdk.shared.wire_decode import (
    I16,
    MISSING,
    take,
    wire_array,
    wire_bool,
    wire_commit_info,
    wire_decimal,
    wire_funding_source,
    wire_i64,
    wire_i128,
    wire_int,
    wire_known_or_none,
    wire_nonempty_str,
    wire_opt_i128,
    wire_opt_u64,
    wire_rfc3339,
    wire_side,
    wire_side_text,
    wire_str_or,
    wire_timestamp_ms,
    wire_u64,
    wire_u64_number,
)

U64_MAX = (1 << 64) - 1
I128_MIN = -(1 << 127)


class TestCommittedWireEncodings:
    def test_accepts_integers_as_text_or_numbers(self) -> None:
        data = json.loads(
            '{"revision":"18446744073709551615","slot":7,'
            '"fee":"-170141183460469231731687303715884105728",'
            '"remaining":null,"side":1,"reason":""}'
        )
        assert wire_u64(data["revision"], "revision") == U64_MAX
        assert wire_opt_u64(data["slot"], "slot") == 7
        assert wire_i128(data["fee"], "fee") == I128_MIN
        assert wire_opt_i128(data["remaining"], "remaining") is None
        assert wire_side(data["side"], "side") is Side.ASK
        assert wire_nonempty_str(data["reason"], "reason") is None

    def test_keeps_large_integer_tokens_exact(self) -> None:
        data = json.loads('{"revision":18446744073709551615}')
        assert wire_u64(data["revision"], "revision") == U64_MAX

    def test_decodes_text_sides_and_kept_reasons(self) -> None:
        assert wire_u64(42, "revision") == 42
        assert wire_side("bid", "side") is Side.BID
        assert wire_nonempty_str("expired", "reason") == "expired"

    @pytest.mark.parametrize(
        "value",
        [
            "-1",
            "+1",
            "1.0",
            "18446744073709551616",
            -1,
            "",
            " 1",
            "12\n",
            "١٢",
            "1_000",
            1.5,
            5.0,
            True,
            None,
        ],
    )
    def test_rejects_malformed_or_out_of_range_u64(self, value: object) -> None:
        with pytest.raises(DeserializationError):
            wire_u64(value, "revision")

    def test_reports_oversized_integer_text_as_out_of_range(self) -> None:
        # Longer than Python's 4,300-digit int(str) cap: still a named decode error.
        for decode in (wire_u64, wire_i128):
            with pytest.raises(DeserializationError, match="revision: integer out of"):
                decode("1" * 5000, "revision")
        assert wire_u64("0" * 60 + "7", "revision") == 7

    def test_rejects_malformed_i128_and_sides(self) -> None:
        for value in ("", str(1 << 127), True):
            with pytest.raises(DeserializationError):
                wire_i128(value, "fee")
        for value in (2, "1", True, False, "BID", None):
            with pytest.raises(DeserializationError):
                wire_side(value, "side")

    def test_decodes_plain_integers_and_rest_sides_as_rust_does(self) -> None:
        assert wire_i64(-5, "id") == -5
        assert wire_int(30, "fee_bps", *I16) == 30
        assert wire_u64_number(7, "revision") == 7
        for value in ("5", "-5", 1.5, 5.0, True):
            with pytest.raises(DeserializationError):
                wire_i64(value, "id")
            with pytest.raises(DeserializationError):
                wire_int(value, "fee_bps", *I16)
        with pytest.raises(DeserializationError):
            wire_u64_number("7", "revision")
        with pytest.raises(DeserializationError):
            wire_int(40000, "fee_bps", *I16)
        assert wire_side_text("sell", "side") is Side.ASK
        for value in (0, 1, True):
            with pytest.raises(DeserializationError):
                wire_side_text(value, "side")

    def test_defaults_absent_fields_but_rejects_null_as_serde_default_does(
        self,
    ) -> None:
        assert wire_bool(MISSING, "has_more", default=False) is False
        assert (
            wire_array(MISSING, "fills", lambda item, _: item, default_empty=True) == []
        )
        assert wire_str_or(MISSING, "order_type", "") == ""
        with pytest.raises(DeserializationError):
            wire_bool(None, "has_more", default=False)
        with pytest.raises(DeserializationError):
            wire_array(None, "fills", lambda item, _: item, default_empty=True)
        with pytest.raises(DeserializationError):
            wire_str_or(None, "order_type", "")
        with pytest.raises(DeserializationError):
            wire_bool(1, "ready")

    def test_take_distinguishes_absent_from_null(self) -> None:
        data = {"present": None}
        assert take(data, "present") is None
        assert take(data, "absent") is MISSING

    def test_names_the_field_in_decode_failures(self) -> None:
        with pytest.raises(DeserializationError, match=r"order\.committed_revision"):
            wire_u64("x", "order.committed_revision")

    def test_keeps_plain_decimal_text_exact_and_rejects_other_forms(self) -> None:
        with localcontext() as context:
            context.prec = 3
            assert wire_decimal("12.500000", "amount") == Decimal("12.500000")
            assert str(wire_decimal("12.500000", "amount")) == "12.500000"
            assert wire_decimal("-0.000001", "amount") == Decimal("-0.000001")
        for value in (
            12,
            "1e3",
            "+1",
            ".5",
            "1.",
            " 1",
            "1 ",
            "NaN",
            "0x1",
            "١٢",
            None,
        ):
            with pytest.raises(DeserializationError):
                wire_decimal(value, "amount")

    def test_maps_unknown_enum_values_without_failing(self) -> None:
        assert wire_funding_source("global", "source") is FundingSource.GLOBAL
        assert wire_funding_source("vault", "source") is FundingSource.UNKNOWN
        assert wire_known_or_none("unsupported", "tif", _Tif) is None
        assert wire_known_or_none("IOC", "tif", _Tif) is _Tif.IOC
        assert wire_known_or_none(None, "tif", _Tif) is None

    def test_decodes_flattened_commit_metadata(self) -> None:
        commit = wire_commit_info(
            {"effect_id": "9:0", "committed_revision": 9, "projection_generation": "2"},
            "fact",
        )
        assert commit == CommitInfo(
            effect_id="9:0",
            committed_revision=9,
            projection_generation=2,
            actionable=False,
        )
        with pytest.raises(DeserializationError, match=r"fact\.actionable"):
            wire_commit_info(
                {
                    "effect_id": "9:0",
                    "committed_revision": 9,
                    "projection_generation": 2,
                    "actionable": None,
                },
                "fact",
            )

    def test_decodes_rfc3339_with_fractional_seconds_and_offsets(self) -> None:
        expected = datetime(2026, 9, 29, 12, 0, 0, 987654, tzinfo=UTC)
        assert wire_rfc3339("2026-09-29T12:00:00.987654321+00:00", "at") == expected
        assert wire_rfc3339("2026-09-29T12:00:00.987654Z", "at") == expected
        assert wire_rfc3339("2026-09-29T14:00:00.987654+02:00", "at") == expected
        assert wire_rfc3339("2026-09-29 12:00:00.987654z", "at") == expected
        for value in (
            "2026-09-29T12:00:00",
            "2026-09-29",
            "20260929T120000Z",
            "2026-13-29T12:00:00Z",
            "١٢٠٢-09-29T12:00:00Z",
            1790000000,
        ):
            with pytest.raises(DeserializationError):
                wire_rfc3339(value, "at")

    def test_decodes_millisecond_timestamps_as_plain_numbers(self) -> None:
        assert wire_timestamp_ms(1790685600123, "created_at") == datetime(
            2026, 9, 29, 12, 40, 0, 123000, tzinfo=UTC
        )
        for value in ("1790685600123", -1, True, 1.5, 1 << 63):
            with pytest.raises(DeserializationError):
                wire_timestamp_ms(value, "created_at")


class _Tif(str, Enum):
    GTC = "GTC"
    IOC = "IOC"
    FOK = "FOK"


class TestDecimalText:
    def test_keeps_u128_aggregates_exact_beyond_96_bit_range(self) -> None:
        text = wire_decimal_text(
            "340282366920938463463374607431768.211455", "order_reserved"
        )
        assert text == "340282366920938463463374607431768.211455"
        # Beyond the Rust SDK decimal's range, as Rust's `to_decimal()` reports.
        assert text.to_decimal() is None
        assert DecimalText("79228162514264337593543950335").to_decimal() == Decimal(
            "79228162514264337593543950335"
        )
        assert DecimalText("0." + "1" * 29).to_decimal() is None

    def test_reports_oversized_text_as_beyond_range(self) -> None:
        assert DecimalText("1" * 5000).to_decimal() is None
        assert DecimalText("0" * 40 + "7.5").to_decimal() == Decimal("7.5")

    def test_converts_values_and_reports_sign(self) -> None:
        text = DecimalText("-0.000001")
        assert text.is_negative()
        assert text.to_decimal() == Decimal("-0.000001")
        assert DecimalText("40").to_decimal() == 40

    @pytest.mark.parametrize(
        "text",
        ["", "-", "1.", ".5", "+1", "1e3", " 1", "-0", "-0.000", "0x1", "1.2.3", "١"],
    )
    def test_rejects_non_canonical_text(self, text: str) -> None:
        with pytest.raises(ValueError):
            DecimalText(text)

    def test_rejects_json_numbers(self) -> None:
        with pytest.raises(DeserializationError):
            wire_decimal_text(12, "order_reserved")


class TestQuotePerBase:
    def test_derives_a_normalized_exact_price(self) -> None:
        assert format(quote_per_base(Decimal("2.2"), Decimal("4")), "f") == "0.55"
        assert format(quote_per_base(Decimal("100"), Decimal("1")), "f") == "100"
        assert (
            format(quote_per_base(Decimal("1"), Decimal("3")), "f")
            == "0.3333333333333333333333333333"
        )

    # Known answers from rust_decimal 1.40 `checked_div(..).normalize()`.
    @pytest.mark.parametrize(
        ("quote", "base", "expected"),
        [
            ("10", "3", "3.3333333333333333333333333333"),
            ("1", "3000000", "0.0000003333333333333333333333"),
            ("123456.789012", "7", "17636.684144571428571428571429"),
            ("0.150000", "3.00000001", "0.049999999833333333888888887"),
            ("12", "0.7", "17.142857142857142857142857143"),
            ("1", "81", "0.012345679012345679012345679"),
            ("-1", "3", "-0.3333333333333333333333333333"),
        ],
    )
    def test_matches_the_rust_sdk_96_bit_decimal_division(
        self, quote: str, base: str, expected: str
    ) -> None:
        with localcontext() as context:
            context.prec = 6
            result = quote_per_base(Decimal(quote), Decimal(base))
        assert result is not None
        assert format(result, "f") == expected

    def test_has_no_price_for_inputs_beyond_the_rust_decimal_range(self) -> None:
        assert quote_per_base(Decimal("1" * 5000), Decimal("1")) is None
        assert (
            quote_per_base(Decimal("1"), Decimal("79228162514264337593543950336"))
            is None
        )
        assert quote_per_base(Decimal("1"), Decimal("0." + "0" * 28 + "1")) is None

    def test_has_no_price_for_a_zero_base_or_out_of_range_quotient(self) -> None:
        assert quote_per_base(Decimal("1"), Decimal("0")) is None
        assert (
            format(
                quote_per_base(
                    Decimal("0.5"), Decimal("0.0000000000000000000000000001")
                ),
                "f",
            )
            == "5000000000000000000000000000"
        )
        assert (
            quote_per_base(Decimal("79228162514264337593543950335"), Decimal("0.1"))
            is None
        )


class TestAddDecimals:
    def test_adds_exactly_at_the_larger_scale(self) -> None:
        with localcontext() as context:
            context.prec = 6
            assert (
                str(add_decimals(Decimal("0"), Decimal("5.00000000"))) == "5.00000000"
            )
            assert (
                str(add_decimals(Decimal("0"), Decimal("12344.678912345")))
                == "12344.678912345"
            )
            assert str(add_decimals(Decimal("-1.5"), Decimal("0.25"))) == "-1.25"


class TestDepositSource:
    def test_sends_conditional_and_accepts_the_legacy_market_name(self) -> None:
        assert DepositSource.MARKET.as_str() == "conditional"
        assert DepositSource.GLOBAL.as_str() == "global"
        assert DepositSource.from_wire("conditional") is DepositSource.MARKET
        assert DepositSource.from_wire("market") is DepositSource.MARKET


class TestReviewRegressions:
    def test_integer_text_with_many_leading_zeros_decodes(self) -> None:
        assert wire_u64("0" * 5000 + "1", "revision") == 1
        assert wire_i128("-" + "0" * 5000 + "5", "fee") == -5
        assert wire_u64("0", "revision") == 0

    def test_json_numbers_wider_than_u64_or_i64_must_be_text(self) -> None:
        assert wire_i128(-(1 << 63), "fee") == -(1 << 63)
        assert wire_i128(U64_MAX, "fee") == U64_MAX
        assert wire_i128(str(I128_MIN), "fee") == I128_MIN
        for value in (U64_MAX + 1, -(1 << 63) - 1):
            with pytest.raises(DeserializationError, match="JSON number out of range"):
                wire_i128(value, "fee")

    @pytest.mark.parametrize(
        "text", ["9999-12-31T23:59:59-01:00", "0001-01-01T00:00:00+01:00"]
    )
    def test_times_outside_pythons_range_raise_decode_errors(self, text) -> None:
        with pytest.raises(DeserializationError, match="executed_at"):
            wire_rfc3339(text, "executed_at")

    def test_strict_json_rejects_duplicates_and_deep_nesting(self) -> None:
        from lightcone_sdk.shared.wire_decode import parse_json

        assert parse_json('{"a":{"b":1}}', "body") == {"a": {"b": 1}}
        with pytest.raises(DeserializationError, match="duplicate field 'a'"):
            parse_json('{"a":1,"a":2}', "body")
        with pytest.raises(DeserializationError, match="nesting too deep"):
            parse_json("[" * 100_000 + "]" * 100_000, "body")
