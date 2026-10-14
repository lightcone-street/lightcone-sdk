// Notification domain types — user notifications for market / order events.

// A market's resolution outcome, attached to market_resolved notifications.
module MarketResolution: {
  module Kind: {
    @genType @spice
    type t =
      | @as("single_winner") @spice.as("single_winner") SingleWinner
      | @as("scalar") @spice.as("scalar") Scalar
  }

  @genType @spice
  type payout = {
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("payout_numerator") payoutNumerator: float,
  }

  @genType
  type t = {
    kind: Kind.t,
    payoutDenominator: float,
    payouts: array<payout>,
    // Absent for scalar markets (no single winner); the wire sends `null`.
    singleWinningOutcome?: float,
  }
}

// Payload of a market_resolved notification.
module MarketResolved: {
  @genType
  type t = {
    marketPubkey: Shared.pubkeyStr,
    marketSlug?: string,
    marketName?: string,
    resolution?: MarketResolution.t,
  }
}

// Payload of an order_filled notification. `side` is a plain wire string;
// Decimal fields are wire strings.
module OrderFilled: {
  @genType
  type t = {
    orderHash: string,
    marketPubkey: Shared.pubkeyStr,
    side: string,
    price: string,
    filled: string,
    remaining: string,
    marketSlug?: string,
    marketName?: string,
    outcomeName?: string,
    outcomeNameLong?: string,
    outcomeIconUrlLow?: string,
    outcomeIconUrlMedium?: string,
    outcomeIconUrlHigh?: string,
  }
}

// Payload of new_market / rules_clarified notifications.
module MarketData: {
  @genType
  type t = {
    marketPubkey: Shared.pubkeyStr,
    marketSlug?: string,
    marketName?: string,
  }
}

// The notification's kind + payload.
module Kind: {
  @genType
  type t =
    | MarketResolved(MarketResolved.t)
    | OrderFilled(OrderFilled.t)
    | NewMarket(MarketData.t)
    | RulesClarified(MarketData.t)
    | Global
}

// A user notification.
@genType
type t = {
  id: string,
  kind: Kind.t,
  title: string,
  message: string,
  // ISO-8601 timestamps.
  expiresAt?: string,
  createdAt: string,
}

let isGlobal: t => bool

// The market slug associated with this notification, if any.
let marketSlug: t => option<string>
