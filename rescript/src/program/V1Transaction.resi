module Resources: {
  @genType
  type t = {
    computeUnitLimit: int,
    loadedAccountsDataSizeLimit: int,
    priorityFeeLamports: bigint,
    heapSize?: int,
  }
  let validate: t => result<unit, SdkError.t>
}
module Context: {
  @genType type t = {blockhash: string, lastValidBlockHeight: bigint, resources: Resources.t}
}
@genType.opaque type t
let compile: (
  ~instructions: array<SolanaKit.instruction>,
  ~payer: SolanaKit.address,
  ~context: Context.t,
) => result<t, SdkError.t>
let fromWireBytes: (Uint8Array.t, Context.t) => result<t, SdkError.t>
let context: t => Context.t
let toWireBytes: t => Uint8Array.t
let messageBytes: t => Uint8Array.t
let requiredSigners: t => array<SolanaKit.address>
let feePayer: t => option<SolanaKit.address>
let verifySignatures: t => promise<result<unit, SdkError.t>>
let acceptSignedBytes: (t, Uint8Array.t) => promise<result<t, SdkError.t>>
let sign: (t, array<SolanaKit.cryptoKeyPair>) => promise<result<t, SdkError.t>>
let signature: t => promise<result<string, SdkError.t>>
