open RescriptBun.Test
open RescriptBun.Test.Expect

// Runtime tests for the @solana/kit binding — run the compiled .res.mjs under Bun.
// Self-contained within the package (references only SolanaKit.* sub-modules).
let utf8 = text => SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), text)
let toHex = bytes => SolanaKitCodec.decode(SolanaKitCodec.getBase16Decoder(), bytes)
let byteLength: Uint8Array.t => int = %raw(`(bytes) => bytes.length`)
let seed: Uint8Array.t = %raw(`new Uint8Array(32).fill(7)`)

describe("SolanaKit (Address)", () => {
  test("address() round-trips through addressToString", () => {
    let a = SolanaKit.address("So11111111111111111111111111111111111111112")
    expect(SolanaKit.addressToString(a))->toBe("So11111111111111111111111111111111111111112")
  })
  test("isAddress validates a base58 address", () =>
    expect(SolanaKit.isAddress("So11111111111111111111111111111111111111112"))->toBe(true)
  )
})

describe("SolanaKit.Codec", () => {
  test("u64 encoder produces 8 little-endian bytes", () => {
    let bytes = SolanaKitCodec.encode(SolanaKitCodec.getU64Encoder(), 1n)
    expect(byteLength(bytes))->toBe(8)
  })
  test("base16 decoder yields lowercase hex bytes->string", () =>
    expect("abc"->utf8->toHex->String.length)->toBe(6)
  )
})

describe("SolanaKit.Keys", () => {
  testAsync("ed25519 sign/verify roundtrip over message bytes", async () => {
    let keypair = await SolanaKitKeys.createKeyPairFromPrivateKeyBytes(seed)
    let message = utf8("lightcone")
    let signature = await SolanaKitKeys.signBytes(keypair.privateKey, message)
    expect(byteLength(signature))->toBe(64)
    let verified = await SolanaKitKeys.verifySignature(keypair.publicKey, signature, message)
    expect(verified)->toBe(true)
  })
  testAsync("getAddressFromPublicKey returns a base58 address", async () => {
    let keypair = await SolanaKitKeys.createKeyPairFromPrivateKeyBytes(seed)
    let address = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)
    expect(SolanaKit.addressToString(address)->String.length > 30)->toBe(true)
  })
})

describe("SolanaKit.Pda", () => {
  testAsync("getProgramDerivedAddress returns (address, bump)", async () => {
    let programId = SolanaKit.address("11111111111111111111111111111112")
    let (pda, bump) = await SolanaKitPda.getProgramDerivedAddress({
      programAddress: programId,
      seeds: [utf8("central_state")],
    })
    expect(SolanaKit.addressToString(pda)->String.length > 30 && bump >= 0 && bump <= 255)->toBe(true)
  })
})
