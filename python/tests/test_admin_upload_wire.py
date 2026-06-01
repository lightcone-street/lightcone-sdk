from lightcone_sdk.domain.admin import (
    MarketDeploymentConditionalToken,
    MarketDeploymentMarket,
    MarketDeploymentOutcome,
    UploadMarketDeploymentAssetsResponse,
    UploadedConditionalToken,
)


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
