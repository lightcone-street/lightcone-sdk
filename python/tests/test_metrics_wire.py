from lightcone_sdk.domain.metrics import (
    DepositTokenVolumeHistory,
    DepositTokenVolumeHistoryQuery,
    OpenInterestHistory,
    OpenInterestHistoryQuery,
    PlatformMetrics,
    UniqueTradersHistory,
    UniqueTradersHistoryQuery,
)
from lightcone_sdk.shared import Resolution


def test_platform_metrics_reads_open_interest_and_fee_fields():
    metrics = PlatformMetrics.from_dict(
        {
            "volume_24h_usd": "1",
            "volume_7d_usd": "2",
            "volume_30d_usd": "3",
            "volume_total_usd": "4",
            "taker_bid_volume_24h_usd": "5",
            "taker_bid_volume_7d_usd": "6",
            "taker_bid_volume_30d_usd": "7",
            "taker_bid_volume_total_usd": "8",
            "taker_ask_volume_24h_usd": "9",
            "taker_ask_volume_7d_usd": "10",
            "taker_ask_volume_30d_usd": "11",
            "taker_ask_volume_total_usd": "12",
            "taker_bid_ask_imbalance_24h_pct": "13",
            "taker_bid_ask_imbalance_7d_pct": "14",
            "taker_bid_ask_imbalance_30d_pct": "15",
            "taker_bid_ask_imbalance_total_pct": "16",
            "open_interest_usd": "12345.67",
            "fees_24h_usd": "0",
            "fees_7d_usd": "0",
            "fees_30d_usd": "0",
            "unique_traders_24h": 17,
            "unique_traders_7d": 18,
            "unique_traders_30d": 19,
            "active_markets": 20,
            "active_orderbooks": 21,
            "deposit_token_volumes": [],
        }
    )

    assert metrics.open_interest_usd == "12345.67"
    assert metrics.fees_24h_usd == "0"
    assert metrics.fees_7d_usd == "0"
    assert metrics.fees_30d_usd == "0"


def test_platform_metrics_defaults_new_fields_to_zero():
    metrics = PlatformMetrics.from_dict({})

    assert metrics.open_interest_usd == "0"
    assert metrics.fees_24h_usd == "0"
    assert metrics.fees_7d_usd == "0"
    assert metrics.fees_30d_usd == "0"


def test_deposit_token_volume_history_query_serializes_bounds_and_limit():
    query = DepositTokenVolumeHistoryQuery(
        from_ms=1_704_067_200_000,
        to_ms=1_760_000_000_000,
        limit=365,
    )

    assert query.to_query() == {
        "from": "1704067200000",
        "to": "1760000000000",
        "limit": "365",
    }


def test_deposit_token_volume_history_reads_daily_points():
    history = DepositTokenVolumeHistory.from_dict(
        {
            "timestamp": 1_760_000_000_000,
            "resolution": "1d",
            "from": 1_704_067_200_000,
            "to": 1_760_000_000_000,
            "volume_total_usd": "123456.78",
            "total_days": 365,
            "deposit_tokens": [
                {
                    "rank": 1,
                    "deposit_asset": "deposit-asset",
                    "symbol": "BTC",
                    "volume_total_usd": "90000.00",
                }
            ],
            "points": [
                {
                    "bucket_start": 1_704_067_200_000,
                    "bucket_start_date": "2024-01-01",
                    "total_volume_usd": "1000.00",
                    "cumulative_volume_usd": "1000.00",
                    "deposit_token_volumes": [
                        {
                            "deposit_asset": "deposit-asset",
                            "symbol": "BTC",
                            "volume_usd": "700.00",
                        },
                        {
                            "deposit_asset": "other-deposit-asset",
                            "symbol": "ETH",
                            "volume_usd": "300.00",
                        },
                    ],
                }
            ],
        }
    )

    assert history.resolution == Resolution.ONE_DAY
    assert history.from_ms == 1_704_067_200_000
    assert history.to_ms == 1_760_000_000_000
    assert history.volume_total_usd == "123456.78"
    assert history.deposit_tokens[0].rank == 1
    assert history.deposit_tokens[0].symbol == "BTC"
    assert history.deposit_tokens[0].volume_total_usd == "90000.00"
    assert history.points[0].bucket_start_date == "2024-01-01"
    assert history.points[0].total_volume_usd == "1000.00"
    assert history.points[0].cumulative_volume_usd == "1000.00"
    assert history.points[0].deposit_token_volumes[0].volume_usd == "700.00"


def test_open_interest_history_query_serializes_bounds_and_limit():
    query = OpenInterestHistoryQuery(
        from_ms=1_704_067_200_000,
        to_ms=1_760_000_000_000,
        limit=30,
    )

    assert query.to_query() == {
        "from": "1704067200000",
        "to": "1760000000000",
        "limit": "30",
    }


def test_open_interest_history_reads_daily_snapshots_and_preserves_zero_values():
    history = OpenInterestHistory.from_dict(
        {
            "timestamp": 1_760_000_000_000,
            "resolution": "1d",
            "from": 1_704_067_200_000,
            "to": 1_760_000_000_000,
            "latest_open_interest_usd": "123456.78",
            "total_days": 30,
            "deposit_assets": [
                {
                    "rank": 1,
                    "deposit_asset": "deposit-asset",
                    "symbol": "BTC",
                    "latest_open_interest_usd": "90000.00",
                    "max_open_interest_usd": "100000.00",
                }
            ],
            "points": [
                {
                    "bucket_start": 1_704_067_200_000,
                    "bucket_start_date": "2024-01-01",
                    "total_open_interest_usd": "123456.78",
                    "deposit_asset_open_interest": [
                        {
                            "deposit_asset": "deposit-asset",
                            "symbol": "BTC",
                            "open_interest_usd": "90000.00",
                        },
                        {
                            "deposit_asset": "other-deposit-asset",
                            "symbol": "ETH",
                            "open_interest_usd": "0",
                        },
                    ],
                }
            ],
        }
    )

    assert history.resolution == Resolution.ONE_DAY
    assert history.from_ms == 1_704_067_200_000
    assert history.to_ms == 1_760_000_000_000
    assert history.latest_open_interest_usd == "123456.78"
    assert history.deposit_assets[0].rank == 1
    assert history.deposit_assets[0].latest_open_interest_usd == "90000.00"
    assert history.deposit_assets[0].max_open_interest_usd == "100000.00"
    assert history.points[0].bucket_start_date == "2024-01-01"
    assert history.points[0].total_open_interest_usd == "123456.78"
    assert (
        history.points[0].deposit_asset_open_interest[0].open_interest_usd == "90000.00"
    )
    assert history.points[0].deposit_asset_open_interest[1].open_interest_usd == "0"


def test_unique_traders_history_default_query_uses_backend_defaults():
    assert UniqueTradersHistoryQuery().to_query() == {}


def test_unique_traders_history_query_serializes_scope_bounds_and_limit():
    query = UniqueTradersHistoryQuery(
        scope="market",
        scope_key="market-pubkey",
        from_ms=1_710_000_000_000,
        to_ms=1_720_000_000_000,
        limit=30,
    )

    assert query.to_query() == {
        "scope": "market",
        "scope_key": "market-pubkey",
        "from": "1710000000000",
        "to": "1720000000000",
        "limit": "30",
    }


def test_unique_traders_history_reads_daily_counts_and_preserves_zero_days():
    history = UniqueTradersHistory.from_dict(
        {
            "timestamp": 1_760_000_000_000,
            "resolution": "1d",
            "scope": "platform",
            "scope_key": "platform",
            "from": 1_710_000_000_000,
            "to": 1_720_000_000_000,
            "latest_unique_traders": 42,
            "total_days": 30,
            "points": [
                {
                    "bucket_start": 1_710_000_000_000,
                    "bucket_start_date": "2024-03-09",
                    "unique_traders": 42,
                },
                {
                    "bucket_start": 1_710_086_400_000,
                    "bucket_start_date": "2024-03-10",
                    "unique_traders": 0,
                },
            ],
        }
    )

    assert history.resolution == Resolution.ONE_DAY
    assert history.scope == "platform"
    assert history.scope_key == "platform"
    assert history.from_ms == 1_710_000_000_000
    assert history.to_ms == 1_720_000_000_000
    assert history.latest_unique_traders == 42
    assert history.total_days == 30
    assert history.points[0].bucket_start_date == "2024-03-09"
    assert history.points[0].unique_traders == 42
    assert history.points[1].bucket_start_date == "2024-03-10"
    assert history.points[1].unique_traders == 0
