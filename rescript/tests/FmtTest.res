open RescriptBun.Test
open RescriptBun.Test.Expect

// Vectors copied verbatim from rust/src/shared/fmt/{decimal,num,str}.rs and
// rust/src/shared/price.rs tests.

let display = (value: string): string => Fmt.Decimal.display(Decimal.fromString(value))
let abbr = (value: string): string => Fmt.Decimal.abbrNumber(Decimal.fromString(value))
let pct = (value: string): string => Fmt.Decimal.displayPct(Decimal.fromString(value))
let pctBare = (value: string): string =>
  Fmt.Decimal.displayPct(Decimal.fromString(value), ~padding=false)
let baseUnits = (value: string, decimals): option<string> =>
  Fmt.Decimal.toBaseUnits(Decimal.fromString(value), ~decimals)->Option.map(units =>
    BigInt.toString(units)
  )

describe("Fmt.Decimal.display", () => {
  test("zero displays as plain 0", () => expect(display("0"))->toBe("0"))

  test("tiered decimal places by magnitude", () => {
    expect(display("12345.67"))->toBe("12,346")
    expect(display("1234.56"))->toBe("1,234.6")
    expect(display("123.456"))->toBe("123.46")
    expect(display("15.4567"))->toBe("15.457")
    expect(display("1.23456"))->toBe("1.2346")
    expect(display("0.123456"))->toBe("0.1235")
    expect(display("0.012345"))->toBe("0.01235")
  })

  test("tier boundaries (rounding can promote a tier's formatting)", () => {
    expect(display("10000"))->toBe("10,000")
    expect(display("9999.99"))->toBe("10,000.0")
    expect(display("1000"))->toBe("1,000.0")
    expect(display("999.999"))->toBe("1,000.00")
    expect(display("100"))->toBe("100.00")
    expect(display("99.9999"))->toBe("100.000")
    expect(display("10"))->toBe("10.000")
    expect(display("9.87654"))->toBe("9.8765")
    expect(display("1"))->toBe("1.0000")
    expect(display("0.999999"))->toBe("1.0000")
    expect(display("0.1"))->toBe("0.1000")
    expect(display("0.099999"))->toBe("0.10000")
  })

  test("small values cap at five decimals; below that rounds to 0", () => {
    expect(display("0.01"))->toBe("0.01000")
    expect(display("0.00003"))->toBe("0.00003")
    expect(display("0.000004"))->toBe("0")
    expect(display("0.000000001"))->toBe("0")
  })

  test("negative values keep the sign unless they round to zero", () => {
    expect(display("-1234.56"))->toBe("-1,234.6")
    expect(display("-15.4567"))->toBe("-15.457")
    expect(display("-0.00003"))->toBe("-0.00003")
    expect(display("-0.000004"))->toBe("0")
  })
})

describe("Fmt.Decimal.abbrNumber", () => {
  test("below a thousand stays plain", () => {
    expect(abbr("0"))->toBe("0.00")
    expect(abbr("1"))->toBe("1.00")
    expect(abbr("999"))->toBe("999.00")
  })

  test("thousands / millions get suffixes (ties to even)", () => {
    expect(abbr("1000"))->toBe("1.00k")
    expect(abbr("1500"))->toBe("1.50k")
    expect(abbr("12345"))->toBe("12.34k")
    expect(abbr("1000000"))->toBe("1.00m")
    expect(abbr("1500000"))->toBe("1.50m")
    expect(abbr("1000000000"))->toBe("1.00b")
    expect(abbr("1000000000000"))->toBe("1.00t")
  })

  test("negative sign is dropped with ~showSign=false", () => {
    expect(abbr("-1500000"))->toBe("-1.50m")
    expect(Fmt.Decimal.abbrNumber(Decimal.fromString("-1500000"), ~showSign=false))->toBe("1.50m")
  })
})

describe("Fmt.Decimal.displayPct", () => {
  test("truncates to two decimal places (never rounds up)", () => {
    expect(pct("12.345"))->toBe("12.34")
    expect(pct("12.999"))->toBe("12.99")
    expect(pct("99.999"))->toBe("99.99")
  })

  test("padding pads to two places; ~padding=false trims", () => {
    expect(pct("12.3"))->toBe("12.30")
    expect(pct("12"))->toBe("12.00")
    expect(pct("0"))->toBe("0.00")
    expect(pctBare("12.345"))->toBe("12.34")
    expect(pctBare("12.3"))->toBe("12.3")
    expect(pctBare("12"))->toBe("12")
    expect(pctBare("0"))->toBe("0")
  })

  test("negative values keep the sign", () => {
    expect(pct("-3.456"))->toBe("-3.45")
    expect(pct("-3.4"))->toBe("-3.40")
    expect(pctBare("-3.456"))->toBe("-3.45")
  })
})

describe("Fmt.Decimal.toBaseUnits", () => {
  test("scales by 10^decimals (USDC = 6)", () => {
    expect(baseUnits("0", 6))->toBe(Some("0"))
    expect(baseUnits("1", 6))->toBe(Some("1000000"))
    expect(baseUnits("10.5", 6))->toBe(Some("10500000"))
    expect(baseUnits("0.000001", 6))->toBe(Some("1"))
  })

  test("negative values are None", () => expect(baseUnits("-1", 6))->toBe(None))
})

describe("Fmt.Num", () => {
  test("displayFormattedString groups thousands, preserving sign + fraction", () => {
    expect(Fmt.Num.displayFormattedString("0"))->toBe("0")
    expect(Fmt.Num.displayFormattedString("123"))->toBe("123")
    expect(Fmt.Num.displayFormattedString("1000"))->toBe("1,000")
    expect(Fmt.Num.displayFormattedString("1234567890"))->toBe("1,234,567,890")
    expect(Fmt.Num.displayFormattedString("1.500"))->toBe("1.500")
    expect(Fmt.Num.displayFormattedString("1000.00"))->toBe("1,000.00")
    expect(Fmt.Num.displayFormattedString("-1234.56"))->toBe("-1,234.56")
  })

  test("display uses the shared magnitude tiers", () => {
    expect(Fmt.Num.display(0.0))->toBe("0")
    expect(Fmt.Num.display(12345.67))->toBe("12,346")
    expect(Fmt.Num.display(1234.56))->toBe("1,234.6")
    expect(Fmt.Num.display(123.456))->toBe("123.46")
    expect(Fmt.Num.display(15.4567))->toBe("15.457")
    expect(Fmt.Num.display(1.23456))->toBe("1.2346")
    expect(Fmt.Num.display(0.123456))->toBe("0.1235")
    expect(Fmt.Num.display(0.000004))->toBe("0")
    expect(Fmt.Num.display(-1234.56))->toBe("-1,234.6")
    expect(Fmt.Num.display(-0.000004))->toBe("0")
  })

  test("displayWithDecimals is explicit (no zero-collapse)", () => {
    expect(Fmt.Num.displayWithDecimals(1.0, ~decimals=0))->toBe("1")
    expect(Fmt.Num.displayWithDecimals(1.0, ~decimals=2))->toBe("1.00")
    expect(Fmt.Num.displayWithDecimals(1.5, ~decimals=2))->toBe("1.50")
    expect(Fmt.Num.displayWithDecimals(1.234, ~decimals=2))->toBe("1.23")
    expect(Fmt.Num.displayWithDecimals(1234567.89, ~decimals=2))->toBe("1,234,567.89")
    expect(Fmt.Num.displayWithDecimals(1234567.0, ~decimals=0))->toBe("1,234,567")
  })

  test("displayPct truncates; ~padding=false trims", () => {
    expect(Fmt.Num.displayPct(12.345))->toBe("12.34")
    expect(Fmt.Num.displayPct(12.3))->toBe("12.30")
    expect(Fmt.Num.displayPct(12.0, ~padding=false))->toBe("12")
    expect(Fmt.Num.displayPct(12.3, ~padding=false))->toBe("12.3")
  })

  test("base-unit conversions roundtrip", () => {
    expect(Fmt.Num.toDecimalValue(1500000000n, ~decimals=9))->toBe(1.5)
    expect(Fmt.Num.toDecimalValue(1000000n, ~decimals=6))->toBe(1.0)
    expect(BigInt.toString(Fmt.Num.fromDecimalValue(1.5, ~decimals=9)))->toBe("1500000000")
    expect(BigInt.toString(Fmt.Num.fromDecimalValue(10.5, ~decimals=6)))->toBe("10500000")
  })
})

describe("Fmt.Str.shorten", () => {
  test("long strings shorten to head...tail", () =>
    expect(Fmt.Str.shorten("FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR", ~qty=8))->toBe(
      "FRGk...WcPR",
    )
  )

  test("strings of qty or fewer characters are unchanged", () =>
    expect(Fmt.Str.shorten("FRGkJho6", ~qty=8))->toBe("FRGkJho6")
  )
})

describe("Price", () => {
  test("parseDecimal is strict", () => {
    expect(Price.parseDecimal("0.500000"))->toBe(Some(0.5))
    expect(Price.parseDecimal("1.000000"))->toBe(Some(1.0))
    expect(Price.parseDecimal("0.123456"))->toBe(Some(0.123456))
    expect(Price.parseDecimal("1.5abc"))->toBe(None)
    expect(Price.parseDecimal(""))->toBe(None)
  })

  test("formatDecimal pads to precision and roundtrips", () => {
    expect(Price.formatDecimal(0.5, ~precision=6))->toBe("0.500000")
    expect(Price.formatDecimal(1.0, ~precision=6))->toBe("1.000000")
    expect(Price.formatDecimal(0.0, ~precision=6))->toBe("0.000000")
    let parsed = Price.parseDecimal("0.750000")->Option.getOr(0.0)
    expect(Price.formatDecimal(parsed, ~precision=6))->toBe("0.750000")
  })
})
