@genType type applyResult = Applied | Ignored | Rejected
@genType.opaque type t
@genType type solBalanceBreakdown = {nativeLamports: bigint, canonicalWsolLamports: bigint}
@genType let make: unit => t
@genType
let applyRestSnapshot: (
  t,
  ~walletAddress: string,
  Position__Raw.DepositTokenBalancesSnapshot.t,
  ~minimumSnapshotSlot: bigint=?,
) => applyResult
@genType
let applyEvent: (
  t,
  Position__Raw.WalletDepositBalancesEvent.t,
  ~minimumSnapshotSlot: bigint=?,
) => applyResult
@genType let combinedSolBalance: t => result<string, SdkError.t>
@genType let solBalanceBreakdown: t => result<solBalanceBreakdown, SdkError.t>
@genType let nativeSolLamports: t => result<bigint, SdkError.t>
@genType let canonicalWsolLamports: t => result<bigint, SdkError.t>
@genType let walletAddress: t => option<string>
@genType let contextSlot: t => option<bigint>
@genType let nativeSolBalance: t => option<string>
@genType let balances: t => dict<Position__Raw.DepositTokenBalance.t>
@genType let clear: t => unit
let wrappedSolMint: string
