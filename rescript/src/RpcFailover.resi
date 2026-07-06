// RPC failover state machine + executor.

// Which endpoint is live.
module Active: {
  @genType
  type t = | @as("primary") Primary | @as("backup") Backup
}

// Mutable failover state held on the client (Client.t). Plain data — exported structurally
// (which endpoint is live + when it flipped), not opaque.
@genType
type t = {
  mutable active: Active.t,
  mutable flippedToBackupAtMs: option<float>,
}

let other: Active.t => Active.t
let cooldownMs: float
let fastRetryDelayMs: int

let make: unit => t
let active: t => Active.t
let toString: Active.t => string
let maybeRecoverToPrimary: t => unit
let flipTo: (t, Active.t) => unit

// Run `tryOn` against the active endpoint with a 100ms fast retry, then fail over
// to the backup (flipping state on success). Every error is treated as infra.
let withFailover: (
  t,
  ~hasBackup: bool,
  ~tryOn: Active.t => promise<result<'a, SdkError.t>>,
) => promise<result<'a, SdkError.t>>
