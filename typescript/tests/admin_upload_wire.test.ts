import { describe, it } from "node:test";
import assert from "node:assert/strict";
import type {
  AddMetadataCategoryRequest,
  AddMetadataCategoryResponse,
  DepositTokenMetadataPayload,
  MarketMetadataPayload,
  UnifiedMetadataResponse,
  UploadMarketDeploymentAssetsRequest,
  UploadMarketDeploymentAssetsResponse,
} from "../src/domain/admin";

describe("admin upload wire types", () => {
  it("serializes deposit token metadata without legacy s3 fields", () => {
    const request: DepositTokenMetadataPayload = {
      deposit_asset: "TOKEN_MINT",
      min_order_size: 1_000_000,
      binance_symbol: "BTCUSDT",
      binance_enabled: true,
      okx_inst_id: "BTC-USDT",
    };

    const payload = JSON.parse(JSON.stringify(request)) as Record<string, any>;
    assert.deepEqual(payload, {
      deposit_asset: "TOKEN_MINT",
      min_order_size: 1_000_000,
      binance_symbol: "BTCUSDT",
      binance_enabled: true,
      okx_inst_id: "BTC-USDT",
    });
    assert.equal("s3_synced" in payload, false);
    assert.equal("s3_synced_at" in payload, false);
    assert.equal("s3_error" in payload, false);
  });

  it("reads deposit token metadata response fields", () => {
    const response: UnifiedMetadataResponse = {
      deposit_tokens: [{
        id: 1,
        deposit_asset: "TOKEN_MINT",
        display_name: "Bitcoin",
        symbol: "BTC",
        token_symbol: null,
        binance_symbol: "BTCUSDT",
        binance_enabled: true,
        okx_inst_id: "BTC-USDT",
        description: null,
        icon_url_low: null,
        icon_url_medium: null,
        icon_url_high: null,
        metadata_uri: null,
        decimals: 8,
        min_order_size: 100_000,
        created_at: "2026-05-12T00:00:00Z",
        updated_at: "2026-05-12T00:00:00Z",
      }],
    };

    assert.equal(response.deposit_tokens?.[0]?.min_order_size, 100_000);
    assert.equal(response.deposit_tokens?.[0]?.okx_inst_id, "BTC-USDT");
  });

  it("uses direct category admin request and response bodies", () => {
    const request: AddMetadataCategoryRequest = { category: "Crypto" };
    const response: AddMetadataCategoryResponse = { category: "Crypto" };

    assert.deepEqual(JSON.parse(JSON.stringify(request)), { category: "Crypto" });
    assert.equal(response.category, "Crypto");
  });

  it("omits market resolution fields when they are undefined", () => {
    const request: MarketMetadataPayload = {
      market_id: 1,
      market_name: "Updated name",
    };

    const payload = JSON.parse(JSON.stringify(request)) as Record<string, any>;
    assert.deepEqual(payload, {
      market_id: 1,
      market_name: "Updated name",
    });
    assert.equal("resolution" in payload, false);
    assert.equal("resolution_by" in payload, false);
  });

  it("serializes market resolution date updates", () => {
    const request: MarketMetadataPayload = {
      market_id: 1,
      resolution_by: 1_735_689_600_000,
    };

    assert.deepEqual(JSON.parse(JSON.stringify(request)), {
      market_id: 1,
      resolution_by: 1_735_689_600_000,
    });
  });

  it("serializes explicit market resolution states", () => {
    const enabled: MarketMetadataPayload = {
      market_id: 1,
      resolution: true,
      resolution_by: 1_735_689_600_000,
    };
    const cleared: MarketMetadataPayload = {
      market_id: 1,
      resolution: false,
    };

    assert.deepEqual(JSON.parse(JSON.stringify(enabled)), {
      market_id: 1,
      resolution: true,
      resolution_by: 1_735_689_600_000,
    });
    assert.deepEqual(JSON.parse(JSON.stringify(cleared)), {
      market_id: 1,
      resolution: false,
    });
  });

  it("reads market resolution response fields", () => {
    const response: UnifiedMetadataResponse = {
      markets: [{
        id: 1,
        market_id: 1,
        resolution: true,
        resolution_by: 1_735_689_600_000,
        created_at: "2026-05-12T00:00:00Z",
        updated_at: "2026-05-12T00:00:00Z",
      }, {
        id: 2,
        market_id: 2,
        resolution: false,
        created_at: "2026-05-12T00:00:00Z",
        updated_at: "2026-05-12T00:00:00Z",
      }],
    };

    assert.equal(response.markets?.[0]?.resolution, true);
    assert.equal(response.markets?.[0]?.resolution_by, 1_735_689_600_000);
    assert.equal(response.markets?.[1]?.resolution, false);
    assert.equal(response.markets?.[1]?.resolution_by, undefined);
  });

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
