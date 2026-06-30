// On-chain program constants — well-known program addresses, the per-environment
// program id, instruction opcodes, account discriminators, and account/order
// sizes. Every binding here is a public named constant (no internal helpers).

// ── Well-known program addresses ──────────────────────────────────────────────
let altProgram: SolanaKit.address
let systemProgram: SolanaKit.address
let rentSysvar: SolanaKit.address
let mplTokenMetadataProgram: SolanaKit.address
let tokenProgram: SolanaKit.address
let associatedTokenProgram: SolanaKit.address

// The canonical Lightcone program id for an environment (honors SDK_PROGRAM_ID).
let programIdFor: Env.t => SolanaKit.address

// ── Instruction opcodes (data[0]) ─────────────────────────────────────────────
module Instruction: {
  let mintCompleteSet: int
  let mergeCompleteSet: int
  let cancelOrder: int
  let incrementNonce: int
  let redeemWinnings: int
  let depositToGlobal: int
  let globalToMarketDeposit: int
  let initPositionTokens: int
  let extendPositionTokens: int
  let withdrawFromGlobal: int
}

// ── Account discriminators (first 8 bytes of account data) ─────────────────────
module Discriminator: {
  let exchange: array<int>
  let market: array<int>
  let orderStatus: array<int>
  let userNonce: array<int>
  let position: array<int>
  let orderbook: array<int>
  let globalDepositToken: array<int>
}

// ── Account / order sizes (bytes) ─────────────────────────────────────────────
let signedOrderSize: int
let orderSize: int
let signatureSize: int
let maxOutcomes: int
let minOutcomes: int
