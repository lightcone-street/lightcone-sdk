// Order domain types — the order-kind enum and the user's open limit / resting
// trigger orders (the domain shapes the `Order__State` containers hold, built
// from the `Order__Raw` wire payloads). Decimal fields stay wire strings; the
// only Decimal math is `limitPrice`, with malformed strings treated as zero.

// ── Order type ────────────────────────────────────────────────────────────────
// Kind of order for display / classification; snake_case on the wire.
module Type: {
  @genType @spice
  type t =
    | @as("limit") @spice.as("limit") Limit
    | @as("market") @spice.as("market") Market
    | @as("deposit") @spice.as("deposit") Deposit
    | @as("merge") @spice.as("merge") Merge
    | @as("withdraw") @spice.as("withdraw") Withdraw
    | @as("stop_limit") @spice.as("stop_limit") StopLimit
    | @as("take_profit_limit") @spice.as("take_profit_limit") TakeProfitLimit

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

// ── Trigger orders ────────────────────────────────────────────────────────────
// A resting trigger (TP/SL) order.
module Trigger: {
  @genType
  type t = {
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
  let limitPrice: t => option<string>
}
