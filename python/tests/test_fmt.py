"""Tests for shared display formatting helpers."""

from decimal import Decimal

from lightcone_sdk.shared.fmt.decimal import display as display_decimal
from lightcone_sdk.shared.fmt.num import (
    display,
    display_formatted_string,
    display_with_decimals,
)


def test_display_formatted_string_preserves_trailing_zeros():
    assert display_formatted_string("2.00") == "2.00"
    assert display_formatted_string("1234.500") == "1,234.500"


def test_display_preserves_selected_decimal_places():
    assert display(2.0) == "2.00"
    assert display(0.0) == "0.00"
    assert display(100.0) == "100.00"
    assert display(1234.567) == "1,234.57"
    assert display(0.1) == "0.10"
    assert display(0.004) == "0.004"
    assert display(0.0000005) == "0.0000005"
    assert display(0.000000001) == "0.0(8)1"
    assert display_with_decimals(2.0, 3) == "2.000"


def test_decimal_display_preserves_selected_decimal_places():
    assert display_decimal(Decimal("0")) == "0.00"
    assert display_decimal(Decimal("2.00")) == "2.00"
    assert display_decimal(Decimal("2.5")) == "2.50"
    assert display_decimal(Decimal("100")) == "100.00"
    assert display_decimal(Decimal("1234.567")) == "1,234.57"
    assert display_decimal(Decimal("0.1")) == "0.10"
    assert display_decimal(Decimal("0.004")) == "0.004"
    assert display_decimal(Decimal("0.0000005")) == "0.0000005"
    assert display_decimal(Decimal("0.000000001")) == "0.0(8)1"
