open RescriptBun.Test
open RescriptBun.Test.Expect

describe("Exact protocol JSON", () => {
  test("request bytes preserve unsafe signed integers", () => {
    let json = JSON.Object(
      Dict.fromArray([
        ("amount_in", ExactJson.integer(9007199254740993n)),
        ("salt", ExactJson.integer(9223372036854775807n)),
        ("expiration", ExactJson.integer(-1n)),
      ]),
    )
    expect(ExactJson.stringify(json))->toBe(
      "{\"amount_in\":9007199254740993,\"salt\":9223372036854775807,\"expiration\":-1}",
    )
  })
  test("unsafe response integers are preserved before domain decoding", () => {
    switch ExactJson.parse("{\"seq\":9007199254740993,\"decimals\":6}") {
    | Ok(JSON.Object(fields)) =>
      expect(Dict.get(fields, "seq"))->toEqual(Some(JSON.String("9007199254740993")))
      expect(Dict.get(fields, "decimals"))->toEqual(Some(JSON.Number(6.0)))
    | _ => expect(false)->toBe(true)
    }
  })
  test("duplicate fields and malformed JSON fail", () => {
    expect(ExactJson.parse("{\"amount\":1,\"amount\":2}")->Result.isError)->toBe(true)
    expect(ExactJson.parse("{\"amount\":}")->Result.isError)->toBe(true)
  })
})
