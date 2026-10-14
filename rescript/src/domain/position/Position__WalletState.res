// Complete snapshots replace state. Component slots are observations, not a
// global sequence: only explicit complete-snapshot floors gate freshness.
type applyResult = Applied | Ignored | Rejected
type t = {
  mutable walletAddress: option<string>,
  mutable contextSlot: option<bigint>,
  mutable balances: dict<Position__Raw.DepositTokenBalance.t>,
  mutable nativeSolBalance: option<string>,
}
let wrappedSolMint = "So11111111111111111111111111111111111111112"
let make = () => {
  walletAddress: None,
  contextSlot: None,
  balances: Dict.make(),
  nativeSolBalance: None,
}
let matches = (state, wallet) =>
  state.walletAddress == Some(wallet) &&
  state.contextSlot->Option.isSome &&
  state.nativeSolBalance->Option.isSome
let applyRestSnapshot = (
  state,
  ~walletAddress,
  snapshot: Position__Raw.DepositTokenBalancesSnapshot.t,
  ~minimumSnapshotSlot=?,
) => {
  if minimumSnapshotSlot->Option.mapOr(false, floor => snapshot.contextSlot < floor) {
    Ignored
  } else {
    state.walletAddress = Some(walletAddress)
    state.contextSlot = Some(snapshot.contextSlot)
    state.balances = snapshot.balances->Dict.toArray->Dict.fromArray
    state.nativeSolBalance = Some(snapshot.nativeSolBalance)
    Applied
  }
}
let applyEvent = (
  state,
  event: Position__Raw.WalletDepositBalancesEvent.t,
  ~minimumSnapshotSlot=?,
) =>
  switch event {
  | Snapshot(snapshot) =>
    applyRestSnapshot(
      state,
      ~walletAddress=snapshot.walletAddress,
      {
        contextSlot: snapshot.contextSlot,
        balances: snapshot.balances,
        nativeSolBalance: snapshot.nativeSolBalance,
      },
      ~minimumSnapshotSlot?,
    )
  | BalanceUpdate(update) if matches(state, update.walletAddress) =>
    switch Decimal.fromString(update.balance.idle) {
    | value if !Decimal.isFinite(value) || Decimal.lt(value, Decimal.fromInt(0)) => Rejected
    | value =>
      if Decimal.isZero(value) {
        Dict.delete(state.balances, update.balance.mint)
      } else {
        Dict.set(state.balances, update.balance.mint, update.balance)
      }
      state.contextSlot = Some(update.contextSlot)
      Applied
    | exception _ => Rejected
    }
  | NativeSolBalanceUpdate(update) if matches(state, update.walletAddress) =>
    switch Position__Raw.NativeSolBalance.t_decode(JSON.String(update.nativeSolBalance)) {
    | Error(_) => Rejected
    | Ok(value) =>
      state.nativeSolBalance = Some(value)
      state.contextSlot = Some(update.contextSlot)
      Applied
    }
  | _ => Ignored
  }
let nativeSolLamports = state =>
  switch state.nativeSolBalance {
  | None => Error(SdkError.Validation("wallet balance state is not initialized"))
  | Some(value) =>
    ExactDecimal.scaledInteger(value, 9)->Result.mapError(message => SdkError.Validation(message))
  }
let canonicalWsolLamports = state =>
  ExactDecimal.scaledInteger(
    Dict.get(state.balances, wrappedSolMint)->Option.mapOr("0", balance => balance.idle),
    9,
  )->Result.mapError(message => SdkError.Validation(message))
let combinedSolBalance = state =>
  nativeSolLamports(state)->Result.flatMap(native =>
    canonicalWsolLamports(state)->Result.flatMap(wrapped =>
      ExactDecimal.formatScaled(native + wrapped, 9)->Result.mapError(
        message => SdkError.Validation(message),
      )
    )
  )
type solBalanceBreakdown = {nativeLamports: bigint, canonicalWsolLamports: bigint}
let solBalanceBreakdown = state =>
  nativeSolLamports(state)->Result.flatMap(native =>
    canonicalWsolLamports(state)->Result.flatMap(wrapped => {
      if native > 18446744073709551615n || wrapped > 18446744073709551615n {
        Error(SdkError.Validation("SOL balance exceeds transaction u64 range"))
      } else {
        Ok({nativeLamports: native, canonicalWsolLamports: wrapped})
      }
    })
  )
let walletAddress = state => state.walletAddress
let contextSlot = state => state.contextSlot
let nativeSolBalance = state => state.nativeSolBalance
let balances = state => state.balances->Dict.toArray->Dict.fromArray
let clear = state => {
  state.walletAddress = None
  state.contextSlot = None
  state.balances = Dict.make()
  state.nativeSolBalance = None
}
