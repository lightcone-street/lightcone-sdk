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
  wireArray,
  wireBool,
  wireCommitInfo,
  wireDecimal,
  wireFundingSource,
  wireI128,
  wireI64,
  wireInt,
  wireKnownOrUndefined,
  wireNonEmptyString,
  wireOptI128,
  wireOptU64,
  wireRfc3339,
  wireSide,
  wireSideText,
  wireStringOr,
  wireTimestampMs,
  wireU64,
  wireU64Number,
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

  it("decodes plain integers and REST sides as the Rust SDK does", () => {
    assert.equal(wireI64(-5, "id"), -5n);
    assert.equal(wireInt(30, "fee_bps", -32768, 32767), 30);
    for (const value of ["5", "-5", 1.5]) {
      assert.throws(() => wireI64(value, "id"), isSerde, String(value));
      assert.throws(() => wireInt(value, "fee_bps", -32768, 32767), isSerde, String(value));
    }
    assert.equal(wireSideText("sell", "side"), Side.Ask);
    assert.throws(() => wireSideText(0, "side"), isSerde);
    assert.throws(() => wireSideText(1n, "side"), isSerde);
  });

  it("reads wide integers from text only, as Rust's u64-or-i64 JSON numbers do", () => {
    assert.equal(wireI128(-(1n << 63n), "fee"), -(1n << 63n));
    assert.equal(wireI128("-170141183460469231731687303715884105728", "fee"), -(1n << 127n));
    for (const value of [1n << 64n, -(1n << 63n) - 1n]) {
      assert.throws(() => wireI128(value, "fee"), isSerde, String(value));
    }
  });

  it("accepts only JSON numbers for plain u64 values and millisecond times", () => {
    assert.equal(wireU64Number(275, "revision"), 275n);
    assert.equal(wireU64Number(18446744073709551615n, "revision"), 18446744073709551615n);
    assert.equal(wireTimestampMs(1790685521784, "created_at").toISOString(), "2026-09-29T12:38:41.784Z");
    for (const value of ["275", "1790685521784", -1, 1.5, true]) {
      assert.throws(() => wireU64Number(value, "revision"), isSerde, String(value));
      assert.throws(() => wireTimestampMs(value, "created_at"), isSerde, String(value));
    }
  });

  it("decodes RFC 3339 with an offset, including chrono's relaxed forms", () => {
    for (const [text, iso] of [
      ["2026-09-29T12:00:00Z", "2026-09-29T12:00:00.000Z"],
      ["2026-09-29t12:00:00z", "2026-09-29T12:00:00.000Z"],
      ["2026-09-29 12:00:00Z", "2026-09-29T12:00:00.000Z"],
      ["2026-09-29T14:00:00.987654321+02:00", "2026-09-29T12:00:00.987Z"],
      ["2024-02-29T00:00:00Z", "2024-02-29T00:00:00.000Z"],
      ["0050-06-01T00:00:00Z", "0050-06-01T00:00:00.000Z"],
      ["2026-09-29T12:00:00.5Z", "2026-09-29T12:00:00.500Z"],
    ] as const) {
      assert.equal(wireRfc3339(text, "executed_at").toISOString(), iso, text);
    }
    for (const text of [
      "2026-09-29T12:00:00",
      "2026-09-29",
      "20260929T120000Z",
      "2026-09-29T12:00:00+0000",
      "2026-02-30T00:00:00Z",
      "2026-09-29T24:00:00Z",
      "2026-09-29T12:00:00+24:00",
      "2026-09-29T12:00:00Z ",
    ]) {
      assert.throws(() => wireRfc3339(text, "executed_at"), isSerde, text);
    }
  });

  it("defaults absent fields but rejects null, as serde(default) does", () => {
    assert.equal(wireBool(undefined, "has_more", false), false);
    assert.deepEqual(wireArray(undefined, "fills", (item) => item, true), []);
    assert.equal(wireStringOr(undefined, "order_type", ""), "");
    assert.throws(() => wireBool(null, "has_more", false), isSerde);
    assert.throws(() => wireArray(null, "fills", (item) => item, true), isSerde);
    assert.throws(() => wireStringOr(null, "order_type", ""), isSerde);
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
    // Beyond the Rust SDK decimal's range, as Rust's `to_decimal()` reports.
    assert.equal(decimalTextToDecimal(text), undefined);
  });

  it("converts values within the 96-bit, 28-scale range judged on the literal text", () => {
    const text = parseDecimalText("-0.000001");
    assert.ok(decimalTextIsNegative(text));
    assert.ok(decimalTextToDecimal(text)?.eq(new Decimal("-0.000001")));
    assert.ok(decimalTextToDecimal(parseDecimalText("40"))?.eq(40));
    const maxMantissa = "79228162514264337593543950335";
    assert.equal(decimalTextToDecimal(parseDecimalText(maxMantissa))?.toFixed(), maxMantissa);
    assert.equal(decimalTextToDecimal(parseDecimalText("79228162514264337593543950336")), undefined);
    assert.ok(decimalTextToDecimal(parseDecimalText(`0.${"0".repeat(27)}1`)));
    // Trailing zeros still count toward the scale, as `from_str_exact` does.
    assert.equal(decimalTextToDecimal(parseDecimalText(`1.${"0".repeat(29)}`)), undefined);
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

  // Known answers from rust_decimal 1.40 `checked_div(..).normalize()`.
  it("matches the Rust SDK's 96-bit decimal division", () => {
    assert.equal(quotePerBase("10", "3"), "3.3333333333333333333333333333");
    assert.equal(quotePerBase("1", "3000000"), "0.0000003333333333333333333333");
    assert.equal(quotePerBase("123456.789012", "7"), "17636.684144571428571428571429");
    assert.equal(quotePerBase("0.150000", "3.00000001"), "0.049999999833333333888888887");
    assert.equal(quotePerBase("12", "0.7"), "17.142857142857142857142857143");
    assert.equal(quotePerBase("1", "81"), "0.012345679012345679012345679");
    assert.equal(quotePerBase("-1", "3"), "-0.3333333333333333333333333333");
  });

  it("has no price for a zero base amount or an out-of-range quotient", () => {
    assert.equal(quotePerBase("1", "0"), undefined);
    assert.equal(quotePerBase("0.5", "0.0000000000000000000000000001"), "5000000000000000000000000000");
    assert.equal(quotePerBase("79228162514264337593543950335", "0.1"), undefined);
  });
});
