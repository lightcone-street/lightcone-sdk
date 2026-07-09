open RescriptBun.Test
open RescriptBun.Test.Expect

let byteLength: Uint8Array.t => int = %raw(`(a) => a.length`)
let byteAt: (Uint8Array.t, int) => int = %raw(`(a, i) => a[i]`)
let bigStr = value => BigInt.toString(value)

let decimals66: Scaling.OrderbookDecimals.t = {baseDecimals: 6, quoteDecimals: 6, priceDecimals: 2, tickSize: 0.0}
let decimals69: Scaling.OrderbookDecimals.t = {baseDecimals: 6, quoteDecimals: 9, priceDecimals: 2, tickSize: 0.0}

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

// ── Position instruction byte layouts ─────────────────────────────────────────
// Verifies the newly-ported builders against the Rust account counts, ordered
// account roles (0 RO, 1 W, 2 RO-signer, 3 W-signer), 1-byte opcodes, and
// little-endian data packing (1_000_000 = 0x0F4240 → LE 0x40 0x42 0x0F).
describe("Instructions (position ops)", () => {
  let programId = SolanaKit.address("9cCFQnmWqWmZF3LNdAVWTh7ECGJK4tCVPtgPMcYum81A")
  let user = SolanaKit.address("So11111111111111111111111111111111111111112")
  let market = SolanaKit.address("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
  let mint = SolanaKit.address("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")
  let lookupTable = SolanaKit.address("Vote111111111111111111111111111111111111111")

  let role = (instruction: SolanaKit.instruction, index) =>
    (instruction.accounts->Array.getUnsafe(index)).role
  let address = (instruction: SolanaKit.instruction, index) =>
    SolanaKit.addressToString((instruction.accounts->Array.getUnsafe(index)).address)

  testAsync("deposit (mint complete set): opcode 3 + amount LE; 11 + 2n accounts", async () => {
    let ix = await Instructions.deposit(~programId, ~user, ~market, ~mint, ~amount=1000000n, ~numOutcomes=2)
    expect(byteLength(ix.data))->toBe(9)
    expect(byteAt(ix.data, 0))->toBe(3)
    expect(byteAt(ix.data, 1))->toBe(0x40)
    expect(byteAt(ix.data, 2))->toBe(0x42)
    expect(byteAt(ix.data, 3))->toBe(0x0f)
    expect(Array.length(ix.accounts))->toBe(15)
    expect(role(ix, 0))->toBe(SolanaKit.Role.writableSigner)
    expect(address(ix, 0))->toBe(SolanaKit.addressToString(user))
    let (exchange, _) = await Pda.exchange(programId)
    expect(address(ix, 1))->toBe(SolanaKit.addressToString(exchange))
    let (position, _) = await Pda.position(programId, ~owner=user, ~market)
    expect(address(ix, 6))->toBe(SolanaKit.addressToString(position))
    expect(role(ix, 6))->toBe(SolanaKit.Role.writable)
  })

  testAsync("withdrawFromPosition: opcode 11 + amount LE + outcome u8; 8 accounts", async () => {
    let ix = await Instructions.withdrawFromPosition(
      ~programId,
      ~user,
      ~market,
      ~mint,
      ~amount=5n,
      ~outcomeIndex=1,
    )
    expect(byteLength(ix.data))->toBe(10)
    expect(byteAt(ix.data, 0))->toBe(11)
    expect(byteAt(ix.data, 1))->toBe(5)
    expect(byteAt(ix.data, 9))->toBe(1)
    expect(Array.length(ix.accounts))->toBe(8)
    expect(role(ix, 0))->toBe(SolanaKit.Role.writableSigner)
    let (position, _) = await Pda.position(programId, ~owner=user, ~market)
    expect(address(ix, 2))->toBe(SolanaKit.addressToString(position))
    expect(role(ix, 2))->toBe(SolanaKit.Role.writable)
    expect(address(ix, 3))->toBe(SolanaKit.addressToString(mint))
    let (exchange, _) = await Pda.exchange(programId)
    expect(address(ix, 7))->toBe(SolanaKit.addressToString(exchange))
  })

  testAsync("extendPositionTokens: opcode 21 + mint count; 10 + m*(3+2n) accounts", async () => {
    let ix = await Instructions.extendPositionTokens(
      ~programId,
      ~operator=user,
      ~user,
      ~market,
      ~lookupTable,
      ~depositMints=[mint, market],
      ~numOutcomes=2,
    )
    expect(byteLength(ix.data))->toBe(2)
    expect(byteAt(ix.data, 0))->toBe(21)
    expect(byteAt(ix.data, 1))->toBe(2)
    expect(Array.length(ix.accounts))->toBe(24)
    expect(role(ix, 0))->toBe(SolanaKit.Role.writableSigner)
    // position is READONLY here (unlike initPositionTokens, where it is writable).
    let (position, _) = await Pda.position(programId, ~owner=user, ~market)
    expect(address(ix, 4))->toBe(SolanaKit.addressToString(position))
    expect(role(ix, 4))->toBe(SolanaKit.Role.readonly)
    expect(address(ix, 5))->toBe(SolanaKit.addressToString(lookupTable))
    expect(role(ix, 5))->toBe(SolanaKit.Role.writable)
  })

  testAsync("closePositionAlt: opcode-only data; 6 accounts", async () => {
    let position = SolanaKit.address("Stake11111111111111111111111111111111111111")
    let ix = await Instructions.closePositionAlt(
      ~programId,
      ~operator=user,
      ~position,
      ~market,
      ~lookupTable,
    )
    expect(byteLength(ix.data))->toBe(1)
    expect(byteAt(ix.data, 0))->toBe(23)
    expect(Array.length(ix.accounts))->toBe(6)
    expect(role(ix, 0))->toBe(SolanaKit.Role.writableSigner)
    expect(address(ix, 2))->toBe(SolanaKit.addressToString(position))
    expect(address(ix, 4))->toBe(SolanaKit.addressToString(lookupTable))
    expect(role(ix, 4))->toBe(SolanaKit.Role.writable)
    expect(address(ix, 5))->toBe(SolanaKit.addressToString(Constants.altProgram))
  })

  testAsync("closePositionTokenAccounts: opcode-only data; 5 + m*(1+2n) accounts", async () => {
    let position = SolanaKit.address("Stake11111111111111111111111111111111111111")
    let ix = await Instructions.closePositionTokenAccounts(
      ~programId,
      ~operator=user,
      ~market,
      ~position,
      ~depositMints=[mint],
      ~numOutcomes=3,
    )
    expect(byteLength(ix.data))->toBe(1)
    expect(byteAt(ix.data, 0))->toBe(25)
    expect(Array.length(ix.accounts))->toBe(12)
    expect(role(ix, 0))->toBe(SolanaKit.Role.writableSigner)
    expect(address(ix, 3))->toBe(SolanaKit.addressToString(position))
    expect(address(ix, 5))->toBe(SolanaKit.addressToString(mint))
    // Per outcome: conditional mint readonly, position ATA writable.
    expect(role(ix, 6))->toBe(SolanaKit.Role.readonly)
    expect(role(ix, 7))->toBe(SolanaKit.Role.writable)
  })
})

// ── OrderPayload extended surface (serialize / verify / compact / math) ────────
describe("OrderPayload signed-order serialization + helpers", () => {
  testAsync("233-byte roundtrip preserves the payload and verifies", async () => {
    let seed: Uint8Array.t = %raw(`new Uint8Array(32).fill(11)`)
    let keypair = await SolanaKitKeys.createKeyPairFromPrivateKeyBytes(seed)
    let maker = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)
    let mint = SolanaKit.address("So11111111111111111111111111111111111111112")
    let order = OrderPayload.newBid(
      ~nonce=1n,
      ~salt=42n,
      ~maker,
      ~market=mint,
      ~baseMint=mint,
      ~quoteMint=mint,
      ~amountIn=65000000n,
      ~amountOut=100000000n,
    )
    let signature = await OrderPayload.sign(order, keypair)
    let bytes = OrderPayload.serialize(order, ~signature)
    expect(byteLength(bytes))->toBe(233)
    switch OrderPayload.deserialize(bytes) {
    | Ok((decoded, decodedSignature)) => {
        expect(decoded)->toEqual(order)
        expect(await OrderPayload.verifySignature(decoded, ~signature=decodedSignature))->toBe(true)
        // A different order must NOT verify against this signature.
        let tampered = {...decoded, amountIn: 1n}
        expect(await OrderPayload.verifySignature(tampered, ~signature=decodedSignature))->toBe(false)
      }
    | Error(error) => expect(error)->toBe("roundtrip")
    }
  })

  test("compact order: 37-byte roundtrip; toOrder truncates the nonce to u32", () => {
    let compact: OrderPayload.Compact.t = {
      nonce: 7,
      salt: 5n,
      side: 1,
      amountIn: 10n,
      amountOut: 20n,
      expiration: 0n,
    }
    let bytes = OrderPayload.Compact.serialize(compact)
    expect(byteLength(bytes))->toBe(37)
    expect(OrderPayload.Compact.deserialize(bytes))->toEqual(Ok(compact))

    let mint = SolanaKit.address("So11111111111111111111111111111111111111112")
    let payload = OrderPayload.newAsk(
      ~nonce=4294967303n, // 2^32 + 7
      ~salt=5n,
      ~maker=mint,
      ~market=mint,
      ~baseMint=mint,
      ~quoteMint=mint,
      ~amountIn=10n,
      ~amountOut=20n,
    )
    expect(OrderPayload.toOrder(payload).nonce)->toBe(7)
    // Compact + accounts → payload (nonce widens back losslessly for u32 values).
    let restored = OrderPayload.ofOrder(
      compact,
      ~maker=mint,
      ~market=mint,
      ~baseMint=mint,
      ~quoteMint=mint,
    )
    expect(BigInt.toString(restored.nonce))->toBe("7")
    expect(restored.side)->toBe(1)
  })

  test("ordersCanCross + calculateTakerFill", () => {
    let mint = SolanaKit.address("So11111111111111111111111111111111111111112")
    let bid = OrderPayload.newBid(
      ~nonce=1n,
      ~salt=1n,
      ~maker=mint,
      ~market=mint,
      ~baseMint=mint,
      ~quoteMint=mint,
      ~amountIn=65n, // pays 65 quote
      ~amountOut=100n, // for 100 base → price 0.65
    )
    let ask = OrderPayload.newAsk(
      ~nonce=1n,
      ~salt=1n,
      ~maker=mint,
      ~market=mint,
      ~baseMint=mint,
      ~quoteMint=mint,
      ~amountIn=100n, // sells 100 base
      ~amountOut=60n, // for 60 quote → price 0.60
    )
    expect(OrderPayload.ordersCanCross(~buyOrder=bid, ~sellOrder=ask))->toBe(true)
    // Seller asking more than the buyer pays does not cross.
    let greedyAsk = {...ask, amountOut: 70n}
    expect(OrderPayload.ordersCanCross(~buyOrder=bid, ~sellOrder=greedyAsk))->toBe(false)
    // Wrong sides never cross.
    expect(OrderPayload.ordersCanCross(~buyOrder=ask, ~sellOrder=bid))->toBe(false)

    switch OrderPayload.calculateTakerFill(ask, ~makerFillAmount=50n) {
    | Ok(fill) => expect(BigInt.toString(fill))->toBe("30") // 50 * 60 / 100
    | Error(error) => expect(error)->toBe("ok")
    }
    expect(
      OrderPayload.calculateTakerFill({...ask, amountIn: 0n}, ~makerFillAmount=50n)->Result.isError,
    )->toBe(true)
  })

  test("deriveConditionId is a 32-byte keccak over oracle ‖ question ‖ outcomes", () => {
    let oracle = SolanaKit.address("So11111111111111111111111111111111111111112")
    let questionId: Uint8Array.t = %raw(`new Uint8Array(32).fill(1)`)
    let conditionId = OrderPayload.deriveConditionId(~oracle, ~questionId, ~numOutcomes=2)
    expect(byteLength(conditionId))->toBe(32)
    // Changing any input changes the id.
    let other = OrderPayload.deriveConditionId(~oracle, ~questionId, ~numOutcomes=3)
    expect(conditionId == other)->toBe(false)
  })

  test("isOrderExpired + deriveOrderbookId + signatureFromBs58", () => {
    let mint = SolanaKit.address("So11111111111111111111111111111111111111112")
    let order = OrderPayload.newBid(
      ~nonce=1n,
      ~salt=1n,
      ~maker=mint,
      ~market=mint,
      ~baseMint=mint,
      ~quoteMint=mint,
      ~amountIn=1n,
      ~amountOut=1n,
      ~expiration=100n,
    )
    expect(OrderPayload.isOrderExpired(order, ~currentTime=99n))->toBe(false)
    expect(OrderPayload.isOrderExpired(order, ~currentTime=100n))->toBe(true)
    expect(OrderPayload.isOrderExpired({...order, expiration: 0n}, ~currentTime=100n))->toBe(false)

    expect(OrderPayload.deriveOrderbookId(order))->toBe("So111111_So111111")
    expect(OrderPayload.signatureFromBs58("not-base58!")->Result.isError)->toBe(true)
  })
})

// ── Admin / matching instruction layouts ───────────────────────────────────────
describe("Instructions (admin + matching ops)", () => {
  let programId = SolanaKit.address("9cCFQnmWqWmZF3LNdAVWTh7ECGJK4tCVPtgPMcYum81A")
  let authority = SolanaKit.address("So11111111111111111111111111111111111111112")
  let market = SolanaKit.address("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
  let mint = SolanaKit.address("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")

  testAsync("createMarket: opcode 1 + 70-byte data (outcomes, oracle, question, fees)", async () => {
    let questionId: Uint8Array.t = %raw(`new Uint8Array(32).fill(2)`)
    switch await Instructions.createMarket(
      ~programId,
      ~manager=authority,
      ~marketId=1n,
      ~numOutcomes=2,
      ~oracle=mint,
      ~questionId,
      ~makerFeeBps=-25,
      ~takerFeeBps=75,
    ) {
    | Ok(ix) => {
        expect(byteLength(ix.data))->toBe(70)
        expect(byteAt(ix.data, 0))->toBe(1)
        expect(byteAt(ix.data, 1))->toBe(2) // num_outcomes
        // maker fee -25 → two's-complement LE (0xE7, 0xFF)
        expect(byteAt(ix.data, 66))->toBe(0xe7)
        expect(byteAt(ix.data, 67))->toBe(0xff)
        expect(byteAt(ix.data, 68))->toBe(75)
        expect(Array.length(ix.accounts))->toBe(5)
      }
    | Error(_) => expect("error")->toBe("ok")
    }
    // Fee pair summing negative is rejected.
    expect(
      (
        await Instructions.createMarket(
          ~programId,
          ~manager=authority,
          ~marketId=1n,
          ~numOutcomes=2,
          ~oracle=mint,
          ~questionId,
          ~makerFeeBps=-100,
          ~takerFeeBps=50,
        )
      )->Result.isError,
    )->toBe(true)
  })

  testAsync("settleMarket: opcode 7 + u32 numerators; winnerTakesAll helper", async () => {
    switch Instructions.winnerTakesAllNumerators(~winningOutcome=1, ~numOutcomes=3) {
    | Ok(numerators) => {
        expect(numerators)->toEqual([0, 1, 0])
        switch await Instructions.settleMarket(
          ~programId,
          ~oracle=authority,
          ~marketId=1n,
          ~payoutNumerators=numerators,
        ) {
        | Ok(ix) => {
            expect(byteLength(ix.data))->toBe(13) // 1 + 3 * 4
            expect(byteAt(ix.data, 0))->toBe(7)
            expect(byteAt(ix.data, 5))->toBe(1) // second numerator LE
            // oracle is a READONLY signer
            expect((ix.accounts->Array.getUnsafe(0)).role)->toBe(SolanaKit.Role.readonlySigner)
          }
        | Error(_) => expect("error")->toBe("ok")
        }
      }
    | Error(_) => expect("error")->toBe("ok")
    }
    expect(
      (
        await Instructions.settleMarket(
          ~programId,
          ~oracle=authority,
          ~marketId=1n,
          ~payoutNumerators=[0, 0],
        )
      )->Result.isError,
    )->toBe(true)
  })

  testAsync("matchOrdersMulti: data = 1 + 103 + 117/maker; bitmask trims order-status accounts", async () => {
    let signature: Uint8Array.t = %raw(`new Uint8Array(64).fill(3)`)
    let payload = OrderPayload.newBid(
      ~nonce=1n,
      ~salt=1n,
      ~maker=authority,
      ~market,
      ~baseMint=mint,
      ~quoteMint=mint,
      ~amountIn=10n,
      ~amountOut=20n,
    )
    let taker: Instructions.SignedOrder.t = {order: payload, signature}
    let maker: Instructions.MatchMaker.t = {
      order: {order: {...payload, side: 1}, signature},
      makerFillAmount: 5n,
      takerFillAmount: 10n,
      isFullFill: true,
    }
    switch await Instructions.matchOrdersMulti(
      ~programId,
      ~operator=authority,
      ~market,
      ~baseMint=mint,
      ~quoteMint=mint,
      ~feeReceiver=authority,
      ~takerOrder=taker,
      ~takerIsFullFill=true,
      ~makers=[maker],
    ) {
    | Ok(ix) => {
        expect(byteLength(ix.data))->toBe(1 + 37 + 64 + 2 + 37 + 64 + 16)
        expect(byteAt(ix.data, 0))->toBe(13)
        expect(byteAt(ix.data, 102))->toBe(1) // num_makers
        expect(byteAt(ix.data, 103))->toBe(129) // bitmask: taker bit 7 + maker bit 0
        // Full fills: 15 fixed taker accounts + 4 per maker, no order-status.
        expect(Array.length(ix.accounts))->toBe(19)
      }
    | Error(_) => expect("error")->toBe("ok")
    }
    expect(
      (
        await Instructions.matchOrdersMulti(
          ~programId,
          ~operator=authority,
          ~market,
          ~baseMint=mint,
          ~quoteMint=mint,
          ~feeReceiver=authority,
          ~takerOrder=taker,
          ~takerIsFullFill=false,
          ~makers=[],
        )
      )->Result.isError,
    )->toBe(true)
  })

  testAsync("depositToGlobalWithAlt (Create): base accounts + ALT block; slot appended", async () => {
    let ix = await Instructions.depositToGlobalWithAlt(
      ~programId,
      ~user=authority,
      ~mint,
      ~amount=1000000n,
      ~altContext=Instructions.DepositToGlobalAltContext.Create({recentSlot: 5n}),
    )
    expect(Array.length(ix.accounts))->toBe(11) // 8 base + nonce + lookup table + alt program
    expect(byteLength(ix.data))->toBe(17) // opcode + amount + recent slot
    expect(byteAt(ix.data, 0))->toBe(17)
    expect(byteAt(ix.data, 9))->toBe(5) // recent_slot LE low byte
    let extended = await Instructions.depositToGlobalWithAlt(
      ~programId,
      ~user=authority,
      ~mint,
      ~amount=1000000n,
      ~altContext=Instructions.DepositToGlobalAltContext.Extend({lookupTable: market}),
    )
    expect(byteLength(extended.data))->toBe(9) // no slot appended
    expect(
      SolanaKit.addressToString((extended.accounts->Array.getUnsafe(9)).address),
    )->toBe(SolanaKit.addressToString(market))
  })

  testAsync("closeOrderStatus: opcode 24 + 32-byte hash; 3 accounts", async () => {
    let orderHash: Uint8Array.t = %raw(`new Uint8Array(32).fill(9)`)
    let ix = await Instructions.closeOrderStatus(~programId, ~operator=authority, ~orderHash)
    expect(byteLength(ix.data))->toBe(33)
    expect(byteAt(ix.data, 0))->toBe(24)
    expect(Array.length(ix.accounts))->toBe(3)
  })

  testAsync("conditional metadata: validates lengths; update drops system+rent", async () => {
    switch await Instructions.createConditionalMetadata(
      ~programId,
      ~manager=authority,
      ~market,
      ~depositMint=mint,
      ~outcomeIndex=0,
      ~name="Yes",
      ~symbol="YES",
      ~uri="https://example.com/yes.json",
    ) {
    | Ok(ix) => {
        expect(byteAt(ix.data, 0))->toBe(31)
        expect(Array.length(ix.accounts))->toBe(10)
      }
    | Error(_) => expect("error")->toBe("ok")
    }
    switch await Instructions.updateConditionalMetadata(
      ~programId,
      ~manager=authority,
      ~market,
      ~depositMint=mint,
      ~outcomeIndex=0,
      ~name="Yes",
      ~symbol="YES",
      ~uri="https://example.com/yes.json",
    ) {
    | Ok(ix) => {
        expect(byteAt(ix.data, 0))->toBe(32)
        expect(Array.length(ix.accounts))->toBe(8)
        expect((ix.accounts->Array.getUnsafe(0)).role)->toBe(SolanaKit.Role.readonlySigner)
      }
    | Error(_) => expect("error")->toBe("ok")
    }
    expect(
      (
        await Instructions.createConditionalMetadata(
          ~programId,
          ~manager=authority,
          ~market,
          ~depositMint=mint,
          ~outcomeIndex=0,
          ~name="Yes",
          ~symbol="WAY-TOO-LONG-SYMBOL",
          ~uri="https://example.com/yes.json",
        )
      )->Result.isError,
    )->toBe(true)
  })
})
