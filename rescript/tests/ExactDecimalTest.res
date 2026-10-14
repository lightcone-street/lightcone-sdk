open RescriptBun.Test
open RescriptBun.Test.Expect

describe("Exact decimal boundaries", () => {
  test("accepts exponents and insignificant trailing zeroes", () => {
    expect(ExactDecimal.scaledInteger("+1.2500e2", 2))->toEqual(Ok(12500n))
    expect(ExactDecimal.scaledInteger("9.007199254740993e15", 0))->toEqual(Ok(9007199254740993n))
    expect(ExactDecimal.scaledInteger("0.0000000010", 9))->toEqual(Ok(1n))
  })
  test("rejects rounding and malformed decimal syntax", () => {
    ["1.0000000001", "-1", "NaN", "Infinity", "0x10", "1e2x", "1e10001"]->Array.forEach(
      text => expect(ExactDecimal.scaledInteger(text, 9)->Result.isError)->toBe(true),
    )
  })
  test("formats transaction range and wider display amounts exactly", () => {
    expect(ExactDecimal.formatScaled(18446744073709551615n, 9))->toEqual(
      Ok("18446744073.709551615"),
    )
    expect(ExactDecimal.formatScaled(1n, 9))->toEqual(Ok("0.000000001"))
  })
})
