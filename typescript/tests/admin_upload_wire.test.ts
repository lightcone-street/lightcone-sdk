import { describe, it } from "node:test";
import assert from "node:assert/strict";
import type {
  AddMetadataCategoryRequest,
  AddMetadataCategoryResponse,
  AdminMarketsQuery,
  AdminMarketsResponse,
  AdminMarketStatusFilter,
  CriticalLogErrors24hCountResponse,
  DepositTokenMetadataPayload,
  MarketMetadataPayload,
  MarketsToSettleCountResponse,
  MarketsToSettleQuery,
  MarketsToSettleResponse,
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
    assert.equal(response.deposit_tokens?.[0]?.binance_symbol, "BTCUSDT");
    assert.equal(response.deposit_tokens?.[0]?.okx_inst_id, "BTC-USDT");
  });

  it("uses direct category admin request and response bodies", () => {
    const request: AddMetadataCategoryRequest = { category: "Crypto" };
    const response: AddMetadataCategoryResponse = { category: "Crypto" };

    assert.deepEqual(JSON.parse(JSON.stringify(request)), { category: "Crypto" });
    assert.equal(response.category, "Crypto");
  });

  it("omits market resolution_by when it is undefined", () => {
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

  it("serializes market resolution_by timestamp updates", () => {
    const request: MarketMetadataPayload = {
      market_id: 1,
      resolution_by: 1_735_689_600_000,
    };

    assert.deepEqual(JSON.parse(JSON.stringify(request)), {
      market_id: 1,
      resolution_by: 1_735_689_600_000,
    });
  });

  it("serializes market resolution_by null clears", () => {
    const request: MarketMetadataPayload = {
      market_id: 1,
      resolution_by: null,
    };

    assert.deepEqual(JSON.parse(JSON.stringify(request)), {
      market_id: 1,
      resolution_by: null,
    });
  });

  it("reads market resolution_by response values", () => {
    const response: UnifiedMetadataResponse = {
      markets: [{
        id: 1,
        market_id: 1,
        resolution_by: 1_735_689_600_000,
        created_at: "2026-05-12T00:00:00Z",
        updated_at: "2026-05-12T00:00:00Z",
      }, {
        id: 2,
        market_id: 2,
        resolution_by: null,
        created_at: "2026-05-12T00:00:00Z",
        updated_at: "2026-05-12T00:00:00Z",
      }],
    };

    assert.equal(response.markets?.[0]?.resolution_by, 1_735_689_600_000);
    assert.equal(response.markets?.[1]?.resolution_by, null);
  });

  it("serializes admin markets status filters and range query fields", () => {
    const status: AdminMarketStatusFilter = "resolved";
    const query: AdminMarketsQuery = {
      cursor: 100,
      limit: 50,
      sort_by: "open_interest_usd",
      sort_direction: "asc",
      market_status: status,
      category: "Crypto",
      search: "btc",
      min_volume_24h_usd: "1000",
      max_open_interest_usd: "50000",
      min_unique_traders_total: 10,
    };
    const params = new URLSearchParams();
    for (const [key, value] of Object.entries(query)) {
      if (value !== undefined && value !== null) {
        params.append(key, String(value));
      }
    }

    assert.equal(
      params.toString(),
      "cursor=100&limit=50&sort_by=open_interest_usd&sort_direction=asc&market_status=resolved&category=Crypto&search=btc&min_volume_24h_usd=1000&max_open_interest_usd=50000&min_unique_traders_total=10"
    );
  });

  it("reads admin markets response shape", () => {
    const response: AdminMarketsResponse = {
      timestamp: 1_710_000_000_000,
      sort_by: "volume_24h_usd",
      sort_direction: "desc",
      total: 123,
      limit: 100,
      next_cursor: 100,
      has_more: true,
      markets: [{
        rank: 1,
        market_id: 123,
        market_pubkey: "market-pubkey",
        market_status: "Active",
        slug: "btc-100k",
        market_name: "Will BTC hit $100k?",
        category: "Crypto",
        icon_url: "https://example.com/icon.png",
        num_outcomes: 2,
        resolution_by: 1_760_000_000_000,
        open_interest_usd: "12345.67",
        volume_24h_usd: "1000.00",
        volume_7d_usd: "7000.00",
        volume_30d_usd: "30000.00",
        volume_total_usd: "50000.00",
        unique_traders_24h: 50,
        unique_traders_7d: 200,
        unique_traders_30d: 600,
        unique_traders_total: 900,
        fees_24h_usd: "0",
        fees_7d_usd: "0",
        fees_30d_usd: "0",
        fees_total_usd: "0",
        created_at: "2026-01-01T00:00:00+00:00",
        activated_at: "2026-01-02T00:00:00+00:00",
        settled_at: null,
        updated_at: "2026-01-03T00:00:00+00:00",
      }],
    };

    assert.equal(response.markets[0]?.market_status, "Active");
    assert.equal(response.markets[0]?.resolution_by, 1_760_000_000_000);
    assert.equal(response.markets[0]?.open_interest_usd, "12345.67");
    assert.equal(response.markets[0]?.unique_traders_total, 900);
    assert.equal(response.markets[0]?.fees_total_usd, "0");
    assert.equal(response.next_cursor, 100);
  });

  it("reads markets-to-settle admin response shapes", () => {
    const count: MarketsToSettleCountResponse = {
      markets_to_settle_count: 3,
    };
    const query: MarketsToSettleQuery = {
      cursor: 123,
      limit: 200,
    };
    const response: MarketsToSettleResponse = {
      markets: [{
        market_id: 123,
        market_pubkey: "market-pubkey",
        market_status: "Active",
        market_name: "Market",
        slug: "market",
        outcomes: [],
        deposit_assets: [],
        orderbooks: [],
        oracle: "oracle",
        question_id: "question",
        condition_id: "condition",
        created_at: "2026-05-12T00:00:00Z",
      }],
      next_cursor: 456,
      has_more: true,
    };

    assert.equal(count.markets_to_settle_count, 3);
    assert.deepEqual(JSON.parse(JSON.stringify(query)), {
      cursor: 123,
      limit: 200,
    });
    assert.equal(response.markets[0]?.market_id, 123);
    assert.equal(response.next_cursor, 456);
    assert.equal(response.has_more, true);
  });

  it("reads critical log error count response shape", () => {
    const response: CriticalLogErrors24hCountResponse = {
      critical_log_errors_24h: 1,
    };

    assert.equal(response.critical_log_errors_24h, 1);
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
