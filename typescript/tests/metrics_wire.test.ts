import { describe, it } from "node:test";
import assert from "node:assert/strict";
import type { PlatformMetrics } from "../src/domain/metrics";

describe("metrics wire types", () => {
  it("reads platform open interest and fee fields", () => {
    const metrics: PlatformMetrics = {
      volume_24h_usd: "1",
      volume_7d_usd: "2",
      volume_30d_usd: "3",
      volume_total_usd: "4",
      taker_bid_volume_24h_usd: "5",
      taker_bid_volume_7d_usd: "6",
      taker_bid_volume_30d_usd: "7",
      taker_bid_volume_total_usd: "8",
      taker_ask_volume_24h_usd: "9",
      taker_ask_volume_7d_usd: "10",
      taker_ask_volume_30d_usd: "11",
      taker_ask_volume_total_usd: "12",
      taker_bid_ask_imbalance_24h_pct: "13",
      taker_bid_ask_imbalance_7d_pct: "14",
      taker_bid_ask_imbalance_30d_pct: "15",
      taker_bid_ask_imbalance_total_pct: "16",
      open_interest_usd: "12345.67",
      fees_24h_usd: "0",
      fees_7d_usd: "0",
      fees_30d_usd: "0",
      unique_traders_24h: 17,
      unique_traders_7d: 18,
      unique_traders_30d: 19,
      active_markets: 20,
      active_orderbooks: 21,
      deposit_token_volumes: [],
    };

    assert.equal(metrics.open_interest_usd, "12345.67");
    assert.equal(metrics.fees_24h_usd, "0");
    assert.equal(metrics.fees_7d_usd, "0");
    assert.equal(metrics.fees_30d_usd, "0");
  });
});
