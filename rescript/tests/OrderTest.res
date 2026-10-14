open RescriptBun.Test
open RescriptBun.Test.Expect

let rules66 = TradingRulesFixture.make()

describe("Envelope.buildLimitOrder", () => {
  testAsync(
    "builds + signs a bid into a SubmitOrderRequest (scaled amounts + hex sig)",
    async () => {
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
        ~rules=rules66,
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
    },
  )
})

describe("Order cancel signing", () => {
  testAsync("cancelBodySigned produces a hex signature over the order hash", async () => {
    let seed: Uint8Array.t = %raw(`new Uint8Array(32).fill(9)`)
    let keypair = await SolanaKitKeys.createKeyPairFromPrivateKeyBytes(seed)
    let body = await Order.Client.cancelBodySigned(~orderHash="deadbeef", ~maker="maker1", ~keypair)
    expect(String.length(body.signatureHex))->toBe(128)
    expect(body.orderHash)->toBe("deadbeef")
  })
})

// ── Wire decoding: user snapshot orders, order events, snapshot frames ─────────
let parseJson: string => JSON.t = %raw("JSON.parse")

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

test("retired order types and trigger snapshots are rejected", () => {
  ["deposit", "withdraw", "stop_limit", "take_profit_limit"]->Array.forEach(value => {
    expect(Order.Type.t_decode(JSON.String(value))->Result.isError)->toBe(true)
  })
  expect(
    Order.Raw.SnapshotOrder.t_decode(parseJson(`{"order_type":"trigger"}`))->Result.isError,
  )->toBe(true)
})
