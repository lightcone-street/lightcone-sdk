import { describe, it } from "node:test";
import assert from "node:assert/strict";
import Decimal from "decimal.js";

import {
  decimalTextIsNegative,
  decimalTextToDecimal,
  parseDecimalText,
  wireDecimalText,
} from "../src/shared/decimal_text";
import { quotePerBase } from "../src/shared/price";
import { FundingSource, Side } from "../src/shared/types";
import {
  wireCommitInfo,
  wireDecimal,
  wireFundingSource,
  wireI128,
  wireKnownOrUndefined,
  wireNonEmptyString,
  wireOptI128,
  wireOptU64,
  wireSide,
  wireU64,
} from "../src/shared/wire_decode";
import { parseJsonExact } from "../src/shared/json";
import { SdkError } from "../src/error";

const U64_MAX = (1n << 64n) - 1n;
const I128_MIN = -(1n << 127n);

function isSerde(error: unknown): boolean {
  return error instanceof SdkError && error.variant === "Serde";
}

describe("committed wire encodings", () => {
  it("accepts integers as text or numbers", () => {
    const data = parseJsonExact<Record<string, unknown>>(
      '{"revision":"18446744073709551615","slot":7,"fee":"-170141183460469231731687303715884105728","remaining":null,"side":1,"reason":""}',
    );
    assert.equal(wireU64(data.revision, "revision"), U64_MAX);
    assert.equal(wireOptU64(data.slot, "slot"), 7n);
    assert.equal(wireI128(data.fee, "fee"), I128_MIN);
    assert.equal(wireOptI128(data.remaining, "remaining"), undefined);
    assert.equal(wireSide(data.side, "side"), Side.Ask);
    assert.equal(wireNonEmptyString(data.reason, "reason"), undefined);
  });

  it("accepts integer tokens json-bigint keeps as bigint", () => {
    const data = parseJsonExact<Record<string, unknown>>('{"revision":18446744073709551615}');
    assert.equal(typeof data.revision, "bigint");
    assert.equal(wireU64(data.revision, "revision"), U64_MAX);
  });

  it("decodes text sides and kept reasons", () => {
    assert.equal(wireU64(42, "revision"), 42n);
    assert.equal(wireSide("bid", "side"), Side.Bid);
    assert.equal(wireNonEmptyString("expired", "reason"), "expired");
  });

  it("rejects malformed or out-of-range integers and sides", () => {
    for (const value of ["-1", "+1", "1.0", "18446744073709551616", -1, "", " 1", 1.5, true]) {
      assert.throws(() => wireU64(value, "revision"), isSerde, String(value));
    }
    assert.throws(() => wireI128("", "fee"), isSerde);
    assert.throws(() => wireI128((1n << 127n).toString(), "fee"), isSerde);
    assert.throws(() => wireSide(2, "side"), isSerde);
    assert.throws(() => wireSide("1", "side"), isSerde);
  });

  it("names the field in decode failures", () => {
    assert.throws(
      () => wireU64("x", "order.committed_revision"),
      (error) => isSerde(error) && /order\.committed_revision/.test((error as Error).message),
    );
  });

  it("keeps plain decimal text exact and rejects other forms", () => {
    assert.equal(wireDecimal("12.500000", "amount"), "12.500000");
    assert.equal(wireDecimal("-0.000001", "amount"), "-0.000001");
    for (const value of [12, "1e3", "+1", ".5", "1.", " 1", "NaN", "0x1"]) {
      assert.throws(() => wireDecimal(value, "amount"), isSerde, String(value));
    }
  });

  it("maps unknown enum values without failing", () => {
    assert.equal(wireFundingSource("global", "source"), FundingSource.Global);
    assert.equal(wireFundingSource("vault", "source"), FundingSource.Unknown);
    assert.equal(wireKnownOrUndefined("unsupported", "tif", ["GTC", "IOC", "FOK"]), undefined);
    assert.equal(wireKnownOrUndefined("IOC", "tif", ["GTC", "IOC", "FOK"]), "IOC");
  });

  it("decodes flattened commit metadata", () => {
    const commit = wireCommitInfo(
      { effect_id: "9:0", committed_revision: 9, projection_generation: "2" },
      "fact",
    );
    assert.deepEqual(commit, {
      effect_id: "9:0",
      committed_revision: 9n,
      projection_generation: 2n,
      actionable: false,
    });
  });
});

describe("DecimalText", () => {
  it("keeps u128 aggregates exact beyond 96-bit decimal range", () => {
    const text = wireDecimalText("340282366920938463463374607431768.211455", "order_reserved");
    assert.equal(text, "340282366920938463463374607431768.211455");
    assert.equal(
      decimalTextToDecimal(text).toFixed(),
      "340282366920938463463374607431768.211455",
    );
  });

  it("converts values and reports sign", () => {
    const text = parseDecimalText("-0.000001");
    assert.ok(decimalTextIsNegative(text));
    assert.ok(decimalTextToDecimal(text).eq(new Decimal("-0.000001")));
    assert.ok(decimalTextToDecimal(parseDecimalText("40")).eq(40));
  });

  it("rejects non-canonical text and JSON numbers", () => {
    for (const text of ["", "-", "1.", ".5", "+1", "1e3", " 1", "-0", "-0.000", "0x1", "1.2.3"]) {
      assert.throws(() => parseDecimalText(text), RangeError, JSON.stringify(text));
    }
    assert.throws(() => wireDecimalText(12, "order_reserved"), isSerde);
  });
});

describe("quotePerBase", () => {
  it("derives a normalized exact price", () => {
    assert.equal(quotePerBase("2.2", "4"), "0.55");
    assert.equal(quotePerBase("100", "1"), "100");
    assert.equal(quotePerBase("1", "3"), "0.3333333333333333333333333333");
  });

  it("has no price for a zero base amount", () => {
    assert.equal(quotePerBase("1", "0"), undefined);
  });
});
