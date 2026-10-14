// RPC failover: automatic switch to a backup Solana RPC endpoint on infrastructure
// errors, with a 120s cooldown recovery to primary.
//
// The callers (`Rpc.getAccountData` / `getLatestBlockhash`) wrap only transport
// primitives, whose only failures are transport-level, so every error reaching
// `withFailover` is treated as an infrastructure error.

// Which endpoint is live.
module Active = {
  type t = | @as("primary") Primary | @as("backup") Backup
}

let other = (active: Active.t): Active.t =>
  switch active {
  | Primary => Backup
  | Backup => Primary
  }

// Mutable failover state held on the client.
type t = {
  mutable active: Active.t,
  // Unix ms when we last flipped to backup; None while on primary.
  mutable flippedToBackupAtMs: option<float>,
}

let cooldownMs = 120000.0
let fastRetryDelayMs = 100

// Resolve after `ms` — `setTimeout` is the stdlib global (Stdlib includes Stdlib_Global).
let sleep = (ms: int): promise<unit> =>
  Promise.make((resolve, _reject) => setTimeout(() => resolve(), ms)->ignore)

let make = (): t => {active: Primary, flippedToBackupAtMs: None}

let active = (state: t): Active.t => state.active

let toString = (active: Active.t): string =>
  switch active {
  | Primary => "primary"
  | Backup => "backup"
  }

// On backup past the cooldown → recover to primary (probe it again).
let maybeRecoverToPrimary = (state: t): unit =>
  switch (state.active, state.flippedToBackupAtMs) {
  | (Backup, Some(flippedAt)) if Date.now() -. flippedAt >= cooldownMs =>
    state.active = Primary
    state.flippedToBackupAtMs = None
  | _ => ()
  }

let flipTo = (state: t, target: Active.t): unit =>
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
  state: t,
  ~hasBackup: bool,
  ~tryOn: Active.t => promise<result<'a, SdkError.t>>,
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
