open RescriptBun.Test
open RescriptBun.Test.Expect

// Runtime tests for the web-crypto binding — run the compiled .res.mjs under Bun to prove
// `crypto.randomUUID` is reachable and returns a fresh UUID string each call.

describe("WebCrypto", () => {
  test("randomUUID returns a 36-char UUID string", () =>
    expect(String.length(WebCrypto.randomUUID()))->toBe(36)
  )
  test("two randomUUIDs differ", () =>
    expect(WebCrypto.randomUUID() == WebCrypto.randomUUID())->toBe(false)
  )
})
