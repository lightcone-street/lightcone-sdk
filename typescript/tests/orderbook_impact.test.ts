import { describe, it } from "node:test";
import assert from "node:assert/strict";
import Decimal from "decimal.js";
import { ImpactDirection, impact, impactSign } from "../src/domain/orderbook";

describe("orderbook impact", () => {
  it("classifies positive impact", () => {
    const result = impact(new Decimal(100), new Decimal(125));

    assert.deepEqual(result, {
      direction: ImpactDirection.Positive,
      pct: 25,
      dollar: "25",
    });
    assert.equal(impactSign(result.direction), "+");
  });

  it("classifies zero impact", () => {
    const result = impact(new Decimal(100), new Decimal(100));

    assert.deepEqual(result, {
      direction: ImpactDirection.Zero,
      pct: 0,
      dollar: "0",
    });
    assert.equal(impactSign(result.direction), "");
  });

  it("classifies negative impact", () => {
    const result = impact(new Decimal(100), new Decimal(75));

    assert.deepEqual(result, {
      direction: ImpactDirection.Negative,
      pct: 25,
      dollar: "25",
    });
    assert.equal(impactSign(result.direction), "-");
  });

  it("returns zero impact when the deposit price is zero", () => {
    assert.deepEqual(impact(new Decimal(0), new Decimal(75)), {
      direction: ImpactDirection.Zero,
      pct: 0,
      dollar: "0",
    });
  });
});
