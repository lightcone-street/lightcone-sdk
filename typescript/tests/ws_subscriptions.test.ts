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
import { subscribeBooks } from "../src/ws";
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
