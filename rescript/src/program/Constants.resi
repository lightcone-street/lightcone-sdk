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
  let initialize: int
  let createMarket: int
  let addDepositMint: int
  let mintCompleteSet: int
  let mergeCompleteSet: int
  let cancelOrder: int
  let incrementNonce: int
  let settleMarket: int
  let redeemWinnings: int
  let setPaused: int
  let setOperator: int
  let withdrawFromPosition: int
  let activateMarket: int
  let matchOrdersMulti: int
  let setAuthority: int
  let createOrderbook: int
  let whitelistDepositToken: int
  let depositToGlobal: int
  let globalToMarketDeposit: int
  let initPositionTokens: int
  let depositAndSwap: int
  let extendPositionTokens: int
  let withdrawFromGlobal: int
  let closePositionAlt: int
  let closeOrderStatus: int
  let closePositionTokenAccounts: int
  let closeOrderbookAlt: int
  let closeOrderbook: int
  let setManager: int
  let setMarketFees: int
  let setFeeReceiver: int
  let createConditionalMetadata: int
  let updateConditionalMetadata: int
  let setOracle: int
  let refreshOrderbookAlt: int
  let acceptAuthority: int
  let acceptManager: int
  let acceptOperator: int
  let setDepositTokenStatus: int
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
let maxMakers: int
