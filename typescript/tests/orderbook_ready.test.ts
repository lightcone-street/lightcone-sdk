import { describe, it } from "node:test";
import assert from "node:assert/strict";

import { LightconeClient } from "../src/client";
import { collectTickerPages } from "../src/domain/metrics/client";
import type { OrderbookTickersResponse } from "../src/domain/metrics";
import { SdkError } from "../src/error";
import { parseMessageIn } from "../src/ws";

const RULES = {
  base_size_decimals: 0,
  max_price_decimals: 4,
  max_price_significant_figures: 5,
  integer_prices_always_allowed: true,
  price_quantum: "0.0001",
  price_quantum_raw: "100",
  base_size_quantum: "1",
  base_size_quantum_raw: "1",
};

function page(tickers: number, nextCursor: string | undefined, hasMore: boolean): OrderbookTickersResponse {
  return {
    tickers: Array.from({ length: tickers }, () => ({
      orderbook_id: "ob",
      market_pubkey: "market",
      base_deposit_asset: "base",
      quote_deposit_asset: "quote",
    })),
    next_cursor: nextCursor,
    has_more: hasMore,
  };
}

async function collect(maxPages: number, pages: OrderbookTickersResponse[]) {
  const queue = [...pages];
  const cursors: Array<string | undefined> = [];
  const result = collectTickerPages(maxPages, async (cursor) => {
    cursors.push(cursor);
    const next = queue.shift();
    if (!next) throw new Error("unexpected page request");
    return next;
  });
  return { result, cursors };
}

describe("orderbook tickers paging", () => {
  it("merges pages until the last one", async () => {
    const { result, cursors } = await collect(10, [
      page(8, "a", true),
      page(0, "b", true),
      page(3, undefined, false),
    ]);
    const merged = await result;
    assert.equal(merged.tickers.length, 11);
    assert.equal(merged.has_more, false);
    assert.equal(merged.next_cursor, undefined);
    assert.deepEqual(cursors, [undefined, "a", "b"]);
  });

  it("reports truncation at the page cap", async () => {
    const { result } = await collect(2, [page(8, "a", true), page(8, "b", true)]);
    const merged = await result;
    assert.equal(merged.tickers.length, 16);
    assert.equal(merged.has_more, true);
    assert.equal(merged.next_cursor, "b");
  });

  it("fails when a page reports more without an advancing cursor", async () => {
    const isValidation = (error: unknown) => error instanceof SdkError && error.variant === "Validation";
    await assert.rejects((await collect(10, [page(8, "a", true), page(8, "a", true)])).result, isValidation);
    await assert.rejects((await collect(10, [page(8, undefined, true)])).result, isValidation);
  });

  it("decodes a page's cursor and omitted prices", async () => {
    const client = LightconeClient.builder().baseUrl("https://example.test").build();
    const urls: string[] = [];
    (client.http as unknown as { get: (url: string) => Promise<unknown> }).get = async (url) => {
      urls.push(url);
      return {
        tickers: [
          {
            orderbook_id: "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
            market_pubkey: "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
            outcome_index: 1,
            outcome_name: "Republicans Sweep",
            base_deposit_asset: "4o5Vsd7iPu97qkKypojXDbpu8BR3t5poD8ThGo8hnUKy",
            quote_deposit_asset: "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
            computed_at: "2026-09-29T12:38:41.952+00:00",
          },
        ],
        next_cursor: "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
        has_more: true,
      };
    };
    const response = await client.metrics().orderbookTickersPage(" mint ", "cursor", 8);
    assert.equal(urls[0], "https://example.test/api/metrics/orderbooks/tickers?deposit_asset=mint&cursor=cursor&limit=8");
    assert.equal(response.has_more, true);
    assert.equal(response.next_cursor, "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a");
    assert.equal(response.tickers[0]!.best_bid, undefined);
    assert.equal(response.tickers[0]!.midpoint, undefined);
  });
});

describe("orderbook decoding strictness", () => {
  function depthBody(overrides: Record<string, unknown> = {}): Record<string, unknown> {
    return {
      orderbook_id: "ob",
      bids: [{ price: "0.5000", size: "10" }],
      asks: [],
      price_quantum: "0.0001",
      trading_rules: RULES,
      revision: 275,
      captured_at_ms: 1790685521784,
      decimals: { price: 4, size: 8 },
      ready: true,
      ...overrides,
    };
  }

  function stub(body: unknown): LightconeClient {
    const client = LightconeClient.builder().baseUrl("https://example.test").build();
    (client.http as unknown as { get: () => Promise<unknown> }).get = async () => body;
    return client;
  }

  const isSerde = (error: unknown) => error instanceof SdkError && error.variant === "Serde";

  it("decodes depth levels and defaults absent flags", async () => {
    const depth = await stub(depthBody({ ready: undefined })).orderbooks().get("ob");
    assert.equal(depth.ready, false);
    assert.equal(depth.bids_truncated, false);
    assert.equal(depth.revision, 275n);
    assert.deepEqual(depth.bids, [{ price: "0.5000", size: "10" }]);
  });

  it("rejects a malformed depth body as Serde", async () => {
    for (const overrides of [
      { ready: "yes" },
      { ready: null },
      { revision: "abc" },
      { revision: "275" },
      { captured_at_ms: "1790685521784" },
      { revision: -1 },
      { captured_at_ms: undefined },
      { decimals: { price: 300, size: 8 } },
      { bids: [{ price: "x", size: "1" }] },
      { trading_rules: { ...RULES, price_quantum_raw: "0.1" } },
    ]) {
      await assert.rejects(() => stub(depthBody(overrides)).orderbooks().get("ob"), isSerde, JSON.stringify(overrides));
    }
  });

  it("rejects a malformed ticker entry as Serde", async () => {
    const entry = {
      orderbook_id: "ob",
      market_pubkey: "market",
      base_deposit_asset: "base",
      quote_deposit_asset: "quote",
    };
    for (const body of [
      { tickers: [{ ...entry, base_deposit_asset: undefined }], has_more: false },
      { tickers: [{ ...entry, best_bid: "abc" }], has_more: false },
      { tickers: [{ ...entry, outcome_index: "1" }], has_more: false },
      { tickers: [{ ...entry, computed_at: "yesterday" }], has_more: false },
      { tickers: [entry], has_more: null },
    ]) {
      await assert.rejects(() => stub(body).metrics().orderbookTickersPage(), isSerde, JSON.stringify(body));
    }
  });
});

describe("orderbook readiness", () => {
  it("exposes ready on REST depth and WS frames", async () => {
    const client = LightconeClient.builder().baseUrl("https://example.test").build();
    (client.http as unknown as { get: (url: string) => Promise<unknown> }).get = async () => ({
      market_pubkey: "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
      orderbook_id: "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
      bids: [],
      asks: [],
      tick_size: "0.1000",
      price_quantum: "0.1000",
      trading_rules: RULES,
      revision: 275,
      captured_at_ms: 1790685521784,
      decimals: { price: 4, size: 8 },
      ready: true,
    });
    const depth = await client.orderbooks().get("j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a");
    assert.equal(depth.ready, true);
    assert.equal(depth.best_bid, undefined);

    const frame = parseMessageIn(
      JSON.stringify({
        type: "book_update",
        version: 0.1,
        data: {
          orderbook_id: "ob",
          timestamp: "2026-09-29T12:00:00+00:00",
          seq: 276,
          ready: true,
          bids: [],
          asks: [],
          is_snapshot: true,
        },
      }),
    );
    assert.equal(frame.type, "book_update");
    if (frame.type === "book_update") {
      assert.equal(frame.data.ready, true);
      assert.equal(frame.data.seq, 276n);
    }
  });

  it("decodes a resync frame without seq", () => {
    // Exact `orderbook_manager::resync_signal_json` payload.
    const frame = parseMessageIn(
      JSON.stringify({
        type: "book_update",
        version: 0.1,
        data: {
          orderbook_id: "ob",
          resync: true,
          message: "Please re-subscribe to get fresh snapshot",
          n_sig_figs: 5,
          mantissa: 2,
        },
      }),
    );
    assert.equal(frame.type, "book_update");
    if (frame.type === "book_update") {
      assert.equal(frame.data.resync, true);
      assert.equal(frame.data.seq, 0n);
      assert.equal(frame.data.ready, false);
      assert.deepEqual(frame.data.bids, []);
    }
  });
});
