"""Tests for shared display formatting helpers."""

from decimal import Decimal

from lightcone_sdk.shared.fmt.decimal import display as display_decimal, display_pct as display_pct_decimal
from lightcone_sdk.shared.fmt.num import (
    display,
    display_formatted_string,
    display_pct,
    display_with_decimals,
)


def test_display_formatted_string_preserves_trailing_zeros():
    assert display_formatted_string("2.00") == "2.00"
    assert display_formatted_string("1234.500") == "1,234.500"


def test_display_uses_magnitude_based_decimal_places():
    assert display(12345.67) == "12,346"
    assert display(1234.56) == "1,234.6"
    assert display(123.456) == "123.46"
    assert display(15.4567) == "15.457"
    assert display(1.23456) == "1.2346"
    assert display(0.123456) == "0.1235"
    assert display(0.012345) == "0.01235"
    assert display_with_decimals(2.0, 3) == "2.000"


def test_display_tier_boundaries():
    assert display(10000.0) == "10,000"
    assert display(9999.99) == "10,000.0"
    assert display(1000.0) == "1,000.0"
    assert display(999.999) == "1,000.00"
    assert display(100.0) == "100.00"
    assert display(99.9999) == "100.000"
    assert display(10.0) == "10.000"
    assert display(9.87654) == "9.8765"
    assert display(1.0) == "1.0000"
    assert display(0.999999) == "1.0000"
    assert display(0.1) == "0.1000"
    assert display(0.099999) == "0.10000"


def test_display_caps_small_values_at_five_decimals():
    assert display(0.0) == "0"
    assert display(0.01) == "0.01000"
    assert display(0.00003) == "0.00003"
    assert display(0.000004) == "0"
    assert display(-0.000004) == "0"
    assert display(0.000000001) == "0"


def test_decimal_display_uses_magnitude_based_decimal_places():
    assert display_decimal(Decimal("12345.67")) == "12,346"
    assert display_decimal(Decimal("1234.56")) == "1,234.6"
    assert display_decimal(Decimal("123.456")) == "123.46"
    assert display_decimal(Decimal("15.4567")) == "15.457"
    assert display_decimal(Decimal("1.23456")) == "1.2346"
    assert display_decimal(Decimal("0.123456")) == "0.1235"
    assert display_decimal(Decimal("0.012345")) == "0.01235"


def test_decimal_display_tier_boundaries():
    assert display_decimal(Decimal("10000")) == "10,000"
    assert display_decimal(Decimal("9999.99")) == "10,000.0"
    assert display_decimal(Decimal("1000")) == "1,000.0"
    assert display_decimal(Decimal("999.999")) == "1,000.00"
    assert display_decimal(Decimal("100")) == "100.00"
    assert display_decimal(Decimal("99.9999")) == "100.000"
    assert display_decimal(Decimal("10")) == "10.000"
    assert display_decimal(Decimal("9.87654")) == "9.8765"
    assert display_decimal(Decimal("1")) == "1.0000"
    assert display_decimal(Decimal("0.999999")) == "1.0000"
    assert display_decimal(Decimal("0.1")) == "0.1000"
    assert display_decimal(Decimal("0.099999")) == "0.10000"


def test_decimal_display_caps_small_values_at_five_decimals():
    assert display_decimal(Decimal("0")) == "0"
    assert display_decimal(Decimal("0.01")) == "0.01000"
    assert display_decimal(Decimal("0.00003")) == "0.00003"
    assert display_decimal(Decimal("0.000004")) == "0"
    assert display_decimal(Decimal("-0.000004")) == "0"
    assert display_decimal(Decimal("0.000000001")) == "0"


def test_display_pct_truncation():
    assert display_pct(12.345) == "12.34"
    assert display_pct(12.999) == "12.99"
    assert display_pct(99.999) == "99.99"


def test_display_pct_padding_true():
    assert display_pct(12.3) == "12.30"
    assert display_pct(12.0) == "12.00"
    assert display_pct(0.0) == "0.00"


def test_display_pct_padding_false():
    assert display_pct(12.345, padding=False) == "12.34"
    assert display_pct(12.3, padding=False) == "12.3"
    assert display_pct(12.0, padding=False) == "12"
    assert display_pct(0.0, padding=False) == "0"


def test_display_pct_negative():
    assert display_pct(-3.456) == "-3.45"
    assert display_pct(-3.4) == "-3.40"
    assert display_pct(-3.456, padding=False) == "-3.45"


def test_display_pct_decimal_truncation():
    assert display_pct_decimal(Decimal("12.345")) == "12.34"
    assert display_pct_decimal(Decimal("12.999")) == "12.99"
    assert display_pct_decimal(Decimal("99.999")) == "99.99"


def test_display_pct_decimal_padding_true():
    assert display_pct_decimal(Decimal("12.3")) == "12.30"
    assert display_pct_decimal(Decimal("12")) == "12.00"
    assert display_pct_decimal(Decimal("0")) == "0.00"


def test_display_pct_decimal_padding_false():
    assert display_pct_decimal(Decimal("12.345"), padding=False) == "12.34"
    assert display_pct_decimal(Decimal("12.3"), padding=False) == "12.3"
    assert display_pct_decimal(Decimal("12"), padding=False) == "12"
    assert display_pct_decimal(Decimal("0"), padding=False) == "0"


def test_display_pct_decimal_negative():
    assert display_pct_decimal(Decimal("-3.456")) == "-3.45"
    assert display_pct_decimal(Decimal("-3.4")) == "-3.40"
    assert display_pct_decimal(Decimal("-3.456"), padding=False) == "-3.45"
