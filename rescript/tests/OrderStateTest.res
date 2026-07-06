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
  txSignature: ?txSignature,
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

let limitSnapshot = (~market, ~hash, ~remaining): Order.Raw.SnapshotOrder.t =>
  Order.Raw.SnapshotOrder.Limit({common: commonOf(~market, ~hash, ~orderbook="ob1", ~remaining)})

let triggerSnapshot = (~triggerId, ~market, ~orderbook): Order.Raw.SnapshotOrder.t =>
  Order.Raw.SnapshotOrder.Trigger({
    common: commonOf(~market, ~hash=`hash-${triggerId}`, ~orderbook, ~remaining="0"),
    triggerOrderId: triggerId,
    triggerPrice: "0.55",
    triggerType: Shared.TriggerType.TakeProfit,
  })

let triggerOrder = (~triggerId, ~market, ~orderbook): Order.Trigger.t => {
  triggerOrderId: triggerId,
  orderHash: `hash_${triggerId}`,
  marketPubkey: market,
  orderbookId: orderbook,
  triggerPrice: "0.55",
  triggerType: Shared.TriggerType.TakeProfit,
  side: Shared.Side.Bid,
  amountIn: "1000",
  amountOut: "500",
  timeInForce: Shared.TimeInForce.Gtc,
  createdAt: 1700000000000.0,
}

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

  test("limitOrderOfSnapshot maps the common fields", () => {
    switch limitSnapshot(~market="mkt222", ~hash="snap_hash", ~remaining="5") {
    | Order.Raw.SnapshotOrder.Limit(limit) => {
        let order = Order.Raw.SnapshotLimit.toLimit(limit)
        expect(order.orderHash)->toBe("snap_hash")
        expect(order.marketPubkey)->toBe("mkt222")
        expect(order.size)->toBe("5")
        expect(order.txSignature)->toBe(None)
      }
    | Trigger(_) => expect("trigger")->toBe("limit")
    }
  })

  test("triggerOrderOfSnapshot: absent TIF defaults to GTC", () => {
    switch triggerSnapshot(~triggerId="trig-123", ~market="mkt-xyz", ~orderbook="ob_test") {
    | Order.Raw.SnapshotOrder.Trigger(trigger) => {
        let order = Order.Raw.SnapshotTrigger.toTrigger(trigger)
        expect(order.triggerOrderId)->toBe("trig-123")
        expect(order.triggerType == Shared.TriggerType.TakeProfit)->toBe(true)
        expect(order.amountIn)->toBe("1000")
        expect(order.timeInForce == Shared.TimeInForce.Gtc)->toBe(true)
      }
    | Limit(_) => expect("limit")->toBe("trigger")
    }
  })

  test("triggerOrderOfUpdate: trigger direction implies the type; ISO timestamp → ms", () => {
    let update: Order.Raw.TriggerUpdate.t = {
      triggerOrderId: "t1",
      userPubkey: "user",
      marketPubkey: "mkt1",
      orderbookId: "ob1",
      triggerPrice: "0.55",
      triggerAbove: false,
      status: Shared.TriggerStatus.Created,
      updateType: Shared.TriggerUpdateType.Created,
      orderHash: "h1",
      side: Shared.Side.Bid,
      resultFilled: "0",
      resultRemaining: "0",
      timestamp: "2024-01-01T00:00:00Z",
      makerAmount: "1000",
      takerAmount: "500",
      tif: Shared.TimeInForce.Ioc,
    }
    let order = Order.Raw.TriggerUpdate.toTrigger(update)
    expect(order.triggerType == Shared.TriggerType.StopLoss)->toBe(true)
    expect(order.amountIn)->toBe("1000")
    expect(order.amountOut)->toBe("500")
    expect(order.timeInForce == Shared.TimeInForce.Ioc)->toBe(true)
    expect(order.createdAt)->toBe(1704067200000.0)
  })

  test("limitPrice: out/in for asks, in/out for bids, None on a zero divisor", () => {
    let ask = {...triggerOrder(~triggerId="t", ~market="m", ~orderbook="ob"), side: Shared.Side.Ask}
    expect(Order.Trigger.limitPrice(ask))->toBe(Some("0.5")) // 500 / 1000
    let bid = triggerOrder(~triggerId="t", ~market="m", ~orderbook="ob")
    expect(Order.Trigger.limitPrice(bid))->toBe(Some("2")) // 1000 / 500
    let empty = {...bid, amountOut: "0"}
    expect(Order.Trigger.limitPrice(empty))->toBe(None)
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

describe("Order.State.Triggers", () => {
  test("insert + get + size", () => {
    let container = Order.State.Triggers.make()
    expect(Order.State.Triggers.isEmpty(container))->toBe(true)
    expect(Order.State.Triggers.size(container))->toBe(0)

    Order.State.Triggers.insert(container, triggerOrder(~triggerId="t1", ~market="mkt1", ~orderbook="ob1"))
    expect(Order.State.Triggers.isEmpty(container))->toBe(false)
    expect(Order.State.Triggers.size(container))->toBe(1)
    switch Order.State.Triggers.get(container, ~marketPubkey="mkt1", ~orderbookId="ob1") {
    | Some(orders) => expect((orders->Array.getUnsafe(0)).triggerOrderId)->toBe("t1")
    | None => expect("orders")->toBe("present")
    }
  })

  test("getById finds across markets; unknown id is None", () => {
    let container = Order.State.Triggers.make()
    Order.State.Triggers.insert(container, triggerOrder(~triggerId="t1", ~market="mkt1", ~orderbook="ob1"))
    Order.State.Triggers.insert(container, triggerOrder(~triggerId="t2", ~market="mkt1", ~orderbook="ob2"))
    switch Order.State.Triggers.getById(container, ~triggerOrderId="t2") {
    | Some(order) => expect(order.triggerOrderId)->toBe("t2")
    | None => expect("t2")->toBe("present")
    }
    expect(Order.State.Triggers.getById(container, ~triggerOrderId="t99")->Option.isNone)->toBe(true)
  })

  test("groups by market and orderbook", () => {
    let container = Order.State.Triggers.make()
    Order.State.Triggers.insert(container, triggerOrder(~triggerId="t1", ~market="mkt1", ~orderbook="ob1"))
    Order.State.Triggers.insert(container, triggerOrder(~triggerId="t2", ~market="mkt1", ~orderbook="ob1"))
    Order.State.Triggers.insert(container, triggerOrder(~triggerId="t3", ~market="mkt1", ~orderbook="ob2"))
    expect(Order.State.Triggers.size(container))->toBe(3)
    expect(
      Order.State.Triggers.get(container, ~marketPubkey="mkt1", ~orderbookId="ob1")
      ->Option.mapOr(0, Array.length),
    )->toBe(2)
    expect(
      Order.State.Triggers.get(container, ~marketPubkey="mkt1", ~orderbookId="ob2")
      ->Option.mapOr(0, Array.length),
    )->toBe(1)
    switch Order.State.Triggers.getByMarket(container, ~marketPubkey="mkt1") {
    | Some(byOrderbook) => expect(Array.length(Dict.keysToArray(byOrderbook)))->toBe(2)
    | None => expect("market")->toBe("present")
    }
  })

  test("remove returns the removed order", () => {
    let container = Order.State.Triggers.make()
    Order.State.Triggers.insert(container, triggerOrder(~triggerId="t1", ~market="mkt1", ~orderbook="ob1"))
    Order.State.Triggers.insert(container, triggerOrder(~triggerId="t2", ~market="mkt1", ~orderbook="ob1"))
    let removed = Order.State.Triggers.remove(container, ~triggerOrderId="t1")
    expect(removed->Option.mapOr("", order => order.triggerOrderId))->toBe("t1")
    expect(Order.State.Triggers.size(container))->toBe(1)
    expect(Order.State.Triggers.getById(container, ~triggerOrderId="t1")->Option.isNone)->toBe(true)
    expect(Order.State.Triggers.getById(container, ~triggerOrderId="t2")->Option.isSome)->toBe(true)
    expect(Order.State.Triggers.remove(container, ~triggerOrderId="t99")->Option.isNone)->toBe(true)
  })

  test("clear + all", () => {
    let container = Order.State.Triggers.make()
    Order.State.Triggers.insert(container, triggerOrder(~triggerId="t1", ~market="mkt1", ~orderbook="ob1"))
    Order.State.Triggers.insert(container, triggerOrder(~triggerId="t2", ~market="mkt2", ~orderbook="ob2"))
    expect(Array.length(Order.State.Triggers.all(container)))->toBe(2)
    Order.State.Triggers.clear(container)
    expect(Order.State.Triggers.isEmpty(container))->toBe(true)
  })
})

describe("Order.State.fromSnapshotOrders", () => {
  test("splits limits and triggers; zero-remaining limits are skipped", () => {
    let (openOrders, triggerOrders) = Order.State.fromSnapshotOrders([
      limitSnapshot(~market="mkt1", ~hash="o1", ~remaining="1"),
      limitSnapshot(~market="mkt1", ~hash="o2", ~remaining="0"),
      triggerSnapshot(~triggerId="t1", ~market="mkt-xyz", ~orderbook="ob_test"),
      triggerSnapshot(~triggerId="t2", ~market="mkt-xyz", ~orderbook="ob_test"),
    ])
    switch Order.State.Limits.getByMarket(openOrders, ~marketPubkey="mkt1") {
    | Some(byOrderbook) => {
        let total =
          byOrderbook->Dict.valuesToArray->Array.reduce(0, (sum, orders) => sum + Array.length(orders))
        expect(total)->toBe(1)
      }
    | None => expect("mkt1")->toBe("present")
    }
    expect(Order.State.Triggers.size(triggerOrders))->toBe(2)
    expect(
      Order.State.Triggers.get(triggerOrders, ~marketPubkey="mkt-xyz", ~orderbookId="ob_test")
      ->Option.mapOr(0, Array.length),
    )->toBe(2)
  })
})
