open RescriptBun.Test
open RescriptBun.Test.Expect

let decimals66: Scaling.orderbookDecimals = {baseDecimals: 6, quoteDecimals: 6, priceDecimals: 2, tickSize: 0.0}

describe("Envelope.buildLimitOrder", () => {
  testAsync("builds + signs a bid into a SubmitOrderRequest (scaled amounts + hex sig)", async () => {
    let seed: Uint8Array.t = %raw(`new Uint8Array(32).fill(5)`)
    let keypair = await SolanaKitKeys.createKeyPairFromPrivateKeyBytes(seed)
    let maker = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)
    let mint = SolanaKit.address("So11111111111111111111111111111111111111112")

    switch await Envelope.buildLimitOrder(
      ~maker,
      ~market=maker,
      ~baseMint=mint,
      ~quoteMint=mint,
      ~side=0,
      ~price="0.65",
      ~size="100",
      ~decimals=decimals66,
      ~orderbookId="ob_1",
      ~keypair,
    ) {
    | Ok(request) =>
      expect(BigInt.toString(request.amountIn))->toBe("65000000") // bid spends quote
      expect(BigInt.toString(request.amountOut))->toBe("100000000") // bid receives base
      expect(request.side)->toBe(0)
      expect(String.length(request.signatureHex))->toBe(128) // 64-byte sig as hex
      expect(request.orderbookId)->toBe("ob_1")
    | Error(_) => expect("unexpected scaling error")->toBe("ok")
    }
  })
})

describe("Order cancel signing", () => {
  testAsync("cancelBodySigned produces a hex signature over the order hash", async () => {
    let seed: Uint8Array.t = %raw(`new Uint8Array(32).fill(9)`)
    let keypair = await SolanaKitKeys.createKeyPairFromPrivateKeyBytes(seed)
    let body = await Order.cancelBodySigned(~orderHash="deadbeef", ~maker="maker1", ~keypair)
    expect(String.length(body.signatureHex))->toBe(128)
    expect(body.orderHash)->toBe("deadbeef")
  })
})
