// Compile-guard for README.md — a compile-only file (no test blocks). If a README
// snippet drifts from the actual binding signature, `rescript build` fails here.

// Quick start: build the seeds record, await the PDA, read the tuple.
let _quickStart = async () => {
  let seeds: SolanaProgramToken.associatedTokenSeeds = {
    owner: SolanaKit.address("So11111111111111111111111111111111111111112"),
    tokenProgram: SolanaProgramToken.tokenProgramAddress,
    mint: SolanaKit.address("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
  }
  let (ata, bump) = await SolanaProgramToken.findAssociatedTokenPda(seeds)
  let _ata: string = SolanaKit.addressToString(ata)
  let _bump: int = bump
}

// The program-address constants are SolanaKit.address; read them back as strings.
let _tokenProgram: string = SolanaKit.addressToString(SolanaProgramToken.tokenProgramAddress)
let _ataProgram: string = SolanaKit.addressToString(SolanaProgramToken.associatedTokenProgramAddress)

// The seeds record requires all three fields.
let _seeds: SolanaProgramToken.associatedTokenSeeds = {
  owner: SolanaKit.address("So11111111111111111111111111111111111111112"),
  tokenProgram: SolanaProgramToken.tokenProgramAddress,
  mint: SolanaKit.address("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
}
