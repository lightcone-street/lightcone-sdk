import { describe, it } from "node:test";
import assert from "node:assert/strict";
import Decimal from "decimal.js";
import { decimal as decimalFmt, num } from "../src/shared/fmt";

describe("shared fmt", () => {
  it("preserves trailing zeros in formatted strings", () => {
    assert.equal(num.displayFormattedString("2.00"), "2.00");
    assert.equal(num.displayFormattedString("1234.500"), "1,234.500");
  });

  it("preserves selected decimal places for numbers", () => {
    assert.equal(num.display(2), "2.00");
    assert.equal(num.display(0), "0.00");
    assert.equal(num.display(100), "100.00");
    assert.equal(num.display(1234.567), "1,234.57");
    assert.equal(num.display(0.1), "0.10");
    assert.equal(num.display(0.004), "0.004");
    assert.equal(num.display(0.0000005), "0.0000005");
    assert.equal(num.display(0.000000001), "0.0(8)1");
    assert.equal(num.displayWithDecimals(2, 3), "2.000");
  });

  it("preserves selected decimal places for decimals", () => {
    assert.equal(decimalFmt.display(new Decimal("0")), "0.00");
    assert.equal(decimalFmt.display(new Decimal("2.00")), "2.00");
    assert.equal(decimalFmt.display(new Decimal("2.5")), "2.50");
    assert.equal(decimalFmt.display(new Decimal("100")), "100.00");
    assert.equal(decimalFmt.display(new Decimal("1234.567")), "1,234.57");
    assert.equal(decimalFmt.display(new Decimal("0.1")), "0.10");
    assert.equal(decimalFmt.display(new Decimal("0.004")), "0.004");
    assert.equal(decimalFmt.display(new Decimal("0.0000005")), "0.0000005");
    assert.equal(decimalFmt.display(new Decimal("0.000000001")), "0.0(8)1");
  });

  it("preserves abbreviation precision", () => {
    assert.equal(decimalFmt.abbrNumber(new Decimal("1000")), "1.00k");
    assert.equal(decimalFmt.abbrNumber(new Decimal("1500")), "1.50k");
  });
});
