// Binding to @solana-program/token (Codama-generated SPL Token client) — used for
// associated-token-account (ATA) derivation and the well-known program addresses
// that @solana/kit doesn't bundle. Peer-compatible with @solana/kit 6.x.

@module("@solana-program/token")
external tokenProgramAddress: SolanaKit.address = "TOKEN_PROGRAM_ADDRESS"
@module("@solana-program/token")
external associatedTokenProgramAddress: SolanaKit.address = "ASSOCIATED_TOKEN_PROGRAM_ADDRESS"

// findAssociatedTokenPda(seeds) — async (SHA-256 PDA), returns [ata, bump].
type associatedTokenSeeds = {
  owner: SolanaKit.address,
  tokenProgram: SolanaKit.address,
  mint: SolanaKit.address,
}
@module("@solana-program/token")
external findAssociatedTokenPda: associatedTokenSeeds => promise<(SolanaKit.address, int)> =
  "findAssociatedTokenPda"
