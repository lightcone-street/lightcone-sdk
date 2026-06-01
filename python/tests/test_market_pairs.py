"""Tests for deposit-asset-pair derivation during market conversion."""

import pytest

from lightcone_sdk.domain.market import (
    ConditionalToken,
    DepositAsset,
    DepositAssetPair,
    GlobalDepositAsset,
    sort_by_display_priority,
    token_display_priority,
)
from lightcone_sdk.domain.market.convert import _derive_deposit_asset_pairs
from lightcone_sdk.domain.orderbook import OrderBookPair


def deposit_asset(mint: str) -> DepositAsset:
    return DepositAsset(
        id=1,
        market_pda="market",
        deposit_asset=mint,
        num_outcomes=2,
        name=mint,
        symbol=mint,
        description=None,
        decimals=6,
        icon_url_low="",
        icon_url_medium="",
        icon_url_high="",
    )


def orderbook_pair(
    base_mint: str, quote_mint: str, outcome_index: int
) -> OrderBookPair:
    base = ConditionalToken(
        pubkey=f"cond-base-{outcome_index}",
        outcome_index=outcome_index,
        deposit_asset=base_mint,
    )
    quote = ConditionalToken(
        pubkey=f"cond-quote-{outcome_index}",
        outcome_index=outcome_index,
        deposit_asset=quote_mint,
    )
    return OrderBookPair(
        id=outcome_index,
        market_pubkey="market",
        orderbook_id=f"ob-{outcome_index}",
        base=base,
        quote=quote,
        outcome_index=outcome_index,
    )


class TestDeriveDepositAssetPairs:
    def test_deduplicates_across_outcomes(self):
        base = deposit_asset("USDC")
        quote = deposit_asset("USDT")
        pairs = _derive_deposit_asset_pairs(
            [base, quote],
            [
                orderbook_pair("USDC", "USDT", 0),
                orderbook_pair("USDC", "USDT", 1),
            ],
        )

        assert len(pairs) == 1
        assert pairs[0].id == "USDC-USDT"
        assert pairs[0].base == base
        assert pairs[0].quote == quote

    def test_skips_pairs_without_matching_deposit_asset(self):
        pairs = _derive_deposit_asset_pairs(
            [deposit_asset("USDC")],
            [orderbook_pair("USDC", "MISSING", 0)],
        )
        assert pairs == []

    def test_returns_all_distinct_pairs(self):
        pairs = _derive_deposit_asset_pairs(
            [deposit_asset("USDC"), deposit_asset("USDT"), deposit_asset("DAI")],
            [
                orderbook_pair("USDC", "USDT", 0),
                orderbook_pair("USDC", "DAI", 0),
            ],
        )
        pairs.sort(key=lambda pair: pair.id)

        assert len(pairs) == 2
        assert pairs[0].id == "USDC-DAI"
        assert pairs[1].id == "USDC-USDT"


class TestSortByDisplayPriority:
    @pytest.mark.parametrize(
        "symbol, expected_priority",
        [
            ("BTC", 0),
            ("WBTC", 0),
            ("ETH", 1),
            ("WETH", 1),
            ("SOL", 2),
            ("USDC", 255),
            ("ZZZ", 255),
        ],
    )
    def test_token_display_priority(self, symbol, expected_priority):
        asset = GlobalDepositAsset(symbol=symbol)
        assert token_display_priority(asset) == expected_priority

    def test_orders_priority_then_alpha(self):
        assets = [
            GlobalDepositAsset(symbol=symbol)
            for symbol in ["USDC", "SOL", "WETH", "AAA", "WBTC", "ETH", "BTC", "ZZZ"]
        ]
        sorted_symbols = [asset.symbol for asset in sort_by_display_priority(assets)]
        assert sorted_symbols == [
            "BTC",
            "WBTC",
            "ETH",
            "WETH",
            "SOL",
            "AAA",
            "USDC",
            "ZZZ",
        ]

    def test_sort_pairs_via_symbol_property(self):
        # DepositAssetPair exposes `symbol` via a property delegating to base —
        # so the same sort helper works on pairs.
        pairs = [
            DepositAssetPair(
                id=f"{symbol}-DAI",
                base=deposit_asset(symbol),
                quote=deposit_asset("DAI"),
            )
            for symbol in ["USDC", "SOL", "BTC", "ETH"]
        ]
        sorted_symbols = [pair.base.symbol for pair in sort_by_display_priority(pairs)]
        assert sorted_symbols == ["BTC", "ETH", "SOL", "USDC"]
