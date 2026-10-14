// ── Order type ────────────────────────────────────────────────────────────────
// Kind of order for display / classification; snake_case on the wire.
module Type: {
  @genType @spice
  type t =
    | @as("limit") @spice.as("limit") Limit
    | @as("market") @spice.as("market") Market
    | @as("split") @spice.as("split") Split
    | @as("merge") @spice.as("merge") Merge

  // Human-readable label ("Stop Limit", …).
  let label: t => string
}

// ── Limit orders ──────────────────────────────────────────────────────────────
// An open limit order. `size` is the original size (filled + remaining).
module Limit: {
  @genType
  type t = {
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
}
