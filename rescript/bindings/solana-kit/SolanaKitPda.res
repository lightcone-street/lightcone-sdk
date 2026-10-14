// `SolanaKit.Pda` — program-derived addresses. Async (SHA-256 via WebCrypto).
// Seeds are raw byte slices; returns `(pda, bump)`.

@module("@solana/kit")
external getProgramDerivedAddress: SolanaKit.pdaSeedsInput => promise<(SolanaKit.address, int)> =
  "getProgramDerivedAddress"
