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

  let get = (state: t, ~marketPubkey: Shared.pubkeyStr, ~orderbookId: Shared.orderBookId): option<
    array<Order__Model.Limit.t>,
  > =>
    state.orders
    ->Dict.get(marketPubkey)
    ->Option.flatMap(byOrderbook => byOrderbook->Dict.get(orderbookId))

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
        byOrderbook->Dict.set(
          orderbookId,
          orders->Array.filter(order => order.orderHash != orderHash),
        )
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

let fromSnapshotOrders = (orders: array<Order__Raw.SnapshotOrder.t>): Limits.t => {
  let openOrders = Limits.make()
  orders->Array.forEach(order =>
    switch order {
    | Order__Raw.SnapshotOrder.Limit(limit) =>
      if !isZeroDecimal(limit.common.remaining) {
        Limits.insert(openOrders, Order__Raw.SnapshotLimit.toLimit(limit))
      }
    }
  )
  openOrders
}
