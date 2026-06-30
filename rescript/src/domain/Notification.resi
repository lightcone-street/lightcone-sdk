// Notification domain — user notifications for market / order events. Public
// surface: the gentype payload types, the `MarketResolution` sub-tree, the
// `notificationKind` / `notification` types, the `notificationDecode` entry point
// (reused by the WS `NotificationPush` decoder in `ws/Messages.res`), the
// `isGlobal` / `marketSlug` accessors, and the `fetch` / `dismiss` client calls.
// The hand-written field / payload decoders are internal.

// ── Market resolution ────────────────────────────────────────────────────────
module MarketResolution: {
  module Kind: {
    @genType
    type t =
      | @as("single_winner") SingleWinner
      | @as("scalar") Scalar
  }

  @genType
  type payout = {
    outcomeIndex: float,
    payoutNumerator: float,
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

// ── Notification payloads (one per NotificationKind variant) ─────────────────
@genType
type marketResolvedData = {
  marketPubkey: Shared.pubkeyStr,
  marketSlug?: string,
  marketName?: string,
  resolution?: MarketResolution.t,
}

@genType
type orderFilledData = {
  orderHash: string,
  marketPubkey: Shared.pubkeyStr,
  // Plain wire string (the Rust keeps `side` as a `String`, not the `Side` enum).
  side: string,
  // Decimal wire strings (no precision loss).
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

@genType
type marketData = {
  marketPubkey: Shared.pubkeyStr,
  marketSlug?: string,
  marketName?: string,
}

// ── Notification kind + notification ─────────────────────────────────────────
@genType
type notificationKind =
  | MarketResolved(marketResolvedData)
  | OrderFilled(orderFilledData)
  | NewMarket(marketData)
  | RulesClarified(marketData)
  | Global

@genType
type notification = {
  id: string,
  kind: notificationKind,
  title: string,
  message: string,
  // ISO-8601 timestamps, kept as strings (matching the Rust `String` fields).
  expiresAt?: string,
  createdAt: string,
}

// Decode one notification object (adjacently-tagged + flattened kind). Reused by
// the WS `NotificationPush` decoder.
let notificationDecode: JSON.t => result<notification, Spice.decodeError>

// ── Helpers (mirror the Rust `Notification` methods) ─────────────────────────
let isGlobal: notification => bool

// The market slug associated with this notification, if any.
let marketSlug: notification => option<string>

// ── Client functions ─────────────────────────────────────────────────────────
// Fetch all notifications for the authenticated user.
let fetch: (Client.t, ~cookieHeader: string=?) => promise<result<array<notification>, SdkError.t>>

// Dismiss a single notification by id.
let dismiss: (Client.t, ~notificationId: string) => promise<result<unit, SdkError.t>>
