// RPC failover state machine + executor. Mirrors rust/src/rpc_failover.rs.

@genType
type activeRpc = | @as("primary") Primary | @as("backup") Backup

// Mutable failover state held on the client (Client.t). Plain data — exported structurally
// (which endpoint is live + when it flipped), not opaque.
@genType
type state = {
  mutable active: activeRpc,
  mutable flippedToBackupAtMs: option<float>,
}

let other: activeRpc => activeRpc
let cooldownMs: float
let fastRetryDelayMs: int

let make: unit => state
let active: state => activeRpc
let toString: activeRpc => string
let maybeRecoverToPrimary: state => unit
let flipTo: (state, activeRpc) => unit

// Run `tryOn` against the active endpoint with a 100ms fast retry, then fail over
// to the backup (flipping state on success). Every error is treated as infra.
let withFailover: (
  state,
  ~hasBackup: bool,
  ~tryOn: activeRpc => promise<result<'a, SdkError.t>>,
) => promise<result<'a, SdkError.t>>
