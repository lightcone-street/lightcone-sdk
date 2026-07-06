open RescriptBun.Test
open RescriptBun.Test.Expect

let decimals66: Scaling.OrderbookDecimals.t = {baseDecimals: 6, quoteDecimals: 6, priceDecimals: 2, tickSize: 0.0}

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

describe("Envelope.buildTriggerOrder", () => {
  testAsync("carries the trigger fields on the signed request", async () => {
    let seed: Uint8Array.t = %raw(`new Uint8Array(32).fill(7)`)
    let keypair = await SolanaKitKeys.createKeyPairFromPrivateKeyBytes(seed)
    let maker = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)
    let mint = SolanaKit.address("So11111111111111111111111111111111111111112")

    switch await Envelope.buildTriggerOrder(
      ~maker,
      ~market=maker,
      ~baseMint=mint,
      ~quoteMint=mint,
      ~side=1,
      ~price="0.55",
      ~size="100",
      ~decimals=decimals66,
      ~orderbookId="ob_1",
      ~keypair,
      ~triggerPrice=0.75,
      ~triggerType=Shared.TriggerType.TakeProfit,
      ~timeInForce=Shared.TimeInForce.Gtc,
    ) {
    | Ok(request) =>
      expect(request.triggerPrice)->toBe(Some(0.75))
      expect(request.triggerType == Some(Shared.TriggerType.TakeProfit))->toBe(true)
      expect(request.side)->toBe(1)
      expect(BigInt.toString(request.amountIn))->toBe("100000000") // ask spends base
      expect(String.length(request.signatureHex))->toBe(128)
    | Error(_) => expect("unexpected scaling error")->toBe("ok")
    }
  })
})

describe("Order cancel signing", () => {
  testAsync("cancelBodySigned produces a hex signature over the order hash", async () => {
    let seed: Uint8Array.t = %raw(`new Uint8Array(32).fill(9)`)
    let keypair = await SolanaKitKeys.createKeyPairFromPrivateKeyBytes(seed)
    let body = await Order.Client.cancelBodySigned(~orderHash="deadbeef", ~maker="maker1", ~keypair)
    expect(String.length(body.signatureHex))->toBe(128)
    expect(body.orderHash)->toBe("deadbeef")
  })

  testAsync("cancelTriggerBodySigned signs the trigger order id", async () => {
    let seed: Uint8Array.t = %raw(`new Uint8Array(32).fill(9)`)
    let keypair = await SolanaKitKeys.createKeyPairFromPrivateKeyBytes(seed)
    let body = await Order.Client.cancelTriggerBodySigned(
      ~triggerOrderId="trigger-order-uuid-123",
      ~maker="maker1",
      ~keypair,
    )
    expect(String.length(body.signatureHex))->toBe(128)
    expect(body.triggerOrderId)->toBe("trigger-order-uuid-123")
    // The signed message is the raw trigger-order-id bytes.
    let hexToBytes: string => Uint8Array.t = %raw(`(hex) => Uint8Array.from(hex.match(/.{2}/g).map((b) => parseInt(b, 16)))`)
    let message = SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), "trigger-order-uuid-123")
    let verified = await SolanaKitKeys.verifySignature(
      keypair.publicKey,
      hexToBytes(body.signatureHex),
      message,
    )
    expect(verified)->toBe(true)
  })
})

// ── Wire decoding: user snapshot orders, order events, snapshot frames ─────────
let parseJson: string => JSON.t = %raw("JSON.parse")

describe("Order.Raw.SnapshotOrder decode", () => {
  test("limit order: tx_signature kept, status defaults to Open, decimals default to 0", () => {
    let json = parseJson(`{
      "order_type": "limit",
      "order_hash": "hash1",
      "market_pubkey": "mkt1",
      "orderbook_id": "ob1",
      "side": "bid",
      "amount_in": "65",
      "amount_out": "100",
      "remaining": "10",
      "filled": "2",
      "created_at": 1700000000000,
      "base_mint": "base",
      "quote_mint": "quote",
      "outcome_index": 0,
      "tx_signature": "sig1"
    }`)
    switch Order.Raw.SnapshotOrder.t_decode(json) {
    | Ok(Limit(order)) =>
      expect(order.common.orderHash)->toBe("hash1")
      expect(order.common.status == Shared.OrderStatus.Open)->toBe(true)
      expect(order.txSignature)->toBe(Some("sig1"))
      expect(order.common.remaining)->toBe("10")
      expect(order.common.price)->toBe("0") // absent → "0"
      expect(order.common.expiration)->toBe(0.0)
    | Ok(Trigger(_)) => expect("trigger")->toBe("limit")
    | Error(error) => expect(error.message)->toBe("decoded")
    }
  })

  test("trigger order: maker/taker_amount aliases, numeric TIF, explicit status", () => {
    let json = parseJson(`{
      "order_type": "trigger",
      "order_hash": "hash2",
      "market_pubkey": "mkt1",
      "orderbook_id": "ob1",
      "side": "ask",
      "maker_amount": "1000",
      "taker_amount": "500",
      "created_at": 1700000000000,
      "base_mint": "base",
      "quote_mint": "quote",
      "outcome_index": 1,
      "status": "PENDING",
      "trigger_order_id": "trig-1",
      "trigger_price": "0.55",
      "trigger_type": "TP",
      "time_in_force": 3
    }`)
    switch Order.Raw.SnapshotOrder.t_decode(json) {
    | Ok(Trigger(order)) =>
      expect(order.common.amountIn)->toBe("1000")
      expect(order.common.amountOut)->toBe("500")
      expect(order.triggerOrderId)->toBe("trig-1")
      expect(order.triggerType == Shared.TriggerType.TakeProfit)->toBe(true)
      expect(order.timeInForce == Some(Shared.TimeInForce.Alo))->toBe(true)
      expect(order.common.status == Shared.OrderStatus.Pending)->toBe(true)
    | Ok(Limit(_)) => expect("limit")->toBe("trigger")
    | Error(error) => expect(error.message)->toBe("decoded")
    }
  })

  test("encode → decode roundtrip preserves both variants", () => {
    let common: Order.Raw.SnapshotCommon.t = {
      orderHash: "h",
      marketPubkey: "mkt",
      orderbookId: "ob",
      side: Shared.Side.Ask,
      amountIn: "3",
      amountOut: "4",
      remaining: "1",
      filled: "2",
      price: "0.75",
      createdAt: 1700000000000.0,
      expiration: 12.0,
      baseMint: "b",
      quoteMint: "q",
      outcomeIndex: 1.0,
      status: Shared.OrderStatus.Matching,
    }
    let limit = Order.Raw.SnapshotOrder.Limit({common, txSignature: "sig"})
    let trigger = Order.Raw.SnapshotOrder.Trigger({
      common,
      triggerOrderId: "t1",
      triggerPrice: "0.5",
      triggerType: Shared.TriggerType.StopLoss,
      timeInForce: Shared.TimeInForce.Ioc,
    })
    expect(Order.Raw.SnapshotOrder.t_encode(limit)->Order.Raw.SnapshotOrder.t_decode)->toEqual(
      Ok(limit),
    )
    expect(Order.Raw.SnapshotOrder.t_encode(trigger)->Order.Raw.SnapshotOrder.t_decode)->toEqual(
      Ok(trigger),
    )
  })

  test("unknown order_type is a decode error", () =>
    expect(parseJson(`{"order_type": "mystery"}`)->Order.Raw.SnapshotOrder.t_decode->Result.isError)->toBe(true)
  )
})

describe("Order.Raw.Event decode", () => {
  test("limit update: internally tagged, balance tree, status defaults to Open", () => {
    let json = parseJson(`{
      "order_type": "limit",
      "market_pubkey": "mkt1",
      "orderbook_id": "ob1",
      "timestamp": "2024-01-01T00:00:00Z",
      "tx_signature": "sig",
      "type": "PLACEMENT",
      "order": {
        "order_hash": "h1",
        "price": "0.5",
        "is_maker": true,
        "remaining": "8",
        "filled": "2",
        "fill_amount": "2",
        "side": "bid",
        "created_at": 1700000000000,
        "base_mint": "b",
        "quote_mint": "q",
        "outcome_index": 0,
        "balance": {
          "outcomes": [
            {"outcome_index": 0, "conditional_token": "ct", "idle": "1", "on_book": "2"}
          ]
        }
      }
    }`)
    switch Order.Raw.Event.t_decode(json) {
    | Ok(Limit(update)) =>
      expect(update.updateType == Shared.OrderUpdateType.Placement)->toBe(true)
      expect(update.txSignature)->toBe(Some("sig"))
      expect(update.order.orderHash)->toBe("h1")
      expect(update.order.status == Shared.OrderStatus.Open)->toBe(true)
      switch update.order.balance {
      | Some(balance) => {
          expect(Array.length(balance.outcomes))->toBe(1)
          let outcome = balance.outcomes->Array.getUnsafe(0)
          expect(outcome.onBook)->toBe("2")
        }
      | None => expect("balance")->toBe("present")
      }
    | Ok(Trigger(_)) => expect("trigger")->toBe("limit")
    | Error(error) => expect(error.message)->toBe("decoded")
    }
  })

  test("trigger update: defaults (type/tif/amounts) + empty result_status → None", () => {
    let json = parseJson(`{
      "order_type": "trigger",
      "trigger_order_id": "t1",
      "market_pubkey": "mkt1",
      "orderbook_id": "ob1",
      "trigger_price": "0.55",
      "trigger_above": true,
      "status": "created",
      "order_hash": "h2",
      "side": "bid",
      "result_status": "",
      "timestamp": "2024-01-01T00:00:00Z"
    }`)
    switch Order.Raw.Event.t_decode(json) {
    | Ok(Trigger(update)) =>
      expect(update.resultStatus)->toBe(None)
      expect(update.tif == Shared.TimeInForce.Gtc)->toBe(true)
      expect(update.updateType == Shared.TriggerUpdateType.Triggered)->toBe(true)
      expect(update.resultFilled)->toBe("0")
      expect(update.makerAmount)->toBe("0")
      expect(update.userPubkey)->toBe("")
      expect(update.status == Shared.TriggerStatus.Created)->toBe(true)
    | Ok(Limit(_)) => expect("limit")->toBe("trigger")
    | Error(error) => expect(error.message)->toBe("decoded")
    }
  })
})

describe("Messages user snapshot/order frames", () => {
  test("a full WS user snapshot frame decodes end-to-end (defaults applied)", () => {
    let json = parseJson(`{
      "type": "user",
      "version": 0.1,
      "data": {
        "event_type": "snapshot",
        "orders": [
          {
            "order_type": "limit",
            "order_hash": "h1",
            "market_pubkey": "mkt1",
            "orderbook_id": "ob1",
            "side": "bid",
            "amount_in": "65",
            "amount_out": "100",
            "remaining": "10",
            "created_at": 1700000000000,
            "base_mint": "b",
            "quote_mint": "q",
            "outcome_index": 0,
            "tx_signature": null
          }
        ],
        "market_balances": [
          {
            "market_pubkey": "mkt1",
            "deposit_assets": [
              {
                "deposit_asset": "usdc",
                "outcomes": [
                  {
                    "outcome_index": 0,
                    "conditional_token": "ct1",
                    "balance": "3",
                    "balance_idle": "1",
                    "balance_on_book": "2"
                  }
                ]
              }
            ]
          }
        ],
        "nonce": 7
      }
    }`)
    switch Messages.decodeMessage(json) {
    | Ok({kind: User(Snapshot(snapshot))}) =>
      expect(Array.length(snapshot.orders))->toBe(1)
      expect(snapshot.nonce)->toBe(7.0)
      expect(Array.length(snapshot.globalDeposits))->toBe(0)
      expect(Array.length(snapshot.notifications))->toBe(0)
      let balance = snapshot.marketBalances->Array.getUnsafe(0)
      expect(balance.marketPubkey)->toBe("mkt1")
    | Ok(_) => expect("other kind")->toBe("user snapshot")
    | Error(error) => expect(SdkError.toMessage(error))->toBe("decoded")
    }
  })

  test("a WS user order frame dispatches into Order.Raw.Event", () => {
    let json = parseJson(`{
      "type": "user",
      "version": 0.1,
      "data": {
        "event_type": "order",
        "order_type": "limit",
        "market_pubkey": "mkt1",
        "orderbook_id": "ob1",
        "timestamp": "2024-01-01T00:00:00Z",
        "order": {
          "order_hash": "h1",
          "price": "0.5",
          "is_maker": false,
          "remaining": "8",
          "filled": "2",
          "fill_amount": "2",
          "side": "ask",
          "created_at": 1700000000000,
          "base_mint": "b",
          "quote_mint": "q",
          "outcome_index": 0
        }
      }
    }`)
    switch Messages.decodeMessage(json) {
    | Ok({kind: User(Order(Limit(update)))}) => {
        expect(update.order.orderHash)->toBe("h1")
        // absent `type` defaults to UPDATE
        expect(update.updateType == Shared.OrderUpdateType.Update)->toBe(true)
      }
    | Ok(_) => expect("other kind")->toBe("user order")
    | Error(error) => expect(SdkError.toMessage(error))->toBe("decoded")
    }
  })
})

describe("Order fills response decode", () => {
  test("UserFillsResponse decodes orders with nested fill events", () => {
    let json = parseJson(`{
      "orders": [
        {
          "order_hash": "h1",
          "market_pubkey": "mkt1",
          "orderbook_id": "ob1",
          "side": "bid",
          "role": "maker",
          "price": "0.5",
          "size": "10",
          "filled_size": "4",
          "remaining_size": "6",
          "base_mint": "b",
          "quote_mint": "q",
          "outcome_index": 0,
          "status": "partially_filled",
          "created_at": 1700000000000,
          "fills": [
            {"fill_amount": "4", "tx_signature": "sig", "filled_at": 1700000001000}
          ]
        }
      ],
      "has_more": false
    }`)
    switch Order.Raw.UserFillsResponse.t_decode(json) {
    | Ok(response) =>
      expect(Array.length(response.orders))->toBe(1)
      let order = response.orders->Array.getUnsafe(0)
      expect(order.role == Order.Raw.Role.Maker)->toBe(true)
      expect(order.status == Order.Raw.FillStatus.PartiallyFilled)->toBe(true)
      expect((order.fills->Array.getUnsafe(0)).fillAmount)->toBe("4")
      expect(response.nextCursor)->toBe(None)
    | Error(error) => expect(error.message)->toBe("decoded")
    }
  })
})
