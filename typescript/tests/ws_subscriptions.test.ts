import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  aggregationFromFrame,
  aggregationKeySuffix,
  isFullPrecision,
  validateAggregation,
} from "../src/domain/orderbook/aggregation";
import {
  subscriptionKey,
  unsubscribeMatches,
  type SubscribeParams,
  type UnsubscribeParams,
} from "../src/ws/subscriptions";
import { parseMessageIn, subscribeBooks } from "../src/ws";
import type { OrderBookId } from "../src/shared";

const id = (value: string) => value as OrderBookId;

describe("BookAggregation", () => {
  it("validates against the backend contract", () => {
    assert.deepStrictEqual(validateAggregation({}), {});
    assert.deepStrictEqual(validateAggregation({ nSigFigs: 3 }), { nSigFigs: 3 });
    // (5, none) normalizes to (5, 1).
    assert.deepStrictEqual(validateAggregation({ nSigFigs: 5 }), {
      nSigFigs: 5,
      mantissa: 1,
    });
    assert.deepStrictEqual(validateAggregation({ nSigFigs: 5, mantissa: 5 }), {
      nSigFigs: 5,
      mantissa: 5,
    });

    assert.throws(() => validateAggregation({ nSigFigs: 1 }));
    assert.throws(() => validateAggregation({ nSigFigs: 6 }));
    assert.throws(() => validateAggregation({ nSigFigs: 4, mantissa: 2 }));
    assert.throws(() => validateAggregation({ mantissa: 2 }));
    assert.throws(() => validateAggregation({ nSigFigs: 5, mantissa: 3 }));
  });

  it("treats untagged frames as full precision", () => {
    assert.equal(isFullPrecision(aggregationFromFrame(undefined, undefined)), true);
    assert.equal(isFullPrecision(aggregationFromFrame(4, undefined)), false);
  });

  it("produces backend-vocabulary key suffixes", () => {
    assert.equal(aggregationKeySuffix({}), "full");
    assert.equal(aggregationKeySuffix({ nSigFigs: 2 }), "sig2");
    assert.equal(aggregationKeySuffix({ nSigFigs: 5 }), "sig5m1");
    assert.equal(aggregationKeySuffix({ nSigFigs: 5, mantissa: 2 }), "sig5m2");
  });
});

describe("book subscription identity", () => {
  it("full precision keeps the pre-aggregation key shape", () => {
    const params: SubscribeParams = {
      type: "book_update",
      orderbook_ids: [id("b"), id("a")],
    };
    assert.equal(subscriptionKey(params), "book:a,b");
  });

  it("aggregated keys are distinct and normalized", () => {
    const grouped: SubscribeParams = {
      type: "book_update",
      orderbook_ids: [id("a")],
      nSigFigs: 5,
      mantissa: 2,
    };
    assert.equal(subscriptionKey(grouped), "book:a:sig5m2");

    const implicit: SubscribeParams = {
      type: "book_update",
      orderbook_ids: [id("a")],
      nSigFigs: 5,
    };
    const explicit: SubscribeParams = {
      type: "book_update",
      orderbook_ids: [id("a")],
      nSigFigs: 5,
      mantissa: 1,
    };
    assert.equal(subscriptionKey(implicit), subscriptionKey(explicit));
  });

  it("unsubscribe matches on ids and normalized aggregation", () => {
    const subscribe: SubscribeParams = {
      type: "book_update",
      orderbook_ids: [id("a")],
      nSigFigs: 5,
    };
    const normalized: UnsubscribeParams = {
      type: "book_update",
      orderbook_ids: [id("a")],
      nSigFigs: 5,
      mantissa: 1,
    };
    const fullPrecision: UnsubscribeParams = {
      type: "book_update",
      orderbook_ids: [id("a")],
    };
    const otherGrouping: UnsubscribeParams = {
      type: "book_update",
      orderbook_ids: [id("a")],
      nSigFigs: 5,
      mantissa: 2,
    };

    // (5, none) and (5, 1) are the same subscription.
    assert.equal(unsubscribeMatches(subscribe, normalized), true);
    // A grouped subscription is never matched by a full-precision or
    // differently grouped unsubscribe.
    assert.equal(unsubscribeMatches(subscribe, fullPrecision), false);
    assert.equal(unsubscribeMatches(subscribe, otherGrouping), false);
  });
});

describe("book message builders", () => {
  it("full-precision messages omit aggregation keys entirely", () => {
    const message = JSON.stringify(subscribeBooks([id("abc")]));
    assert.equal(message.includes("nSigFigs"), false);
    assert.equal(message.includes("mantissa"), false);
    assert.equal(message.includes("n_sig_figs"), false);
  });

  it("aggregated messages use camelCase wire keys and normalize", () => {
    const message = subscribeBooks([id("abc")], { nSigFigs: 5 });
    const wire = JSON.parse(JSON.stringify(message)) as {
      params: Record<string, unknown>;
    };
    assert.equal(wire.params["nSigFigs"], 5);
    // (5, none) is sent in its normalized form (5, 1).
    assert.equal(wire.params["mantissa"], 1);
    assert.equal("n_sig_figs" in wire.params, false);
  });
});

describe("book quote notional decoding", () => {
  it("preserves exact values for full-precision and grouped bid/ask levels", () => {
    const full = parseMessageIn(JSON.stringify({
      type: "book_update",
      version: 0.1,
      data: {
        orderbook_id: "ob1",
        is_snapshot: true,
        seq: 10,
        bids: [{ side: "bid", price: "65000", size: "0.03", quote_notional: "1948.01" }],
        asks: [{ side: "ask", price: "65001", size: "0.02", quote_notional: "1300.02" }],
      },
    }));
    assert.equal(full.type, "book_update");
    if (full.type !== "book_update") throw new Error("expected book_update");
    assert.equal(full.data.bids[0]?.quote_notional, "1948.01");
    assert.equal(full.data.asks[0]?.quote_notional, "1300.02");

    const grouped = parseMessageIn(JSON.stringify({
      type: "book_update",
      version: 0.1,
      data: {
        orderbook_id: "ob1",
        is_snapshot: false,
        seq: 11,
        n_sig_figs: 5,
        mantissa: 2,
        bids: [{ side: "bid", price: "100", size: "2", quote_notional: "199" }],
        asks: [{ side: "ask", price: "101", size: "3", quote_notional: "304" }],
      },
    }));
    assert.equal(grouped.type, "book_update");
    if (grouped.type !== "book_update") throw new Error("expected book_update");
    assert.equal(grouped.data.bids[0]?.quote_notional, "199");
    assert.notEqual(grouped.data.bids[0]?.quote_notional, "200");
    assert.equal(grouped.data.asks[0]?.quote_notional, "304");
    assert.notEqual(grouped.data.asks[0]?.quote_notional, "303");
  });

  it("rejects malformed populated levels without changing empty-frame tolerance", () => {
    assert.doesNotThrow(() => parseMessageIn(JSON.stringify({
      type: "book_update",
      version: 0.1,
      data: { orderbook_id: "ob1", seq: 1 },
    })));

    for (const level of [
      null,
      { side: "bid", price: "1", size: "2" },
      { side: "bid", price: "1", size: "2", quote_notional: 2 },
    ]) {
      assert.throws(
        () => parseMessageIn(JSON.stringify({
          type: "book_update",
          version: 0.1,
          data: { orderbook_id: "ob1", seq: 1, bids: [level], asks: [] },
        })),
        /Invalid book_update bids level/,
      );
    }
  });
});
