import { describe, it } from "node:test";
import assert from "node:assert/strict";
import type {
  UploadMarketDeploymentAssetsRequest,
  UploadMarketDeploymentAssetsResponse,
} from "../src/domain/admin";

describe("admin upload wire types", () => {
  it("uses quality-specific upload fields", () => {
    const request: UploadMarketDeploymentAssetsRequest = {
      market_id: 7,
      market_pubkey: "market-pubkey",
      market: {
        name: "Market",
        slug: "market",
        banner_image_data_url_high: "data:image/webp;base64,banner-high",
        banner_image_content_type_high: "image/webp",
        icon_image_data_url_low: "data:image/webp;base64,icon-low",
        icon_image_content_type_low: "image/webp",
        icon_image_data_url_high: "data:image/webp;base64,icon-high",
        icon_image_content_type_high: "image/webp",
      },
      outcomes: [{
        index: 0,
        name: "Yes",
        symbol: "YES",
        icon_image_data_url_high: "data:image/webp;base64,outcome-high",
        icon_image_content_type_high: "image/webp",
      }],
      conditional_tokens: [{
        outcome_index: 0,
        deposit_mint: "deposit-mint",
        conditional_mint: "conditional-mint",
        name: "Yes USDC",
        symbol: "YES-USDC",
        image_data_url_low: "data:image/webp;base64,token-low",
        image_content_type_low: "image/webp",
        image_data_url_high: "data:image/webp;base64,token-high",
        image_content_type_high: "image/webp",
      }],
    };

    const payload = JSON.parse(JSON.stringify(request)) as Record<string, any>;
    assert.equal(payload.market.banner_image_data_url_high, "data:image/webp;base64,banner-high");
    assert.equal(payload.market.icon_image_data_url_low, "data:image/webp;base64,icon-low");
    assert.equal("banner_image_data_url" in payload.market, false);
    assert.equal("icon_image_data_url" in payload.market, false);
    assert.equal(payload.outcomes[0].icon_image_content_type_high, "image/webp");
    assert.equal("icon_image_content_type" in payload.outcomes[0], false);
    assert.equal(payload.conditional_tokens[0].image_data_url_high, "data:image/webp;base64,token-high");
    assert.equal("image_data_url" in payload.conditional_tokens[0], false);
    assert.equal("image_content_type" in payload.conditional_tokens[0], false);
  });

  it("reads variant response URLs", () => {
    const response: UploadMarketDeploymentAssetsResponse = {
      market_metadata_uri: "s3://metadata/market.json",
      market: {
        banner_image_url_high: "https://cdn/banner-high.webp",
      },
      outcomes: [{
        index: 0,
        icon_url_high: "https://cdn/outcome-high.webp",
      }],
      deposit_assets: [{
        mint: "deposit-mint",
        icon_url_high: "https://cdn/deposit-high.webp",
      }],
      tokens: [{
        conditional_mint: "conditional-mint",
        metadata_uri: "s3://metadata/token.json",
        image_url_low: "https://cdn/token-low.webp",
        image_url_medium: "https://cdn/token-medium.webp",
        image_url_high: "https://cdn/token-high.webp",
      }],
    };

    assert.equal(response.deposit_assets[0]?.mint, "deposit-mint");
    assert.equal(response.tokens[0]?.image_url_high, "https://cdn/token-high.webp");
  });
});
