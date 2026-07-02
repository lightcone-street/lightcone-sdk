// OrderState — the user's open limit orders and resting trigger orders, maintained
// from the WS user channel (mirrors rust/src/domain/order/{mod,state,convert}.rs:
// the `LimitOrder` / `TriggerOrder` domain types, the `UserOpenLimitOrders` /
// `UserTriggerOrders` containers, and the wire→domain conversions).
//
// Both containers group orders market → orderbook → array (insertion order kept,
// matching the Rust `HashMap<PubkeyStr, HashMap<OrderBookId, Vec<_>>>`). Decimal
// wire strings stay strings; the only Decimal math is the derived `size`
// (filled + remaining) and `limitPrice` — malformed strings are treated as zero,
// matching `OrderbookState`'s tolerant parsing.
//
// Update flow (the SDK stays policy-free, like Rust): a `UserUpdate.Snapshot`
// seeds both containers via `ofSnapshotOrders`; a `UserUpdate.Order(Limit(_))`
// feeds `UserOpenLimitOrders.upsert` (or `remove` on cancellation); a
// `UserUpdate.Order(Trigger(_))` converts via `triggerOrderOfUpdate` for
// `UserTriggerOrders.insert` / `remove`.

// ── Domain types ──────────────────────────────────────────────────────────────
// An open limit order. `size` is the original size (filled + remaining).
type limitOrder = {
  marketPubkey: Shared.pubkeyStr,
  orderbookId: Shared.orderBookId,
  txSignature?: string,
  baseMint: Shared.pubkeyStr,
  quoteMint: Shared.pubkeyStr,
  orderHash: string,
  side: Shared.Side.t,
  size: string,
  price: string,
  filledSize: string,
  remainingSize: string,
  // Unix milliseconds.
  createdAt: float,
  status: Shared.OrderStatus.t,
  outcomeIndex: float,
}

// A resting trigger (TP/SL) order.
type triggerOrder = {
  triggerOrderId: string,
  orderHash: string,
  marketPubkey: Shared.pubkeyStr,
  orderbookId: Shared.orderBookId,
  triggerPrice: string,
  triggerType: Shared.TriggerType.t,
  side: Shared.Side.t,
  amountIn: string,
  amountOut: string,
  timeInForce: Shared.TimeInForce.t,
  // Unix milliseconds.
  createdAt: float,
}

// ── Decimal-string helpers (tolerant: malformed → zero) ──────────────────────
let decimalOrZero = (value: string): Decimal.t =>
  switch Decimal.fromString(value) {
  | decimal => decimal
  | exception JsExn(_) => Decimal.fromInt(0)
  }

let sizeOf = (~filled: string, ~remaining: string): string =>
  Decimal.plus(decimalOrZero(filled), decimalOrZero(remaining))->Decimal.toString

let isZeroDecimal = (value: string): bool => Decimal.isZero(decimalOrZero(value))

// ISO-8601 → epoch ms (NaN for malformed input, standard JS Date semantics).
let epochMsOfIso = (timestamp: string): float => Date.fromString(timestamp)->Date.getTime

// The implied limit price of a trigger order: out/in for asks, in/out for bids
// (`None` when the divisor is not positive). Mirrors Rust `TriggerOrder::limit_price`.
let limitPrice = (order: triggerOrder): option<string> => {
  let amountIn = decimalOrZero(order.amountIn)
  let amountOut = decimalOrZero(order.amountOut)
  let zero = Decimal.fromInt(0)
  switch order.side {
  | Ask if Decimal.gt(amountIn, zero) => Some(Decimal.div(amountOut, amountIn)->Decimal.toString)
  | Bid if Decimal.gt(amountOut, zero) => Some(Decimal.div(amountIn, amountOut)->Decimal.toString)
  | _ => None
  }
}

// ── Wire → domain conversions ─────────────────────────────────────────────────
// WS limit-order update → domain order (Rust `From<wire::OrderUpdate> for LimitOrder`).
let limitOrderOfUpdate = (update: Order.orderUpdate): limitOrder => {
  marketPubkey: update.marketPubkey,
  orderbookId: update.orderbookId,
  txSignature: ?update.txSignature,
  baseMint: update.order.baseMint,
  quoteMint: update.order.quoteMint,
  orderHash: update.order.orderHash,
  side: update.order.side,
  size: sizeOf(~filled=update.order.filled, ~remaining=update.order.remaining),
  price: update.order.price,
  filledSize: update.order.filled,
  remainingSize: update.order.remaining,
  createdAt: update.order.createdAt,
  status: update.order.status,
  outcomeIndex: update.order.outcomeIndex,
}

// Snapshot limit arm → domain order (Rust `limit_snapshot_to_order`).
let limitOrderOfSnapshot = (snapshot: Order.UserSnapshotOrder.limit): limitOrder => {
  marketPubkey: snapshot.common.marketPubkey,
  orderbookId: snapshot.common.orderbookId,
  txSignature: ?snapshot.txSignature,
  baseMint: snapshot.common.baseMint,
  quoteMint: snapshot.common.quoteMint,
  orderHash: snapshot.common.orderHash,
  side: snapshot.common.side,
  size: sizeOf(~filled=snapshot.common.filled, ~remaining=snapshot.common.remaining),
  price: snapshot.common.price,
  filledSize: snapshot.common.filled,
  remainingSize: snapshot.common.remaining,
  createdAt: snapshot.common.createdAt,
  status: snapshot.common.status,
  outcomeIndex: snapshot.common.outcomeIndex,
}

// Snapshot trigger arm → domain order (Rust `trigger_snapshot_to_order`).
let triggerOrderOfSnapshot = (snapshot: Order.UserSnapshotOrder.trigger): triggerOrder => {
  triggerOrderId: snapshot.triggerOrderId,
  orderHash: snapshot.common.orderHash,
  marketPubkey: snapshot.common.marketPubkey,
  orderbookId: snapshot.common.orderbookId,
  triggerPrice: snapshot.triggerPrice,
  triggerType: snapshot.triggerType,
  side: snapshot.common.side,
  amountIn: snapshot.common.amountIn,
  amountOut: snapshot.common.amountOut,
  timeInForce: snapshot.timeInForce->Option.getOr(Shared.TimeInForce.Gtc),
  createdAt: snapshot.common.createdAt,
}

// WS trigger update → domain order (Rust `TriggerOrderUpdate::into_trigger_order`):
// the trigger type is implied by the trigger direction.
let triggerOrderOfUpdate = (update: Order.triggerOrderUpdate): triggerOrder => {
  triggerOrderId: update.triggerOrderId,
  orderHash: update.orderHash,
  marketPubkey: update.marketPubkey,
  orderbookId: update.orderbookId,
  triggerPrice: update.triggerPrice,
  triggerType: update.triggerAbove ? Shared.TriggerType.TakeProfit : Shared.TriggerType.StopLoss,
  side: update.side,
  amountIn: update.makerAmount,
  amountOut: update.takerAmount,
  timeInForce: update.tif,
  createdAt: epochMsOfIso(update.timestamp),
}

// ── UserOpenLimitOrders ───────────────────────────────────────────────────────
module UserOpenLimitOrders = {
  // market → orderbook → orders (insertion order).
  type t = {mutable orders: Dict.t<Dict.t<array<limitOrder>>>}

  let make = (): t => {orders: Dict.make()}

  let byMarketOrCreate = (state: t, marketPubkey: Shared.pubkeyStr): Dict.t<array<limitOrder>> =>
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
  ): option<array<limitOrder>> =>
    state.orders->Dict.get(marketPubkey)->Option.flatMap(byOrderbook => byOrderbook->Dict.get(orderbookId))

  let getByMarket = (state: t, ~marketPubkey: Shared.pubkeyStr): option<Dict.t<array<limitOrder>>> =>
    state.orders->Dict.get(marketPubkey)

  // Append without deduplication (snapshot seeding).
  let insert = (state: t, order: limitOrder): unit => {
    let byOrderbook = byMarketOrCreate(state, order.marketPubkey)
    let orders = byOrderbook->Dict.get(order.orderbookId)->Option.getOr([])
    orders->Array.push(order)
    byOrderbook->Dict.set(order.orderbookId, orders)
  }

  // Replace any order with the same hash, then append the update's order.
  let upsert = (state: t, update: Order.orderUpdate): unit => {
    let byOrderbook = byMarketOrCreate(state, update.marketPubkey)
    let orders =
      byOrderbook
      ->Dict.get(update.orderbookId)
      ->Option.getOr([])
      ->Array.filter(order => order.orderHash != update.order.orderHash)
    orders->Array.push(limitOrderOfUpdate(update))
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

// ── UserTriggerOrders ─────────────────────────────────────────────────────────
module UserTriggerOrders = {
  // market → orderbook → orders (insertion order).
  type t = {mutable orders: Dict.t<Dict.t<array<triggerOrder>>>}

  let make = (): t => {orders: Dict.make()}

  let get = (
    state: t,
    ~marketPubkey: Shared.pubkeyStr,
    ~orderbookId: Shared.orderBookId,
  ): option<array<triggerOrder>> =>
    state.orders->Dict.get(marketPubkey)->Option.flatMap(byOrderbook => byOrderbook->Dict.get(orderbookId))

  let getByMarket = (state: t, ~marketPubkey: Shared.pubkeyStr): option<Dict.t<array<triggerOrder>>> =>
    state.orders->Dict.get(marketPubkey)

  // Every resting trigger order across all markets/orderbooks.
  let all = (state: t): array<triggerOrder> =>
    state.orders
    ->Dict.valuesToArray
    ->Array.flatMap(byOrderbook => byOrderbook->Dict.valuesToArray->Array.flat)

  let getById = (state: t, ~triggerOrderId: string): option<triggerOrder> =>
    all(state)->Array.find(order => order.triggerOrderId == triggerOrderId)

  let insert = (state: t, order: triggerOrder): unit => {
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
  let remove = (state: t, ~triggerOrderId: string): option<triggerOrder> => {
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
// Split a user snapshot's orders into both containers (Rust `convert_snapshot_orders`):
// fully-filled limit orders (zero remaining) are skipped; trigger orders are kept as-is.
let ofSnapshotOrders = (
  orders: array<Order.UserSnapshotOrder.t>,
): (UserOpenLimitOrders.t, UserTriggerOrders.t) => {
  let openOrders = UserOpenLimitOrders.make()
  let triggerOrders = UserTriggerOrders.make()
  orders->Array.forEach(order =>
    switch order {
    | Order.UserSnapshotOrder.Limit(limit) =>
      if !isZeroDecimal(limit.common.remaining) {
        UserOpenLimitOrders.insert(openOrders, limitOrderOfSnapshot(limit))
      }
    | Trigger(trigger) => UserTriggerOrders.insert(triggerOrders, triggerOrderOfSnapshot(trigger))
    }
  )
  (openOrders, triggerOrders)
}
