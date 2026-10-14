open RescriptBun.Test
open RescriptBun.Test.Expect

// Runtime tests for the @solana-program/token binding — run the compiled .res.mjs
// under Bun to prove the JS export names (TOKEN_PROGRAM_ADDRESS,
// ASSOCIATED_TOKEN_PROGRAM_ADDRESS, findAssociatedTokenPda), the seed-record arg
// shape, and the [address, bump] return tuple.

// Real mainnet addresses used as known PDA inputs.
let ownerAddress = "So11111111111111111111111111111111111111112"
let usdcMint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
// Canonical ATA for the owner+mint above with the SPL Token program (pins arg order).
let knownAta = "DHe62eeQVEnNK7vg5xUpDkJm7tuqHadjhvmPRFBG9UPo"
let knownBump = 254

let addressLength = address => SolanaKit.addressToString(address)->String.length

let seedsFor = (ownerStr, mintStr): SolanaProgramToken.associatedTokenSeeds => {
  owner: SolanaKit.address(ownerStr),
  tokenProgram: SolanaProgramToken.tokenProgramAddress,
  mint: SolanaKit.address(mintStr),
}

describe("SolanaProgramToken", () => {
  test("tokenProgramAddress is a valid base58 address", () =>
    expect(addressLength(SolanaProgramToken.tokenProgramAddress))->toBeGreaterThan(30.0)
  )

  test("associatedTokenProgramAddress is a valid base58 address", () =>
    expect(addressLength(SolanaProgramToken.associatedTokenProgramAddress))->toBeGreaterThan(30.0)
  )

  testAsync("findAssociatedTokenPda returns a valid address and a bump in 0..255", async () => {
    let (ata, bump) = await SolanaProgramToken.findAssociatedTokenPda(seedsFor(ownerAddress, usdcMint))
    expect(addressLength(ata))->toBeGreaterThan(30.0)
    expect(bump)->toBeGreaterThanOrEqual(0.0)
    expect(bump)->toBeLessThanOrEqual(255.0)
  })

  testAsync("findAssociatedTokenPda matches the known ATA for SOL owner + USDC mint", async () => {
    let (ata, bump) = await SolanaProgramToken.findAssociatedTokenPda(seedsFor(ownerAddress, usdcMint))
    expect(SolanaKit.addressToString(ata))->toBe(knownAta)
    expect(bump)->toBe(knownBump)
  })
})
