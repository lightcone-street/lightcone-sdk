"""Conditional balance delta tests."""

from decimal import getcontext, localcontext

from lightcone_sdk.domain.position import ConditionalBalanceDelta, ConditionalTokenType


def delta(**overrides) -> ConditionalBalanceDelta:
    base = {
        "market_pubkey": "market-1",
        "orderbook_id": "trump-usdc",
        "outcome_index": 0,
        "conditional_token": "trump-usdc-mint",
        "idle": "40.000000",
        "on_book": "85.000000",
    }
    base.update(overrides)
    return ConditionalBalanceDelta(**base)


def test_delta_total_is_full_precision():
    assert delta(idle="40.000001", on_book="85").total() == "125.000001"
    assert delta().total() == "125.000000"


def test_delta_total_ignores_the_decimal_context_and_never_uses_exponents():
    with localcontext() as context:
        context.prec = 6
        assert getcontext().prec == 6
        total = delta(idle="12344.678912345", on_book="0.000000001").total()
    assert total == "12344.678912346"
    assert delta(idle="0.0000001", on_book="0").total() == "0.0000001"
    assert (
        delta(idle="79228162514264337593543950335", on_book="1").total()
        == "79228162514264337593543950336"
    )


def test_delta_is_zero():
    assert delta(idle="0", on_book="0").is_zero() is True
    assert delta(idle="0", on_book="0.000001").is_zero() is False
    assert delta().is_zero() is False


def test_delta_into_token_balance():
    token_balance = delta().into_token_balance()
    assert token_balance.mint == "trump-usdc-mint"
    assert token_balance.idle == "40.000000"
    assert token_balance.on_book == "85.000000"
    assert isinstance(token_balance.token_type, ConditionalTokenType)
    assert token_balance.token_type.orderbook_id == "trump-usdc"
    assert token_balance.token_type.market_pubkey == "market-1"
    assert token_balance.token_type.outcome_index == 0


def test_delta_into_token_balance_defaults_missing_orderbook_id():
    token_balance = delta(orderbook_id=None).into_token_balance()
    assert isinstance(token_balance.token_type, ConditionalTokenType)
    assert token_balance.token_type.orderbook_id == ""
