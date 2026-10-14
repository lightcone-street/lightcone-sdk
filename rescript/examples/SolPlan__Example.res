// App integration helper: call only with an authenticated, signer-configured
// client and a complete matching wallet state. The caller owns UI projections.
let splitAndAwaitSnapshot = async (
  client,
  state,
  ~market,
  ~numOutcomes,
  ~amountLamports,
  ~retainProjection,
) => {
  switch await Position.Sol.planSplit(
    client,
    ~market,
    ~numOutcomes,
    ~amount=amountLamports,
    ~state,
  ) {
  | Error(error) => Error(error)
  | Ok(plan) =>
    switch await Position.Sol.submit(client, plan) {
    | Error(error) => Error(error) // Reconcile authoritative state; do not auto-resend.
    | Ok(confirmed) =>
      retainProjection(plan.expectedDelta, confirmed.slot)
      Ok(confirmed)
    }
  }
}

// Only complete snapshots can cover a pending confirmed operation.
let applyCoveringSnapshot = (state, ~walletAddress, snapshot, confirmedSlot) =>
  Position.WalletState.applyRestSnapshot(
    state,
    ~walletAddress,
    snapshot,
    ~minimumSnapshotSlot=confirmedSlot,
  )
