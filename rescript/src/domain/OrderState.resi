// OrderState — the user's open limit orders + resting trigger orders from the WS
// user channel (mirrors rust order/{mod,state,convert}.rs). Snapshot seeds via
// `ofSnapshotOrders`; limit updates feed `UserOpenLimitOrders.upsert`/`remove`;
// trigger updates convert via `triggerOrderOfUpdate` for `UserTriggerOrders`.
// Decimal fields are wire strings; malformed values are treated as zero.

// ── Domain types ──────────────────────────────────────────────────────────────
// An open limit order. `size` is the original size (filled + remaining).
@genType
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
@genType
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

// The implied limit price of a trigger order: out/in for asks, in/out for bids
// (`None` when the divisor is not positive).
@genType
let limitPrice: triggerOrder => option<string>

// ── Wire → domain conversions ─────────────────────────────────────────────────
@genType
let limitOrderOfUpdate: Order.orderUpdate => limitOrder
@genType
let limitOrderOfSnapshot: Order.UserSnapshotOrder.limit => limitOrder
@genType
let triggerOrderOfSnapshot: Order.UserSnapshotOrder.trigger => triggerOrder
// The trigger type is implied by the trigger direction (`triggerAbove`).
@genType
let triggerOrderOfUpdate: Order.triggerOrderUpdate => triggerOrder

// ── UserOpenLimitOrders ───────────────────────────────────────────────────────
// Open limit orders grouped market → orderbook → array (insertion order).
module UserOpenLimitOrders: {
  @genType.opaque
  type t

  @genType
  let make: unit => t

  @genType
  let get: (t, ~marketPubkey: Shared.pubkeyStr, ~orderbookId: Shared.orderBookId) => option<array<limitOrder>>
  @genType
  let getByMarket: (t, ~marketPubkey: Shared.pubkeyStr) => option<Dict.t<array<limitOrder>>>

  // Append without deduplication (snapshot seeding).
  @genType
  let insert: (t, limitOrder) => unit
  // Replace any order with the same hash, then append the update's order.
  @genType
  let upsert: (t, Order.orderUpdate) => unit
  // Drop the order with this hash from every market/orderbook.
  @genType
  let remove: (t, ~orderHash: string) => unit

  @genType
  let clear: t => unit
  @genType
  let isEmpty: t => bool
}

// ── UserTriggerOrders ─────────────────────────────────────────────────────────
// Resting trigger orders grouped market → orderbook → array (insertion order).
module UserTriggerOrders: {
  @genType.opaque
  type t

  @genType
  let make: unit => t

  @genType
  let get: (t, ~marketPubkey: Shared.pubkeyStr, ~orderbookId: Shared.orderBookId) => option<array<triggerOrder>>
  @genType
  let getByMarket: (t, ~marketPubkey: Shared.pubkeyStr) => option<Dict.t<array<triggerOrder>>>
  // Every resting trigger order across all markets/orderbooks.
  @genType
  let all: t => array<triggerOrder>
  @genType
  let getById: (t, ~triggerOrderId: string) => option<triggerOrder>

  @genType
  let insert: (t, triggerOrder) => unit
  // Remove (and return) the trigger order with this id, if resting.
  @genType
  let remove: (t, ~triggerOrderId: string) => option<triggerOrder>

  @genType
  let clear: t => unit
  @genType
  let isEmpty: t => bool
  @genType
  let size: t => int
}

// ── Snapshot seeding ──────────────────────────────────────────────────────────
// Split a user snapshot's orders into both containers: fully-filled limit orders
// (zero remaining) are skipped; trigger orders are kept as-is.
@genType
let ofSnapshotOrders: array<Order.UserSnapshotOrder.t> => (UserOpenLimitOrders.t, UserTriggerOrders.t)
