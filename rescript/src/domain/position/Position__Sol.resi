module Kind: {
  @genType type t = Split | Merge | Redeem | NativeWithdraw | Wrap | UnwrapAll
}
module Costs: {
  @genType
  type t = {
    feeLamports: bigint,
    upfrontRentLamports: bigint,
    createsCanonicalWsolAccount: bool,
    sponsored: bool,
  }
}
module Availability: {
  @genType
  type t = {
    breakdown: Position__WalletState.solBalanceBreakdown,
    displayedLamports: bigint,
    reserveLamports: bigint,
    spendableLamports: bigint,
  }
}
module Delta: {
  @genType type t = {nativeLamports: bigint, canonicalWsolLamports: bigint}
}
module Plan: {
  @genType
  type t = {
    kind: Kind.t,
    transaction: V1Transaction.t,
    costs: Costs.t,
    availability: Availability.t,
    expectedDelta: Delta.t,
  }
}
module CanonicalAccount: {
  @genType
  type t = {accountLamports: bigint, tokenAmountLamports: bigint, nativeReserveLamports: bigint}
}
let availability: (
  Position__WalletState.solBalanceBreakdown,
  Costs.t,
  ~unwrapAll: bool=?,
) => result<Availability.t, SdkError.t>
let canonicalAccount: (
  Client.t,
  SolanaKit.address,
) => promise<result<option<CanonicalAccount.t>, SdkError.t>>
let planWrap: (
  Client.t,
  ~amount: bigint,
  ~state: Position__WalletState.t,
) => promise<result<Plan.t, SdkError.t>>
let planUnwrapAll: (
  Client.t,
  ~state: Position__WalletState.t,
) => promise<result<Plan.t, SdkError.t>>
let planSplit: (
  Client.t,
  ~market: SolanaKit.address,
  ~numOutcomes: int,
  ~amount: bigint,
  ~state: Position__WalletState.t,
) => promise<result<Plan.t, SdkError.t>>
let planMerge: (
  Client.t,
  ~market: SolanaKit.address,
  ~numOutcomes: int,
  ~amount: bigint,
  ~state: Position__WalletState.t,
) => promise<result<Plan.t, SdkError.t>>
let planRedeem: (
  Client.t,
  ~market: SolanaKit.address,
  ~numOutcomes: int,
  ~outcomeIndex: int,
  ~amount: bigint,
  ~state: Position__WalletState.t,
) => promise<result<Plan.t, SdkError.t>>
let planNativeWithdrawal: (
  Client.t,
  ~recipient: SolanaKit.address,
  ~amount: bigint,
  ~state: Position__WalletState.t,
) => promise<result<Plan.t, SdkError.t>>
let submit: (Client.t, Plan.t) => promise<result<Transactions.Confirmed.t, SdkError.t>>
