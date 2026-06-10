"""Tests for Side/Denominator trading-math helpers."""

from decimal import Decimal

from lightcone_sdk.program.types import OrderSide
from lightcone_sdk.shared.types import Denominator, Side


def test_spend_and_receive_denominators():
    assert Side.BID.spend_denominator() == Denominator.QUOTE
    assert Side.BID.receive_denominator() == Denominator.BASE
    assert Side.ASK.spend_denominator() == Denominator.BASE
    assert Side.ASK.receive_denominator() == Denominator.QUOTE


def test_convert_to_same_denomination_is_identity():
    amount = Decimal("4.25")
    # Price is irrelevant for the identity conversion, even when unusable
    assert Denominator.BASE.convert_to(Denominator.BASE, amount, Decimal(0)) == amount
    assert Denominator.QUOTE.convert_to(Denominator.QUOTE, amount, Decimal(0)) == amount


def test_convert_to_crosses_at_price():
    base_price_in_quote = Decimal("0.25")
    assert Denominator.BASE.convert_to(
        Denominator.QUOTE, Decimal(8), base_price_in_quote
    ) == Decimal(2)
    assert Denominator.QUOTE.convert_to(
        Denominator.BASE, Decimal(2), base_price_in_quote
    ) == Decimal(8)


def test_convert_to_requires_positive_price_to_cross():
    amount = Decimal(10)
    assert Denominator.BASE.convert_to(Denominator.QUOTE, amount, Decimal(0)) is None
    assert Denominator.QUOTE.convert_to(Denominator.BASE, amount, Decimal(-1)) is None


def test_convert_to_round_trips():
    base_price_in_quote = Decimal("3.7")
    amount = Decimal(12)
    quote_amount = Denominator.BASE.convert_to(
        Denominator.QUOTE, amount, base_price_in_quote
    )
    assert quote_amount is not None
    assert (
        Denominator.QUOTE.convert_to(Denominator.BASE, quote_amount, base_price_in_quote)
        == amount
    )


def test_apply_impact_protection_directions():
    worst_fill_price = Decimal(100)
    protection_percent = Decimal(10)
    # buying: willing to pay more
    assert Side.BID.apply_impact_protection(
        worst_fill_price, protection_percent
    ) == Decimal(110)
    # selling: willing to receive less
    assert Side.ASK.apply_impact_protection(
        worst_fill_price, protection_percent
    ) == Decimal(90)


def test_apply_impact_protection_requires_positive_inputs():
    assert Side.BID.apply_impact_protection(Decimal(0), Decimal(10)) is None
    assert Side.ASK.apply_impact_protection(Decimal(100), Decimal(0)) is None


def test_order_side_from_side():
    assert OrderSide.from_side(Side.BID) == OrderSide.BID
    assert OrderSide.from_side(Side.ASK) == OrderSide.ASK
