from lightcone_sdk.domain.metrics import PlatformMetrics


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
