// Live order state — the user's open limit orders + resting trigger orders from
// the WS user channel. A snapshot seeds both containers via `fromSnapshotOrders`;
// limit updates feed `Limits.upsert`/`remove`; trigger updates convert via
// `Order__Raw.TriggerUpdate.toTrigger` for `Triggers`. Decimal fields are wire
// strings; malformed values are treated as zero.

// ── Limits ────────────────────────────────────────────────────────────────────
// Open limit orders grouped market → orderbook → array (insertion order).
module Limits: {
  @genType.opaque
  type t

  @genType
  let make: unit => t

  @genType
  let get: (
    t,
    ~marketPubkey: Shared.pubkeyStr,
    ~orderbookId: Shared.orderBookId,
  ) => option<array<Order__Model.Limit.t>>
  @genType
  let getByMarket: (t, ~marketPubkey: Shared.pubkeyStr) => option<Dict.t<array<Order__Model.Limit.t>>>

  // Append without deduplication (snapshot seeding).
  @genType
  let insert: (t, Order__Model.Limit.t) => unit
  // Replace any order with the same hash, then append the update's order.
  @genType
  let upsert: (t, Order__Raw.Update.t) => unit
  // Drop the order with this hash from every market/orderbook.
  @genType
  let remove: (t, ~orderHash: string) => unit

  @genType
  let clear: t => unit
  @genType
  let isEmpty: t => bool
}

// ── Triggers ──────────────────────────────────────────────────────────────────
// Resting trigger orders grouped market → orderbook → array (insertion order).
module Triggers: {
  @genType.opaque
  type t

  @genType
  let make: unit => t

  @genType
  let get: (
    t,
    ~marketPubkey: Shared.pubkeyStr,
    ~orderbookId: Shared.orderBookId,
  ) => option<array<Order__Model.Trigger.t>>
  @genType
  let getByMarket: (t, ~marketPubkey: Shared.pubkeyStr) => option<Dict.t<array<Order__Model.Trigger.t>>>
  // Every resting trigger order across all markets/orderbooks.
  @genType
  let all: t => array<Order__Model.Trigger.t>
  @genType
  let getById: (t, ~triggerOrderId: string) => option<Order__Model.Trigger.t>

  @genType
  let insert: (t, Order__Model.Trigger.t) => unit
  // Remove (and return) the trigger order with this id, if resting.
  @genType
  let remove: (t, ~triggerOrderId: string) => option<Order__Model.Trigger.t>

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
let fromSnapshotOrders: array<Order__Raw.SnapshotOrder.t> => (Limits.t, Triggers.t)
