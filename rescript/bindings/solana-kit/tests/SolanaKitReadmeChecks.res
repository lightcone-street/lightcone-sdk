// Compile-guard for README.md snippets (no test blocks; `let _ = …`).
let _address = () => SolanaKit.address("So11111111111111111111111111111111111111112")
let _u64 = () => SolanaKitCodec.encode(SolanaKitCodec.getU64Encoder(), 1n)
let _hex = (bytes: Uint8Array.t): string => SolanaKitCodec.decode(SolanaKitCodec.getBase16Decoder(), bytes)
let _sign = async (keypair: SolanaKit.cryptoKeyPair, message: Uint8Array.t): Uint8Array.t =>
  await SolanaKitKeys.signBytes(keypair.privateKey, message)
let _pda = (input: SolanaKit.pdaSeedsInput): promise<(SolanaKit.address, int)> =>
  SolanaKitPda.getProgramDerivedAddress(input)
let _rpc = (url: string): SolanaKitRpc.t => SolanaKitRpc.make(url)
