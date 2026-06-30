open RescriptBun.Test
open RescriptBun.Test.Expect

let byteLength: Uint8Array.t => int = %raw(`(a) => a.length`)
let byteAt: (Uint8Array.t, int) => int = %raw(`(a, i) => a[i]`)
let bigStr = value => BigInt.toString(value)

let decimals66: Scaling.orderbookDecimals = {baseDecimals: 6, quoteDecimals: 6, priceDecimals: 2, tickSize: 0.0}
let decimals69: Scaling.orderbookDecimals = {baseDecimals: 6, quoteDecimals: 9, priceDecimals: 2, tickSize: 0.0}

// Vectors copied verbatim from rust/src/shared/scaling.rs tests.
describe("Scaling", () => {
  test("BID basic: 0.65 * 100 @ 6/6", () => {
    switch Scaling.scalePriceSize(~price="0.65", ~size="100", ~side=0, ~decimals=decimals66) {
    | Ok({amountIn, amountOut}) =>
      expect(bigStr(amountIn))->toBe("65000000")
      expect(bigStr(amountOut))->toBe("100000000")
    | Error(_) => expect("error")->toBe("ok")
    }
  })

  test("ASK basic swaps amount_in/out", () => {
    switch Scaling.scalePriceSize(~price="0.65", ~size="100", ~side=1, ~decimals=decimals66) {
    | Ok({amountIn, amountOut}) =>
      expect(bigStr(amountIn))->toBe("100000000")
      expect(bigStr(amountOut))->toBe("65000000")
    | Error(_) => expect("error")->toBe("ok")
    }
  })

  test("different decimals 6/9", () => {
    switch Scaling.scalePriceSize(~price="0.65", ~size="100", ~side=0, ~decimals=decimals69) {
    | Ok({amountIn, amountOut}) =>
      expect(bigStr(amountIn))->toBe("65000000000")
      expect(bigStr(amountOut))->toBe("100000000")
    | Error(_) => expect("error")->toBe("ok")
    }
  })

  test("f64 noise in size is truncated", () => {
    switch Scaling.scalePriceSize(~price="1", ~size="15.763000000000002", ~side=0, ~decimals=decimals66) {
    | Ok({amountIn, amountOut}) =>
      expect(bigStr(amountIn))->toBe("15763000")
      expect(bigStr(amountOut))->toBe("15763000")
    | Error(_) => expect("error")->toBe("ok")
    }
  })

  test("zero price rejected", () =>
    expect(
      Scaling.scalePriceSize(~price="0", ~size="100", ~side=0, ~decimals=decimals66)->Result.isError,
    )->toBe(true)
  )

  test("sub-lamport size → ZeroAmount", () =>
    expect(
      Scaling.scalePriceSize(~price="1", ~size="0.0000001", ~side=0, ~decimals=decimals66)->Result.isError,
    )->toBe(true)
  )
})

describe("OrderPayload", () => {
  testAsync("169-byte LE message + keccak digest + sign/verify roundtrip", async () => {
    let seed: Uint8Array.t = %raw(`new Uint8Array(32).fill(3)`)
    let keypair = await SolanaKitKeys.createKeyPairFromPrivateKeyBytes(seed)
    let maker = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)
    let order: OrderPayload.t = {
      nonce: 1n,
      salt: 0n,
      maker,
      market: maker,
      baseMint: maker,
      quoteMint: maker,
      side: 0,
      amountIn: 65000000n,
      amountOut: 100000000n,
      expiration: 0n,
    }
    let message = OrderPayload.signingMessage(order)
    expect(byteLength(message))->toBe(169)
    expect(byteAt(message, 0))->toBe(1) // nonce LE low byte
    expect(byteAt(message, 144))->toBe(0) // side = Bid

    let digest = OrderPayload.hash(order)
    expect(byteLength(digest))->toBe(32)

    let signature = await OrderPayload.sign(order, keypair)
    expect(byteLength(signature))->toBe(64)
    let hexBytes = SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), OrderPayload.hashHex(order))
    let verified = await SolanaKitKeys.verifySignature(keypair.publicKey, signature, hexBytes)
    expect(verified)->toBe(true)
  })
})

describe("Pda", () => {
  testAsync("exchange derives; orderbook canonicalizes mint order", async () => {
    let programId = SolanaKit.address("9cCFQnmWqWmZF3LNdAVWTh7ECGJK4tCVPtgPMcYum81A")
    let (exchange, bump) = await Pda.exchange(programId)
    expect(SolanaKit.addressToString(exchange)->String.length > 30 && bump >= 0)->toBe(true)

    let mintA = SolanaKit.address("So11111111111111111111111111111111111111112")
    let mintB = SolanaKit.address("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
    let (forward, _) = await Pda.orderbook(programId, ~mintA, ~mintB)
    let (reverse, _) = await Pda.orderbook(programId, ~mintA=mintB, ~mintB=mintA)
    expect(SolanaKit.addressToString(forward))->toBe(SolanaKit.addressToString(reverse))
  })
})
