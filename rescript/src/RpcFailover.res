// RPC failover: automatic switch to a backup Solana RPC endpoint on infrastructure
// errors, with a 120s cooldown recovery to primary. Mirrors rust/src/rpc_failover.rs.
//
// The callers (`Rpc.getAccountData` / `getLatestBlockhash`) wrap only transport
// primitives, whose only failures are transport-level, so every error reaching
// `withFailover` is treated as an infrastructure error.

type activeRpc = | @as("primary") Primary | @as("backup") Backup

let other = (active: activeRpc): activeRpc =>
  switch active {
  | Primary => Backup
  | Backup => Primary
  }

// Mutable failover state held on the client.
type state = {
  mutable active: activeRpc,
  // Unix ms when we last flipped to backup; None while on primary.
  mutable flippedToBackupAtMs: option<float>,
}

let cooldownMs = 120000.0
let fastRetryDelayMs = 100

// Resolve after `ms` — `setTimeout` is the stdlib global (Stdlib includes Stdlib_Global).
let sleep = (ms: int): promise<unit> =>
  Promise.make((resolve, _reject) => setTimeout(() => resolve(), ms)->ignore)

let make = (): state => {active: Primary, flippedToBackupAtMs: None}

let active = (state: state): activeRpc => state.active

let toString = (active: activeRpc): string =>
  switch active {
  | Primary => "primary"
  | Backup => "backup"
  }

// On backup past the cooldown → recover to primary (probe it again).
let maybeRecoverToPrimary = (state: state): unit =>
  switch (state.active, state.flippedToBackupAtMs) {
  | (Backup, Some(flippedAt)) if Date.now() -. flippedAt >= cooldownMs =>
    state.active = Primary
    state.flippedToBackupAtMs = None
  | _ => ()
  }

let flipTo = (state: state, target: activeRpc): unit =>
  switch target {
  | Primary =>
    state.active = Primary
    state.flippedToBackupAtMs = None
  | Backup =>
    state.active = Backup
    state.flippedToBackupAtMs = Some(Date.now())
  }

// Execute `tryOn` with fast retry + automatic failover:
//   1. cooldown check → maybe recover to primary
//   2. try the active endpoint → success returns immediately
//   3. error → 100ms delay → try the active endpoint again
//   4. still failing + has backup → try the other endpoint:
//      success → flip state to it → Ok; failure → Err (both down; don't flip)
let withFailover = async (
  state: state,
  ~hasBackup: bool,
  ~tryOn: activeRpc => promise<result<'a, SdkError.t>>,
): result<'a, SdkError.t> => {
  maybeRecoverToPrimary(state)
  let startActive = state.active
  switch await tryOn(startActive) {
  | Ok(value) => Ok(value)
  | Error(_firstError) =>
    await sleep(fastRetryDelayMs)
    switch await tryOn(startActive) {
    | Ok(value) => Ok(value)
    | Error(retryError) =>
      if !hasBackup {
        Error(retryError)
      } else {
        switch await tryOn(other(startActive)) {
        | Ok(value) =>
          flipTo(state, other(startActive))
          Ok(value)
        | Error(backupError) => Error(backupError)
        }
      }
    }
  }
}
