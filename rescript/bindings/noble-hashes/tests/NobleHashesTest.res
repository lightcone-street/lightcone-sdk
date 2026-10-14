open RescriptBun.Test
open RescriptBun.Test.Expect

// Runtime tests for the @noble/hashes binding — run the compiled .res.mjs under Bun
// to prove the JS export name (`keccak_256`), arg shape, and 32-byte return. The
// helpers are self-contained (no sibling-binding dependency).
let utf8: string => Uint8Array.t = %raw(`(text) => new TextEncoder().encode(text)`)
let toHex: Uint8Array.t => string = %raw(`(bytes) => Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("")`)
let byteLength: Uint8Array.t => int = %raw(`(bytes) => bytes.length`)
let keccakAbc = "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45"

describe("NobleHashes", () => {
  test("keccak256(\"abc\") matches the canonical vector (lowercase hex)", () =>
    expect("abc"->utf8->NobleHashes.keccak256->toHex)->toBe(keccakAbc)
  )

  test("keccak256 returns a 32-byte digest", () =>
    expect("hello world"->utf8->NobleHashes.keccak256->byteLength)->toBe(32)
  )

  test("keccak256 is deterministic", () => {
    let once = "lightcone"->utf8->NobleHashes.keccak256->toHex
    let twice = "lightcone"->utf8->NobleHashes.keccak256->toHex
    expect(once)->toBe(twice)
  })
})
