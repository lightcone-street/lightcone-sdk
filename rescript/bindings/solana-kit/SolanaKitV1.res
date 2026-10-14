// Official Kit v1 wire codecs. Account compilation stays in V1Transaction so
// merged privileges and raw-address ordering match Rust exactly.
type header = {
  numSignerAccounts: int,
  numReadonlySignerAccounts: int,
  numReadonlyNonSignerAccounts: int,
}
@tag("kind") type configValue = | @as("u32") U32({value: int}) | @as("u64") U64({value: bigint})
type instructionHeader = {
  programAccountIndex: int,
  numInstructionAccounts: int,
  numInstructionDataBytes: int,
}
type instructionPayload = {instructionAccountIndices: array<int>, instructionData: Uint8Array.t}
type compiled = {
  version: int,
  header: header,
  lifetimeToken: string,
  configMask: int,
  configValues: array<configValue>,
  numInstructions: int,
  numStaticAccounts: int,
  staticAccounts: array<SolanaKit.address>,
  instructionHeaders: array<instructionHeader>,
  instructionPayloads: array<instructionPayload>,
}
type transaction = {messageBytes: Uint8Array.t, signatures: dict<Null.t<Uint8Array.t>>}
@module("@solana/kit")
external messageEncoder: unit => SolanaKit.encoder<compiled> =
  "getCompiledTransactionMessageEncoder"
@module("@solana/kit")
external messageDecoder: unit => SolanaKit.decoder<compiled> =
  "getCompiledTransactionMessageDecoder"
@module("@solana/kit")
external transactionEncoder: unit => SolanaKit.encoder<transaction> = "getTransactionEncoder"
@module("@solana/kit")
external transactionDecoder: unit => SolanaKit.decoder<transaction> = "getTransactionDecoder"
