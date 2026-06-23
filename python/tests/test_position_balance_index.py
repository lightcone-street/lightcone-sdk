"""Conditional balance delta + user market balance index tests."""

from lightcone_sdk.domain.order import (
    UserDepositAssetBalance,
    UserMarketBalance,
    UserOutcomeBalance,
)
from lightcone_sdk.domain.position import (
    ConditionalBalanceDelta,
    ConditionalTokenType,
    UserMarketBalanceIndex,
)


def outcome(
    conditional_token: str,
    balance_idle: str,
    balance_on_book: str,
    outcome_index: int = 0,
) -> UserOutcomeBalance:
    return UserOutcomeBalance(
        outcome_index=outcome_index,
        conditional_token=conditional_token,
        balance="0",
        balance_idle=balance_idle,
        balance_on_book=balance_on_book,
    )


def market_balance(market_pubkey: str) -> UserMarketBalance:
    return UserMarketBalance(
        market_pubkey=market_pubkey,
        deposit_assets=[
            UserDepositAssetBalance(
                deposit_asset="usdc-mint",
                outcomes=[
                    outcome("trump-usdc-mint", "40.000000", "85.000000", 0),
                    outcome("kamala-usdc-mint", "0", "0", 1),  # zero -> dropped
                ],
            ),
            UserDepositAssetBalance(
                deposit_asset="empty-asset",
                outcomes=[outcome("zzz-mint", "0", "0", 0)],  # all-zero -> asset dropped
            ),
        ],
    )


def delta(**overrides) -> ConditionalBalanceDelta:
    base = dict(
        market_pubkey="market-1",
        orderbook_id="trump-usdc",
        outcome_index=0,
        conditional_token="trump-usdc-mint",
        idle="40.000000",
        on_book="85.000000",
    )
    base.update(overrides)
    return ConditionalBalanceDelta(**base)


def test_user_outcome_balance_is_zero():
    assert outcome("m", "0", "0").is_zero() is True
    assert outcome("m", "0.000001", "0").is_zero() is False
    assert outcome("m", "0", "1").is_zero() is False


def test_delta_total_is_full_precision():
    assert delta(idle="40.000001", on_book="85").total() == "125.000001"


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


def test_delta_into_user_outcome_balance():
    converted = delta(idle="40.000001", on_book="85").into_user_outcome_balance()
    assert converted.conditional_token == "trump-usdc-mint"
    assert converted.balance == "125.000001"
    assert converted.balance_idle == "40.000001"
    assert converted.balance_on_book == "85"


def test_index_builds_nested_map_skipping_zeros():
    index = UserMarketBalanceIndex.from_user_market_balances(
        [market_balance("market-1")]
    )
    market = index.get("market-1")
    assert market is not None
    assert list(market.keys()) == ["usdc-mint"]  # empty-asset dropped
    assert list(market["usdc-mint"].keys()) == ["trump-usdc-mint"]  # zero outcome dropped


def test_index_returns_none_for_empty_market():
    empty = UserMarketBalance(
        market_pubkey="market-empty",
        deposit_assets=[
            UserDepositAssetBalance(
                deposit_asset="usdc-mint",
                outcomes=[outcome("zzz", "0", "0")],
            )
        ],
    )
    assert UserMarketBalanceIndex.from_user_market_balance(empty) is None


def test_index_market_pubkeys_sorted():
    index = UserMarketBalanceIndex.from_user_market_balances(
        [
            market_balance("market-c"),
            market_balance("market-a"),
            market_balance("market-b"),
        ]
    )
    assert index.market_pubkeys() == ["market-a", "market-b", "market-c"]


def test_index_empty_input():
    index = UserMarketBalanceIndex.from_user_market_balances([])
    assert index.is_empty() is True
    assert index.market_pubkeys() == []
