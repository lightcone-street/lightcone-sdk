// Live order state — the user's open limit orders and resting trigger orders,
// maintained from the WS user channel.
//
// Both containers group orders market → orderbook → array (insertion order
// kept). Decimal wire strings stay strings; the only Decimal math here is the
// zero-remaining test — malformed strings are treated as zero, matching
// `Orderbook.State`'s tolerant parsing.
//
// Update flow (the SDK stays policy-free): a `UserUpdate.Snapshot` seeds both
// containers via `fromSnapshotOrders`; a `UserUpdate.Order(Limit(_))` feeds
// `Limits.upsert` (or `remove` on cancellation); a `UserUpdate.Order(Trigger(_))`
// converts via `Order__Raw.TriggerUpdate.toTrigger` for `Triggers.insert` /
// `remove`.

// ── Decimal-string helpers (tolerant: malformed → zero) ──────────────────────
let decimalOrZero = (value: string): Decimal.t =>
  switch Decimal.fromString(value) {
  | decimal => decimal
  | exception JsExn(_) => Decimal.fromInt(0)
  }

let isZeroDecimal = (value: string): bool => Decimal.isZero(decimalOrZero(value))

// ── Limits ────────────────────────────────────────────────────────────────────
module Limits = {
  // market → orderbook → orders (insertion order).
  type t = {mutable orders: Dict.t<Dict.t<array<Order__Model.Limit.t>>>}

  let make = (): t => {orders: Dict.make()}

  let byMarketOrCreate = (state: t, marketPubkey: Shared.pubkeyStr): Dict.t<
    array<Order__Model.Limit.t>,
  > =>
    switch state.orders->Dict.get(marketPubkey) {
    | Some(byOrderbook) => byOrderbook
    | None =>
      let byOrderbook = Dict.make()
      state.orders->Dict.set(marketPubkey, byOrderbook)
      byOrderbook
    }

  let get = (
    state: t,
    ~marketPubkey: Shared.pubkeyStr,
    ~orderbookId: Shared.orderBookId,
  ): option<array<Order__Model.Limit.t>> =>
    state.orders->Dict.get(marketPubkey)->Option.flatMap(byOrderbook => byOrderbook->Dict.get(orderbookId))

  let getByMarket = (state: t, ~marketPubkey: Shared.pubkeyStr): option<
    Dict.t<array<Order__Model.Limit.t>>,
  > => state.orders->Dict.get(marketPubkey)

  // Append without deduplication (snapshot seeding).
  let insert = (state: t, order: Order__Model.Limit.t): unit => {
    let byOrderbook = byMarketOrCreate(state, order.marketPubkey)
    let orders = byOrderbook->Dict.get(order.orderbookId)->Option.getOr([])
    orders->Array.push(order)
    byOrderbook->Dict.set(order.orderbookId, orders)
  }

  // Replace any order with the same hash, then append the update's order.
  let upsert = (state: t, update: Order__Raw.Update.t): unit => {
    let byOrderbook = byMarketOrCreate(state, update.marketPubkey)
    let orders =
      byOrderbook
      ->Dict.get(update.orderbookId)
      ->Option.getOr([])
      ->Array.filter(order => order.orderHash != update.order.orderHash)
    orders->Array.push(Order__Raw.Update.toLimit(update))
    byOrderbook->Dict.set(update.orderbookId, orders)
  }

  // Drop the order with this hash from every market/orderbook.
  let remove = (state: t, ~orderHash: string): unit =>
    state.orders
    ->Dict.valuesToArray
    ->Array.forEach(byOrderbook =>
      byOrderbook
      ->Dict.toArray
      ->Array.forEach(((orderbookId, orders)) =>
        byOrderbook->Dict.set(orderbookId, orders->Array.filter(order => order.orderHash != orderHash))
      )
    )

  let clear = (state: t): unit => state.orders = Dict.make()

  let isEmpty = (state: t): bool =>
    state.orders
    ->Dict.valuesToArray
    ->Array.every(byOrderbook =>
      byOrderbook->Dict.valuesToArray->Array.every(orders => Array.length(orders) == 0)
    )
}

// ── Triggers ──────────────────────────────────────────────────────────────────
module Triggers = {
  // market → orderbook → orders (insertion order).
  type t = {mutable orders: Dict.t<Dict.t<array<Order__Model.Trigger.t>>>}

  let make = (): t => {orders: Dict.make()}

  let get = (
    state: t,
    ~marketPubkey: Shared.pubkeyStr,
    ~orderbookId: Shared.orderBookId,
  ): option<array<Order__Model.Trigger.t>> =>
    state.orders->Dict.get(marketPubkey)->Option.flatMap(byOrderbook => byOrderbook->Dict.get(orderbookId))

  let getByMarket = (state: t, ~marketPubkey: Shared.pubkeyStr): option<
    Dict.t<array<Order__Model.Trigger.t>>,
  > => state.orders->Dict.get(marketPubkey)

  // Every resting trigger order across all markets/orderbooks.
  let all = (state: t): array<Order__Model.Trigger.t> =>
    state.orders
    ->Dict.valuesToArray
    ->Array.flatMap(byOrderbook => byOrderbook->Dict.valuesToArray->Array.flat)

  let getById = (state: t, ~triggerOrderId: string): option<Order__Model.Trigger.t> =>
    all(state)->Array.find(order => order.triggerOrderId == triggerOrderId)

  let insert = (state: t, order: Order__Model.Trigger.t): unit => {
    let byOrderbook = switch state.orders->Dict.get(order.marketPubkey) {
    | Some(byOrderbook) => byOrderbook
    | None =>
      let byOrderbook = Dict.make()
      state.orders->Dict.set(order.marketPubkey, byOrderbook)
      byOrderbook
    }
    let orders = byOrderbook->Dict.get(order.orderbookId)->Option.getOr([])
    orders->Array.push(order)
    byOrderbook->Dict.set(order.orderbookId, orders)
  }

  // Remove (and return) the trigger order with this id, if resting.
  let remove = (state: t, ~triggerOrderId: string): option<Order__Model.Trigger.t> => {
    let removed = ref(None)
    state.orders
    ->Dict.valuesToArray
    ->Array.forEach(byOrderbook =>
      byOrderbook
      ->Dict.toArray
      ->Array.forEach(((orderbookId, orders)) =>
        if removed.contents->Option.isNone {
          switch orders->Array.find(order => order.triggerOrderId == triggerOrderId) {
          | Some(order) =>
            removed := Some(order)
            byOrderbook->Dict.set(
              orderbookId,
              orders->Array.filter(order => order.triggerOrderId != triggerOrderId),
            )
          | None => ()
          }
        }
      )
    )
    removed.contents
  }

  let clear = (state: t): unit => state.orders = Dict.make()

  let isEmpty = (state: t): bool =>
    state.orders
    ->Dict.valuesToArray
    ->Array.every(byOrderbook =>
      byOrderbook->Dict.valuesToArray->Array.every(orders => Array.length(orders) == 0)
    )

  let size = (state: t): int => all(state)->Array.length
}

// ── Snapshot seeding ──────────────────────────────────────────────────────────
// Split a user snapshot's orders into both containers: fully-filled limit orders
// (zero remaining) are skipped; trigger orders are kept as-is.
let fromSnapshotOrders = (orders: array<Order__Raw.SnapshotOrder.t>): (Limits.t, Triggers.t) => {
  let openOrders = Limits.make()
  let triggerOrders = Triggers.make()
  orders->Array.forEach(order =>
    switch order {
    | Order__Raw.SnapshotOrder.Limit(limit) =>
      if !isZeroDecimal(limit.common.remaining) {
        Limits.insert(openOrders, Order__Raw.SnapshotLimit.toLimit(limit))
      }
    | Trigger(trigger) => Triggers.insert(triggerOrders, Order__Raw.SnapshotTrigger.toTrigger(trigger))
    }
  )
  (openOrders, triggerOrders)
}
