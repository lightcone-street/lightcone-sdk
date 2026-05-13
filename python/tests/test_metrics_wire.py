from lightcone_sdk.domain.metrics import (
    DepositTokenVolumeHistory,
    DepositTokenVolumeHistoryQuery,
    PlatformMetrics,
)


def test_platform_metrics_reads_open_interest_and_fee_fields():
    metrics = PlatformMetrics.from_dict({
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
    })

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
    history = DepositTokenVolumeHistory.from_dict({
        "timestamp": 1_760_000_000_000,
        "resolution": "1d",
        "from": 1_704_067_200_000,
        "to": 1_760_000_000_000,
        "volume_total_usd": "123456.78",
        "total_days": 365,
        "deposit_tokens": [{
            "rank": 1,
            "deposit_asset": "deposit-asset",
            "symbol": "BTC",
            "volume_total_usd": "90000.00",
        }],
        "points": [{
            "bucket_start": 1_704_067_200_000,
            "bucket_start_date": "2024-01-01",
            "total_volume_usd": "1000.00",
            "cumulative_volume_usd": "1000.00",
            "deposit_token_volumes": [{
                "deposit_asset": "deposit-asset",
                "symbol": "BTC",
                "volume_usd": "700.00",
            }, {
                "deposit_asset": "other-deposit-asset",
                "symbol": "ETH",
                "volume_usd": "300.00",
            }],
        }],
    })

    assert history.resolution == "1d"
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
