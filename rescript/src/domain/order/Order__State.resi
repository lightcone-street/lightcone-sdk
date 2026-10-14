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
  let getByMarket: (
    t,
    ~marketPubkey: Shared.pubkeyStr,
  ) => option<Dict.t<array<Order__Model.Limit.t>>>

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

@genType
let fromSnapshotOrders: array<Order__Raw.SnapshotOrder.t> => Limits.t
