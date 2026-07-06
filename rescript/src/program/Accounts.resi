// On-chain account decoders. Public
// surface: the program-layer enums and one module per account row — the clean
// gentype-exported record `t` plus its `decode` entry point (taking the
// account's raw data bytes). The account sizes, low-level field readers, header
// guards, and the spice-free byte→enum `fromU8` hand-decoders are internal.

// ── Enums (decoded from a single byte, per the program's type definitions) ─────
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
// Each `decode` is length + discriminator guarded.

// Exchange — singleton exchange state.
module Exchange: {
  @genType
  type t = {
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

  let decode: Uint8Array.t => result<t, SdkError.t>
}

// Market — a prediction market. `questionId`/`conditionId` are lowercase hex
// (32 bytes → 64 chars); only the first `numOutcomes` payout numerators matter.
module Market: {
  @genType
  type t = {
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

  let decode: Uint8Array.t => result<t, SdkError.t>
}

// Orderbook — on-chain book with its ALT. `baseIndex` 0 ⇒ mintA is the base.
module Orderbook: {
  @genType
  type t = {
    market: Shared.pubkeyStr,
    mintA: Shared.pubkeyStr,
    mintB: Shared.pubkeyStr,
    lookupTable: Shared.pubkeyStr,
    baseIndex: float,
    bump: float,
  }

  let decode: Uint8Array.t => result<t, SdkError.t>
}

// Position — a user's per-market custody account.
module Position: {
  @genType
  type t = {
    owner: Shared.pubkeyStr,
    market: Shared.pubkeyStr,
    bump: float,
  }

  let decode: Uint8Array.t => result<t, SdkError.t>
}

// UserNonce — a user's mass-cancel nonce (u64 counter, kept exact).
module UserNonce: {
  @genType
  type t = {
    nonce: bigint,
  }

  let decode: Uint8Array.t => result<t, SdkError.t>
}

// OrderStatus — on-chain order status (remaining fill amounts + cancellation flag).
module OrderStatus: {
  @genType
  type t = {
    remaining: bigint,
    baseRemaining: bigint,
    isCancelled: bool,
  }

  let decode: Uint8Array.t => result<t, SdkError.t>
}

// GlobalDepositToken — a whitelisted global-deposit token (mint + whitelist
// index + active flag).
module GlobalDepositToken: {
  @genType
  type t = {
    mint: Shared.pubkeyStr,
    bump: float,
    index: float,
    active: bool,
  }

  let decode: Uint8Array.t => result<t, SdkError.t>
}
