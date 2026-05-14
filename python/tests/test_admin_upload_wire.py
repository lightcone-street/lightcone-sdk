from lightcone_sdk.domain.admin import (
    AddMetadataCategoryRequest,
    AddMetadataCategoryResponse,
    AdminMarketStatus,
    AdminMarketStatusFilter,
    AdminMarketsQuery,
    AdminMarketsResponse,
    CriticalLogErrors24hCountResponse,
    DepositTokenMetadataPayload,
    MarketMetadataPayload,
    MarketDeploymentConditionalToken,
    MarketDeploymentMarket,
    MarketDeploymentOutcome,
    MarketsToSettleCountResponse,
    MarketsToSettleQuery,
    MarketsToSettleResponse,
    UnifiedMetadataResponse,
    UploadMarketDeploymentAssetsResponse,
    UploadedConditionalToken,
)


def test_deposit_token_metadata_serializes_without_legacy_s3_fields():
    request = DepositTokenMetadataPayload(
        deposit_asset="TOKEN_MINT",
        min_order_size=1_000_000,
        binance_symbol="BTCUSDT",
        binance_enabled=True,
        okx_inst_id="BTC-USDT",
    )

    payload = request.to_dict()
    assert payload == {
        "deposit_asset": "TOKEN_MINT",
        "min_order_size": 1_000_000,
        "binance_symbol": "BTCUSDT",
        "binance_enabled": True,
        "okx_inst_id": "BTC-USDT",
    }
    assert "s3_synced" not in payload
    assert "s3_synced_at" not in payload
    assert "s3_error" not in payload


def test_unified_metadata_response_reads_deposit_token_fields():
    response = UnifiedMetadataResponse.from_dict({
        "deposit_tokens": [{
            "id": 1,
            "deposit_asset": "TOKEN_MINT",
            "display_name": "Bitcoin",
            "symbol": "BTC",
            "token_symbol": None,
            "binance_symbol": "BTCUSDT",
            "binance_enabled": True,
            "okx_inst_id": "BTC-USDT",
            "description": None,
            "icon_url_low": None,
            "icon_url_medium": None,
            "icon_url_high": None,
            "metadata_uri": None,
            "decimals": 8,
            "min_order_size": 100_000,
            "created_at": "2026-05-12T00:00:00Z",
            "updated_at": "2026-05-12T00:00:00Z",
        }],
    })

    token = response.deposit_tokens[0]
    assert token.deposit_asset == "TOKEN_MINT"
    assert token.binance_symbol == "BTCUSDT"
    assert token.binance_enabled is True
    assert token.okx_inst_id == "BTC-USDT"
    assert token.min_order_size == 100_000


def test_metadata_category_request_and_response_shapes():
    request = AddMetadataCategoryRequest(category="Crypto")
    response = AddMetadataCategoryResponse.from_dict({"category": "Crypto"})

    assert request.to_dict() == {"category": "Crypto"}
    assert response.category == "Crypto"


def test_market_metadata_omits_resolution_by_when_unset():
    request = MarketMetadataPayload(market_id=1, market_name="Updated name")

    payload = request.to_dict()
    assert payload == {
        "market_id": 1,
        "market_name": "Updated name",
    }
    assert "resolution" not in payload
    assert "resolution_by" not in payload


def test_market_metadata_serializes_resolution_by_timestamp():
    request = MarketMetadataPayload(
        market_id=1,
        resolution_by=1_735_689_600_000,
    )

    assert request.to_dict() == {
        "market_id": 1,
        "resolution_by": 1_735_689_600_000,
    }


def test_market_metadata_serializes_resolution_by_null_to_clear():
    request = MarketMetadataPayload(
        market_id=1,
        resolution_by=None,
    )

    assert request.to_dict() == {
        "market_id": 1,
        "resolution_by": None,
    }


def test_market_metadata_resolution_by_helpers_set_and_clear():
    set_request = MarketMetadataPayload(market_id=1).with_resolution_by(
        1_735_689_600_000
    )
    clear_request = MarketMetadataPayload(market_id=1).with_cleared_resolution_by()

    assert set_request.to_dict() == {
        "market_id": 1,
        "resolution_by": 1_735_689_600_000,
    }
    assert clear_request.to_dict() == {
        "market_id": 1,
        "resolution_by": None,
    }


def test_unified_metadata_response_reads_market_resolution_by_values():
    response = UnifiedMetadataResponse.from_dict({
        "markets": [{
            "market_id": 1,
            "resolution_by": 1_735_689_600_000,
        }, {
            "market_id": 2,
            "resolution_by": None,
        }],
    })

    assert response.markets[0]["resolution_by"] == 1_735_689_600_000
    assert response.markets[1]["resolution_by"] is None


def test_admin_markets_query_serializes_status_and_range_filters():
    query = AdminMarketsQuery(
        cursor=100,
        limit=50,
        sort_by="open_interest_usd",
        sort_direction="asc",
        market_status=AdminMarketStatusFilter.RESOLVED,
        category="Crypto",
        search="btc",
        min_volume_24h_usd="1000",
        max_open_interest_usd="50000",
        min_unique_traders_total=10,
    )

    assert query.to_query() == {
        "cursor": "100",
        "limit": "50",
        "sort_by": "open_interest_usd",
        "sort_direction": "asc",
        "market_status": "resolved",
        "category": "Crypto",
        "search": "btc",
        "min_volume_24h_usd": "1000",
        "min_unique_traders_total": "10",
        "max_open_interest_usd": "50000",
    }


def test_admin_markets_response_reads_market_rows():
    response = AdminMarketsResponse.from_dict({
        "timestamp": 1_710_000_000_000,
        "sort_by": "volume_24h_usd",
        "sort_direction": "desc",
        "total": 123,
        "limit": 100,
        "next_cursor": 100,
        "has_more": True,
        "markets": [{
            "rank": 1,
            "market_id": 123,
            "market_pubkey": "market-pubkey",
            "market_status": "Active",
            "slug": "btc-100k",
            "market_name": "Will BTC hit $100k?",
            "category": "Crypto",
            "icon_url": "https://example.com/icon.png",
            "num_outcomes": 2,
            "resolution_by": 1_760_000_000_000,
            "open_interest_usd": "12345.67",
            "volume_24h_usd": "1000.00",
            "volume_7d_usd": "7000.00",
            "volume_30d_usd": "30000.00",
            "volume_total_usd": "50000.00",
            "unique_traders_24h": 50,
            "unique_traders_7d": 200,
            "unique_traders_30d": 600,
            "unique_traders_total": 900,
            "fees_24h_usd": "0",
            "fees_7d_usd": "0",
            "fees_30d_usd": "0",
            "fees_total_usd": "0",
            "created_at": "2026-01-01T00:00:00+00:00",
            "activated_at": "2026-01-02T00:00:00+00:00",
            "settled_at": None,
            "updated_at": "2026-01-03T00:00:00+00:00",
        }],
    })

    assert response.next_cursor == 100
    assert response.has_more is True
    assert response.markets[0].market_status == AdminMarketStatus.ACTIVE
    assert response.markets[0].resolution_by == 1_760_000_000_000
    assert response.markets[0].open_interest_usd == "12345.67"
    assert response.markets[0].unique_traders_total == 900
    assert response.markets[0].fees_total_usd == "0"
    assert response.markets[0].settled_at is None


def test_markets_to_settle_admin_response_shapes():
    count = MarketsToSettleCountResponse.from_dict({
        "markets_to_settle_count": 3,
    })
    query = MarketsToSettleQuery(cursor=123, limit=200)
    response = MarketsToSettleResponse.from_dict({
        "markets": [{
            "market_id": 123,
            "market_pubkey": "market-pubkey",
            "market_status": "Active",
            "market_name": "Market",
            "slug": "market",
            "outcomes": [],
            "deposit_assets": [],
            "orderbooks": [],
        }],
        "next_cursor": 456,
        "has_more": True,
    })

    assert count.markets_to_settle_count == 3
    assert query.to_query() == {
        "cursor": "123",
        "limit": "200",
    }
    assert response.markets[0].market_id == 123
    assert response.next_cursor == 456
    assert response.has_more is True


def test_critical_log_errors_24h_count_response_shape():
    response = CriticalLogErrors24hCountResponse.from_dict({
        "critical_log_errors_24h": 1,
    })

    assert response.critical_log_errors_24h == 1


def test_upload_request_uses_quality_specific_image_fields():
    market = MarketDeploymentMarket(
        name="Market",
        slug="market",
        banner_image_data_url_high="data:image/webp;base64,banner-high",
        banner_image_content_type_high="image/webp",
        icon_image_data_url_low="data:image/webp;base64,icon-low",
        icon_image_content_type_low="image/webp",
        icon_image_data_url_high="data:image/webp;base64,icon-high",
        icon_image_content_type_high="image/webp",
    )
    outcome = MarketDeploymentOutcome(
        index=0,
        name="Yes",
        symbol="YES",
        icon_image_data_url_high="data:image/webp;base64,outcome-high",
        icon_image_content_type_high="image/webp",
    )
    token = MarketDeploymentConditionalToken(
        outcome_index=0,
        deposit_mint="deposit-mint",
        conditional_mint="conditional-mint",
        name="Yes USDC",
        symbol="YES-USDC",
        image_data_url_high="data:image/webp;base64,token-high",
        image_content_type_high="image/webp",
        image_data_url_low="data:image/webp;base64,token-low",
        image_content_type_low="image/webp",
    )

    market_payload = market.to_dict()
    outcome_payload = outcome.to_dict()
    token_payload = token.to_dict()

    assert (
        market_payload["banner_image_data_url_high"]
        == "data:image/webp;base64,banner-high"
    )
    assert (
        market_payload["icon_image_data_url_low"]
        == "data:image/webp;base64,icon-low"
    )
    assert "banner_image_data_url" not in market_payload
    assert "icon_image_data_url" not in market_payload
    assert outcome_payload["icon_image_content_type_high"] == "image/webp"
    assert "icon_image_content_type" not in outcome_payload
    assert token_payload["image_data_url_high"] == "data:image/webp;base64,token-high"
    assert "image_data_url" not in token_payload
    assert "image_content_type" not in token_payload


def test_upload_response_reads_variant_token_urls():
    response = UploadMarketDeploymentAssetsResponse.from_dict({
        "market_metadata_uri": "s3://metadata/market.json",
        "market": {
            "banner_image_url_high": "https://cdn/banner-high.webp",
        },
        "outcomes": [{
            "index": 0,
            "icon_url_high": "https://cdn/outcome-high.webp",
        }],
        "deposit_assets": [{
            "mint": "deposit-mint",
            "icon_url_high": "https://cdn/deposit-high.webp",
        }],
        "tokens": [{
            "conditional_mint": "conditional-mint",
            "metadata_uri": "s3://metadata/token.json",
            "image_url_low": "https://cdn/token-low.webp",
            "image_url_medium": "https://cdn/token-medium.webp",
            "image_url_high": "https://cdn/token-high.webp",
        }],
    })

    assert response.deposit_assets[0].mint == "deposit-mint"
    assert response.tokens[0] == UploadedConditionalToken(
        conditional_mint="conditional-mint",
        metadata_uri="s3://metadata/token.json",
        image_url_low="https://cdn/token-low.webp",
        image_url_medium="https://cdn/token-medium.webp",
        image_url_high="https://cdn/token-high.webp",
    )
