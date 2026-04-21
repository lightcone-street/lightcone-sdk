"""Tests for deposit-asset-pair derivation during market conversion."""

from lightcone_sdk.domain.market import ConditionalToken, DepositAsset
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
        icon_url="",
    )


def orderbook_pair(base_mint: str, quote_mint: str, outcome_index: int) -> OrderBookPair:
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
