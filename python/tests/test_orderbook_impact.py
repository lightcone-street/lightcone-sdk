"""Tests for orderbook impact classification."""

from lightcone_sdk.domain.orderbook import ImpactDirection, OrderBookPair


def test_positive_impact():
    impact = OrderBookPair.impact("100", "125")

    assert impact.direction == ImpactDirection.POSITIVE
    assert impact.sign == "+"
    assert impact.pct == 25.0
    assert impact.dollar == "25"


def test_zero_impact():
    impact = OrderBookPair.impact("100", "100")

    assert impact.direction == ImpactDirection.ZERO
    assert impact.sign == ""
    assert impact.pct == 0.0
    assert impact.dollar == "0"


def test_negative_impact():
    impact = OrderBookPair.impact("100", "75")

    assert impact.direction == ImpactDirection.NEGATIVE
    assert impact.sign == "-"
    assert impact.pct == 25.0
    assert impact.dollar == "25"


def test_zero_deposit_price_returns_zero_impact():
    impact = OrderBookPair.impact("0", "75")

    assert impact.direction == ImpactDirection.ZERO
    assert impact.sign == ""
    assert impact.pct == 0.0
    assert impact.dollar == "0"
