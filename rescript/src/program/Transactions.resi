module Simulation: {
  @genType
  type t = {
    slot: bigint,
    unitsConsumed: option<bigint>,
    loadedAccountsDataSize: option<bigint>,
    logs: array<string>,
  }
}
module Confirmed: {
  @genType type t = {signature: string, slot: bigint}
}
let contextWithResources: (
  Client.t,
  V1Transaction.Resources.t,
) => promise<result<V1Transaction.Context.t, SdkError.t>>
let context: Client.t => promise<result<V1Transaction.Context.t, SdkError.t>>
let build: (
  Client.t,
  ~payer: SolanaKit.address,
  ~instructions: array<SolanaKit.instruction>,
) => promise<result<V1Transaction.t, SdkError.t>>
let ensureV1Supported: Client.t => promise<result<unit, SdkError.t>>
let estimateFee: (Client.t, V1Transaction.t) => promise<result<bigint, SdkError.t>>
let simulate: (Client.t, V1Transaction.t) => promise<result<Simulation.t, SdkError.t>>
let submitSigned: (Client.t, V1Transaction.t) => promise<result<string, SdkError.t>>
let signPrepared: (
  Client.t,
  V1Transaction.t,
  ~additionalSigners: array<SolanaKit.cryptoKeyPair>=?,
) => promise<result<V1Transaction.t, SdkError.t>>
let signAndSubmitPrepared: (Client.t, V1Transaction.t) => promise<result<string, SdkError.t>>
let signAndSubmitConfirmed: (Client.t, V1Transaction.t) => promise<result<Confirmed.t, SdkError.t>>
let confirm: (
  Client.t,
  string,
  ~lastValidBlockHeight: bigint=?,
  ~pollIntervalMs: int=?,
  ~maxPolls: int=?,
) => promise<result<Confirmed.t, SdkError.t>>
let signAndSubmit: (
  Client.t,
  unit => promise<SolanaKit.instruction>,
) => promise<result<string, SdkError.t>>
let signAndSubmitMessage: (Client.t, V1Transaction.t) => promise<result<string, SdkError.t>>
