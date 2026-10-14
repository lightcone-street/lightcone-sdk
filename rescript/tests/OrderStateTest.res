open RescriptBun.Test
open RescriptBun.Test.Expect

// Mirrors the semantics asserted in rust/src/domain/order/{state,convert}.rs:
// upsert replaces by hash, remove drops across markets, snapshot conversion skips
// zero-remaining limits, and the wire→domain conversions carry the Decimal size
// math (size = filled + remaining). Feeds synthetic decoded payloads (no network).

let wsOrder = (~orderHash, ~remaining, ~filled): Order.Raw.WsOrder.t => {
  orderHash,
  price: "0.5",
  isMaker: true,
  remaining,
  filled,
  fillAmount: "0",
  side: Shared.Side.Bid,
  createdAt: 1700000000000.0,
  baseMint: "base",
  quoteMint: "quote",
  outcomeIndex: 0.0,
  status: Shared.OrderStatus.Open,
}

let orderUpdate = (
  ~market,
  ~orderHash,
  ~orderbookId,
  ~remaining,
  ~filled="0",
  ~txSignature: option<string>=?,
): Order.Raw.Update.t => {
  marketPubkey: market,
  orderbookId,
  timestamp: "2024-01-01T00:00:00Z",
  ?txSignature,
  updateType: Shared.OrderUpdateType.Update,
  order: wsOrder(~orderHash, ~remaining, ~filled),
}

let commonOf = (~market, ~hash, ~orderbook, ~remaining): Order.Raw.SnapshotCommon.t => {
  orderHash: hash,
  marketPubkey: market,
  orderbookId: orderbook,
  side: Shared.Side.Bid,
  amountIn: "1000",
  amountOut: "500",
  remaining,
  filled: "0",
  price: "0.5",
  createdAt: 1700000000000.0,
  expiration: 0.0,
  baseMint: "b",
  quoteMint: "q",
  outcomeIndex: 0.0,
  status: Shared.OrderStatus.Open,
}

let limitSnapshot = (
  ~market,
  ~hash,
  ~remaining,
): Order.Raw.SnapshotOrder.t => Order.Raw.SnapshotOrder.Limit({
  common: commonOf(~market, ~hash, ~orderbook="ob1", ~remaining),
})

describe("Order wire → domain conversions", () => {
  test("limitOrderOfUpdate: size = filled + remaining; tx signature carried", () => {
    let update = orderUpdate(
      ~market="mkt111",
      ~orderHash="hash_xyz",
      ~orderbookId="ob_abc",
      ~remaining="8",
      ~filled="2",
      ~txSignature="sig123",
    )
    let order = Order.Raw.Update.toLimit(update)
    expect(order.size)->toBe("10")
    expect(order.filledSize)->toBe("2")
    expect(order.remainingSize)->toBe("8")
    expect(order.txSignature)->toBe(Some("sig123"))
    expect(order.orderHash)->toBe("hash_xyz")
  })
})

describe("Order.State.Limits", () => {
  test("upsert adds an order", () => {
    let container = Order.State.Limits.make()
    Order.State.Limits.upsert(
      container,
      orderUpdate(~market="mkt1", ~orderHash="hash1", ~orderbookId="ob1", ~remaining="10"),
    )
    expect(Order.State.Limits.isEmpty(container))->toBe(false)
    switch Order.State.Limits.get(container, ~marketPubkey="mkt1", ~orderbookId="ob1") {
    | Some(orders) => {
        expect(Array.length(orders))->toBe(1)
        expect((orders->Array.getUnsafe(0)).orderHash)->toBe("hash1")
      }
    | None => expect("orders")->toBe("present")
    }
  })

  test("upsert replaces an order with the same hash", () => {
    let container = Order.State.Limits.make()
    Order.State.Limits.upsert(
      container,
      orderUpdate(~market="mkt1", ~orderHash="hash1", ~orderbookId="ob1", ~remaining="10"),
    )
    Order.State.Limits.upsert(
      container,
      orderUpdate(~market="mkt1", ~orderHash="hash1", ~orderbookId="ob1", ~remaining="5"),
    )
    switch Order.State.Limits.get(container, ~marketPubkey="mkt1", ~orderbookId="ob1") {
    | Some(orders) => {
        expect(Array.length(orders))->toBe(1)
        expect((orders->Array.getUnsafe(0)).remainingSize)->toBe("5")
      }
    | None => expect("orders")->toBe("present")
    }
  })

  test("remove drops the hash everywhere", () => {
    let container = Order.State.Limits.make()
    Order.State.Limits.upsert(
      container,
      orderUpdate(~market="mkt1", ~orderHash="hash1", ~orderbookId="ob1", ~remaining="10"),
    )
    Order.State.Limits.upsert(
      container,
      orderUpdate(~market="mkt1", ~orderHash="hash2", ~orderbookId="ob1", ~remaining="5"),
    )
    Order.State.Limits.remove(container, ~orderHash="hash1")
    switch Order.State.Limits.get(container, ~marketPubkey="mkt1", ~orderbookId="ob1") {
    | Some(orders) => {
        expect(Array.length(orders))->toBe(1)
        expect((orders->Array.getUnsafe(0)).orderHash)->toBe("hash2")
      }
    | None => expect("orders")->toBe("present")
    }
  })

  test("getByMarket groups per orderbook; unknown market is None", () => {
    let container = Order.State.Limits.make()
    Order.State.Limits.upsert(
      container,
      orderUpdate(~market="mkt1", ~orderHash="hash1", ~orderbookId="ob1", ~remaining="10"),
    )
    Order.State.Limits.upsert(
      container,
      orderUpdate(~market="mkt1", ~orderHash="hash2", ~orderbookId="ob2", ~remaining="5"),
    )
    Order.State.Limits.upsert(
      container,
      orderUpdate(~market="mkt2", ~orderHash="hash3", ~orderbookId="ob3", ~remaining="1"),
    )
    switch Order.State.Limits.getByMarket(container, ~marketPubkey="mkt1") {
    | Some(byOrderbook) => expect(Array.length(Dict.keysToArray(byOrderbook)))->toBe(2)
    | None => expect("market")->toBe("present")
    }
    expect(
      Order.State.Limits.getByMarket(container, ~marketPubkey="mkt_nonexistent")->Option.isNone,
    )->toBe(true)
  })

  test("clear empties the container", () => {
    let container = Order.State.Limits.make()
    Order.State.Limits.upsert(
      container,
      orderUpdate(~market="mkt1", ~orderHash="hash1", ~orderbookId="ob1", ~remaining="10"),
    )
    Order.State.Limits.clear(container)
    expect(Order.State.Limits.isEmpty(container))->toBe(true)
    expect(
      Order.State.Limits.get(container, ~marketPubkey="mkt1", ~orderbookId="ob1")->Option.isNone,
    )->toBe(true)
  })
})

test("snapshots skip fully filled limits and keep market grouping", () => {
  let state = Order.State.fromSnapshotOrders([
    limitSnapshot(~market="mkt1", ~hash="open", ~remaining="1"),
    limitSnapshot(~market="mkt1", ~hash="filled", ~remaining="0"),
  ])
  expect(
    Order.State.Limits.get(state, ~marketPubkey="mkt1", ~orderbookId="ob1")->Option.mapOr(
      0,
      Array.length,
    ),
  )->toBe(1)
})
