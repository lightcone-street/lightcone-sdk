import { describe, it } from "node:test";
import assert from "node:assert/strict";
import Decimal from "decimal.js";
import { decimal as decimalFmt, num } from "../src/shared/fmt";

describe("shared fmt", () => {
  it("preserves trailing zeros in formatted strings", () => {
    assert.equal(num.displayFormattedString("2.00"), "2.00");
    assert.equal(num.displayFormattedString("1234.500"), "1,234.500");
  });

  it("formats numbers with magnitude-based decimal places", () => {
    assert.equal(num.display(12345.67), "12,346");
    assert.equal(num.display(1234.56), "1,234.6");
    assert.equal(num.display(123.456), "123.46");
    assert.equal(num.display(15.4567), "15.457");
    assert.equal(num.display(1.23456), "1.2346");
    assert.equal(num.display(0.123456), "0.1235");
    assert.equal(num.display(0.012345), "0.01235");
    assert.equal(num.displayWithDecimals(2, 3), "2.000");
  });

  it("formats tier boundaries", () => {
    assert.equal(num.display(10000), "10,000");
    assert.equal(num.display(9999.99), "10,000.0");
    assert.equal(num.display(1000), "1,000.0");
    assert.equal(num.display(999.999), "1,000.00");
    assert.equal(num.display(100), "100.00");
    assert.equal(num.display(99.9999), "100.000");
    assert.equal(num.display(10), "10.000");
    assert.equal(num.display(9.87654), "9.8765");
    assert.equal(num.display(1), "1.0000");
    assert.equal(num.display(0.999999), "1.0000");
    assert.equal(num.display(0.1), "0.1000");
    assert.equal(num.display(0.099999), "0.10000");
  });

  it("caps small numbers at five decimals", () => {
    assert.equal(num.display(0), "0.00000");
    assert.equal(num.display(0.01), "0.01000");
    assert.equal(num.display(0.00003), "0.00003");
    assert.equal(num.display(0.000004), "0.00000");
    assert.equal(num.display(0.000000001), "0.00000");
  });

  it("formats decimals with magnitude-based decimal places", () => {
    assert.equal(decimalFmt.display(new Decimal("12345.67")), "12,346");
    assert.equal(decimalFmt.display(new Decimal("1234.56")), "1,234.6");
    assert.equal(decimalFmt.display(new Decimal("123.456")), "123.46");
    assert.equal(decimalFmt.display(new Decimal("15.4567")), "15.457");
    assert.equal(decimalFmt.display(new Decimal("1.23456")), "1.2346");
    assert.equal(decimalFmt.display(new Decimal("0.123456")), "0.1235");
    assert.equal(decimalFmt.display(new Decimal("0.012345")), "0.01235");
  });

  it("formats decimal tier boundaries", () => {
    assert.equal(decimalFmt.display(new Decimal("10000")), "10,000");
    assert.equal(decimalFmt.display(new Decimal("9999.99")), "10,000.0");
    assert.equal(decimalFmt.display(new Decimal("1000")), "1,000.0");
    assert.equal(decimalFmt.display(new Decimal("999.999")), "1,000.00");
    assert.equal(decimalFmt.display(new Decimal("100")), "100.00");
    assert.equal(decimalFmt.display(new Decimal("99.9999")), "100.000");
    assert.equal(decimalFmt.display(new Decimal("10")), "10.000");
    assert.equal(decimalFmt.display(new Decimal("9.87654")), "9.8765");
    assert.equal(decimalFmt.display(new Decimal("1")), "1.0000");
    assert.equal(decimalFmt.display(new Decimal("0.999999")), "1.0000");
    assert.equal(decimalFmt.display(new Decimal("0.1")), "0.1000");
    assert.equal(decimalFmt.display(new Decimal("0.099999")), "0.10000");
  });

  it("caps small decimals at five decimals", () => {
    assert.equal(decimalFmt.display(new Decimal("0")), "0.00000");
    assert.equal(decimalFmt.display(new Decimal("0.01")), "0.01000");
    assert.equal(decimalFmt.display(new Decimal("0.00003")), "0.00003");
    assert.equal(decimalFmt.display(new Decimal("0.000004")), "0.00000");
    assert.equal(decimalFmt.display(new Decimal("0.000000001")), "0.00000");
  });

  it("preserves abbreviation precision", () => {
    assert.equal(decimalFmt.abbrNumber(new Decimal("1000")), "1.00k");
    assert.equal(decimalFmt.abbrNumber(new Decimal("1500")), "1.50k");
  });
});
