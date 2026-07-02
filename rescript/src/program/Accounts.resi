// On-chain account decoders (mirror rust/src/program/accounts.rs). Public
// surface: the program-layer enums, the clean gentype-exported account records,
// and the `decode*` entry points (each takes the account's raw data bytes). The
// account sizes, low-level field readers, header guards, and the spice-free
// byte→enum `fromU8` hand-decoders are internal.

// ── Enums (decoded from a single byte; mirror rust/src/program/types.rs) ───────
module MarketStatus: {
  @genType
  type t =
    | @as("Pending") Pending
    | @as("Active") Active
    | @as("Resolved") Resolved
    | @as("Cancelled") Cancelled

  let toString: t => string
}

module PendingRoleKind: {
  @genType
  type t =
    | @as("None") None
    | @as("Authority") Authority
    | @as("Manager") Manager
    | @as("Operator") Operator
}

// ── Domain records (the clean, gentype-exported account shapes) ────────────────

// Exchange — singleton exchange state.
@genType
type exchange = {
  authority: Shared.pubkeyStr,
  operator: Shared.pubkeyStr,
  manager: Shared.pubkeyStr,
  marketCount: bigint,
  paused: bool,
  bump: float,
  depositTokenCount: float,
  feeReceiver: Shared.pubkeyStr,
  pendingRole: Shared.pubkeyStr,
  pendingRoleKind: PendingRoleKind.t,
}

// Market — a prediction market. `questionId`/`conditionId` are lowercase hex
// (32 bytes → 64 chars); only the first `numOutcomes` payout numerators matter.
@genType
type market = {
  marketId: bigint,
  numOutcomes: float,
  status: MarketStatus.t,
  bump: float,
  makerFeeBps: float,
  takerFeeBps: float,
  oracle: Shared.pubkeyStr,
  questionId: string,
  conditionId: string,
  payoutNumerators: array<float>,
  payoutDenominator: float,
}

// Orderbook — on-chain book with its ALT. `baseIndex` 0 ⇒ mintA is the base.
@genType
type orderbook = {
  market: Shared.pubkeyStr,
  mintA: Shared.pubkeyStr,
  mintB: Shared.pubkeyStr,
  lookupTable: Shared.pubkeyStr,
  baseIndex: float,
  bump: float,
}

// Position — a user's per-market custody account.
@genType
type position = {
  owner: Shared.pubkeyStr,
  market: Shared.pubkeyStr,
  bump: float,
}

// UserNonce — a user's mass-cancel nonce (u64 counter, kept exact).
@genType
type userNonce = {
  nonce: bigint,
}

// ── Decoding (length + discriminator guarded) ─────────────────────────────────
let decodeExchange: Uint8Array.t => result<exchange, SdkError.t>
let decodeMarket: Uint8Array.t => result<market, SdkError.t>
let decodeOrderbook: Uint8Array.t => result<orderbook, SdkError.t>
let decodePosition: Uint8Array.t => result<position, SdkError.t>
let decodeUserNonce: Uint8Array.t => result<userNonce, SdkError.t>

// On-chain order status (remaining fill amounts + cancellation flag).
@genType
type orderStatus = {
  remaining: bigint,
  baseRemaining: bigint,
  isCancelled: bool,
}

let decodeOrderStatus: Uint8Array.t => result<orderStatus, SdkError.t>

// A whitelisted global-deposit token (mint + whitelist index + active flag).
@genType
type globalDepositToken = {
  mint: Shared.pubkeyStr,
  bump: float,
  index: float,
  active: bool,
}

let decodeGlobalDepositToken: Uint8Array.t => result<globalDepositToken, SdkError.t>
