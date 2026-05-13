from lightcone_sdk.domain.admin import (
    AddMetadataCategoryRequest,
    AddMetadataCategoryResponse,
    DepositTokenMetadataPayload,
    MarketMetadataPayload,
    MarketDeploymentConditionalToken,
    MarketDeploymentMarket,
    MarketDeploymentOutcome,
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


def test_market_metadata_omits_resolution_fields_when_none():
    request = MarketMetadataPayload(market_id=1, market_name="Updated name")

    payload = request.to_dict()
    assert payload == {
        "market_id": 1,
        "market_name": "Updated name",
    }
    assert "resolution" not in payload
    assert "resolution_by" not in payload


def test_market_metadata_serializes_resolution_by_without_resolution():
    request = MarketMetadataPayload(
        market_id=1,
        resolution_by=1_735_689_600_000,
    )

    assert request.to_dict() == {
        "market_id": 1,
        "resolution_by": 1_735_689_600_000,
    }


def test_market_metadata_serializes_explicit_resolution_states():
    enabled = MarketMetadataPayload(
        market_id=1,
        resolution=True,
        resolution_by=1_735_689_600_000,
    )
    cleared = MarketMetadataPayload(market_id=1, resolution=False)

    assert enabled.to_dict() == {
        "market_id": 1,
        "resolution": True,
        "resolution_by": 1_735_689_600_000,
    }
    assert cleared.to_dict() == {
        "market_id": 1,
        "resolution": False,
    }


def test_unified_metadata_response_reads_market_resolution_fields():
    response = UnifiedMetadataResponse.from_dict({
        "markets": [{
            "market_id": 1,
            "resolution": True,
            "resolution_by": 1_735_689_600_000,
        }, {
            "market_id": 2,
            "resolution": False,
        }],
    })

    assert response.markets[0]["resolution"] is True
    assert response.markets[0]["resolution_by"] == 1_735_689_600_000
    assert response.markets[1]["resolution"] is False
    assert "resolution_by" not in response.markets[1]


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
