// On-chain program constants — program ids, well-known programs, instruction
// opcodes, and account discriminators. Mirrors rust/src/program/constants.rs.
// This is a Pinocchio program: instructions use a SINGLE-byte opcode at data[0]
// (no Anchor 8-byte discriminator), and account data starts with an 8-byte
// discriminator we compare on read.

// ── Well-known program addresses ──────────────────────────────────────────────
let altProgram = SolanaKit.address("AddressLookupTab1e1111111111111111111111111")
let systemProgram = SolanaKit.address("11111111111111111111111111111111")
let rentSysvar = SolanaKit.address("SysvarRent111111111111111111111111111111111")
let mplTokenMetadataProgram = SolanaKit.address("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
let tokenProgram = SolanaProgramToken.tokenProgramAddress
let associatedTokenProgram = SolanaProgramToken.associatedTokenProgramAddress

// Program id for an environment (the canonical Lightcone program), honoring the
// SDK_PROGRAM_ID override via `Env`.
let programIdFor = (env: Env.t): SolanaKit.address => SolanaKit.address(Env.programId(env))

// ── Instruction opcodes (data[0]) ─────────────────────────────────────────────
module Instruction = {
  let mintCompleteSet = 3
  let mergeCompleteSet = 4
  let cancelOrder = 5
  let incrementNonce = 6
  let redeemWinnings = 8
  let depositToGlobal = 17
  let globalToMarketDeposit = 18
  let initPositionTokens = 19
  let extendPositionTokens = 21
  let withdrawFromGlobal = 22
}

// ── Account discriminators (first 8 bytes of account data) ─────────────────────
module Discriminator = {
  let exchange = [0x1e, 0xc8, 0xdc, 0x95, 0x03, 0x3d, 0x68, 0x32]
  let market = [0xdb, 0xbe, 0xd5, 0x37, 0x00, 0xe3, 0xc6, 0x9a]
  let orderStatus = [0x2e, 0x5a, 0xf1, 0x49, 0xb2, 0x68, 0x41, 0x03]
  let userNonce = [0xeb, 0x85, 0x01, 0xf3, 0x12, 0x87, 0x58, 0xe0]
  let position = [0xaa, 0xbc, 0x8f, 0xe4, 0x7a, 0x40, 0xf7, 0xd0]
  let orderbook = [0x2b, 0x22, 0x19, 0x71, 0xc3, 0x45, 0x48, 0x07]
  let globalDepositToken = [0x25, 0xbe, 0xa1, 0xe8, 0x7b, 0x92, 0x2a, 0x57]
}

// Account / order sizes (bytes).
let signedOrderSize = 233
let orderSize = 37
let signatureSize = 64
let maxOutcomes = 6
let minOutcomes = 2
