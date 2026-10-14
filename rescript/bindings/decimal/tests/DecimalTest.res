open RescriptBun.Test
open RescriptBun.Test.Expect

// Runtime tests for the decimal.js binding — run the compiled .res.mjs under Bun to prove
// the JS method names (`plus`/`times`/`cmp`/`toFixed`/...), arg order, and return shapes.
// decimal.js is a pure utility: we call each binding directly and assert the value.

describe("Decimal — construction", () => {
  test("fromString builds an exact decimal", () =>
    expect(Decimal.fromString("1.5")->Decimal.toString)->toBe("1.5")
  )
  test("fromInt builds from an int", () => expect(Decimal.fromInt(42)->Decimal.toString)->toBe("42"))
  test("fromFloat builds from a JS number", () =>
    expect(Decimal.fromFloat(2.5)->Decimal.toString)->toBe("2.5")
  )
})

describe("Decimal — arithmetic", () => {
  test("times: 1.5 * 2 -> toFixed(1) == \"3.0\"", () =>
    expect(Decimal.fromString("1.5")->Decimal.times(Decimal.fromInt(2))->Decimal.toFixed(1))->toBe(
      "3.0",
    )
  )
  test("plus is exact: 0.1 + 0.2 == 0.3 (no binary float drift)", () =>
    expect(
      Decimal.fromString("0.1")->Decimal.plus(Decimal.fromString("0.2"))->Decimal.toFixed(1),
    )->toBe("0.3")
  )
  test("minus: 10 - 3 == 7", () =>
    expect(Decimal.fromInt(10)->Decimal.minus(Decimal.fromInt(3))->Decimal.toString)->toBe("7")
  )
  test("div: 10 / 4 == 2.5", () =>
    expect(Decimal.fromInt(10)->Decimal.div(Decimal.fromInt(4))->Decimal.toString)->toBe("2.5")
  )
  test("pow: 2 ^ 10 == 1024 (exponent is a Decimal)", () =>
    expect(Decimal.fromInt(2)->Decimal.pow(Decimal.fromInt(10))->Decimal.toString)->toBe("1024")
  )
  test("powInt: 10 ^ 3 == 1000 (int exponent convenience)", () =>
    expect(Decimal.fromInt(10)->Decimal.powInt(3)->Decimal.toString)->toBe("1000")
  )
})

describe("Decimal — rounding and sign", () => {
  test("abs: |-5.5| == 5.5", () =>
    expect(Decimal.fromString("-5.5")->Decimal.abs->Decimal.toString)->toBe("5.5")
  )
  test("floor: 2.7 -> 2", () =>
    expect(Decimal.fromString("2.7")->Decimal.floor->Decimal.toString)->toBe("2")
  )
  test("ceil: 2.3 -> 3", () =>
    expect(Decimal.fromString("2.3")->Decimal.ceil->Decimal.toString)->toBe("3")
  )
  test("round: 2.6 -> 3 (ROUND_HALF_UP default)", () =>
    expect(Decimal.fromString("2.6")->Decimal.round->Decimal.toString)->toBe("3")
  )
  test("round: 2.4 -> 2", () =>
    expect(Decimal.fromString("2.4")->Decimal.round->Decimal.toString)->toBe("2")
  )
  test("toSignificantDigits: 123.456 at 2 sig digits -> 120", () =>
    expect(Decimal.fromString("123.456")->Decimal.toSignificantDigits(2)->Decimal.toString)->toBe(
      "120",
    )
  )
  test("toDecimalPlaces with roundDown truncates: 1.2399 at 2 dp -> 1.23", () =>
    expect(
      Decimal.fromString("1.2399")
      ->Decimal.toDecimalPlaces(2, Decimal.roundDown)
      ->Decimal.toString,
    )->toBe("1.23")
  )
})

describe("Decimal — comparison and predicates", () => {
  test("cmp: 1 vs 2 == -1", () =>
    expect(Decimal.fromInt(1)->Decimal.cmp(Decimal.fromInt(2)))->toBe(-1)
  )
  test("cmp: 2 vs 2 == 0", () =>
    expect(Decimal.fromInt(2)->Decimal.cmp(Decimal.fromInt(2)))->toBe(0)
  )
  test("cmp: 3 vs 2 == 1", () =>
    expect(Decimal.fromInt(3)->Decimal.cmp(Decimal.fromInt(2)))->toBe(1)
  )
  test("eq: 1.0 == 1", () =>
    expect(Decimal.fromString("1.0")->Decimal.eq(Decimal.fromInt(1)))->toBe(true)
  )
  test("gt: 2 > 1", () =>
    expect(Decimal.fromInt(2)->Decimal.gt(Decimal.fromInt(1)))->toBe(true)
  )
  test("gte: 2 >= 2", () =>
    expect(Decimal.fromInt(2)->Decimal.gte(Decimal.fromInt(2)))->toBe(true)
  )
  test("lt: 1 < 2", () =>
    expect(Decimal.fromInt(1)->Decimal.lt(Decimal.fromInt(2)))->toBe(true)
  )
  test("lte: 2 <= 2", () =>
    expect(Decimal.fromInt(2)->Decimal.lte(Decimal.fromInt(2)))->toBe(true)
  )
  test("isZero: 0 is zero, 1 is not", () => {
    expect(Decimal.fromInt(0)->Decimal.isZero)->toBe(true)
    expect(Decimal.fromInt(1)->Decimal.isZero)->toBe(false)
  })
  test("isNeg: -1 is negative, 1 is not", () => {
    expect(Decimal.fromString("-1")->Decimal.isNeg)->toBe(true)
    expect(Decimal.fromInt(1)->Decimal.isNeg)->toBe(false)
  })
})

describe("Decimal — terminal accessors", () => {
  test("toFixed pads to the requested places: 1.5 at 3 -> \"1.500\"", () =>
    expect(Decimal.fromString("1.5")->Decimal.toFixed(3))->toBe("1.500")
  )
  test("toString gives the canonical form", () =>
    expect(Decimal.fromInt(42)->Decimal.toString)->toBe("42")
  )
  test("toNumber returns a JS float", () =>
    expect(Decimal.fromString("3.14")->Decimal.toNumber)->toBe(3.14)
  )
})
