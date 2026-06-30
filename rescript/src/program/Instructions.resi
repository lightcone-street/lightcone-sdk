// Instruction builders for the Lightcone Pinocchio program (mirrors
// rust/src/program/instructions.rs). Each returns a fully-assembled, async
// `SolanaKit.instruction`. The data-packing, account-meta, and ATA-derivation
// helpers are internal.

// DepositToGlobal (opcode 17).
let depositToGlobal: (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
) => promise<SolanaKit.instruction>

// WithdrawFromGlobal (opcode 22).
let withdrawFromGlobal: (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
) => promise<SolanaKit.instruction>

// GlobalToMarketDeposit (opcode 18).
let globalToMarketDeposit: (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~numOutcomes: int,
) => promise<SolanaKit.instruction>

// MergeCompleteSet (opcode 4).
let mergeCompleteSet: (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~numOutcomes: int,
) => promise<SolanaKit.instruction>

// RedeemWinnings (opcode 8).
let redeemWinnings: (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~outcomeIndex: int,
) => promise<SolanaKit.instruction>

// InitPositionTokens (opcode 19) — permissionless (payer may differ from user).
let initPositionTokens: (
  ~programId: SolanaKit.address,
  ~payer: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
  ~recentSlot: bigint,
  ~numOutcomes: int,
) => promise<SolanaKit.instruction>

// IncrementNonce (opcode 6).
let incrementNonce: (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
) => promise<SolanaKit.instruction>
