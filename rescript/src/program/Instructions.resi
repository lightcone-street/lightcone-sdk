// Instruction builders for the Lightcone Pinocchio program. Each returns a
// fully-assembled, async `SolanaKit.instruction`. The data-packing,
// account-meta, and ATA-derivation helpers are internal.

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

// Deposit / MintCompleteSet (opcode 3) — market-level direct deposit from the
// user's wallet ATA (mints a complete conditional-token set into the position).
let deposit: (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~numOutcomes: int,
) => promise<SolanaKit.instruction>

// WithdrawFromPosition (opcode 11) — position ATA → user ATA (`~mint` is the
// conditional mint).
let withdrawFromPosition: (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~outcomeIndex: int,
) => promise<SolanaKit.instruction>

// IncrementNonce (opcode 6).
let incrementNonce: (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
) => promise<SolanaKit.instruction>

// ExtendPositionTokens (opcode 21) — operator-signed; appends newly-added deposit
// mints' ATAs to an existing position ALT. `depositMints` must be non-empty.
let extendPositionTokens: (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
  ~numOutcomes: int,
) => promise<SolanaKit.instruction>

// ClosePositionAlt (opcode 23) — operator-signed; deactivates or closes a
// position ALT. `~position` is the position PDA address (passed through).
let closePositionAlt: (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~position: SolanaKit.address,
  ~market: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
) => promise<SolanaKit.instruction>

// ClosePositionTokenAccounts (opcode 25) — operator-signed; closes empty SPL
// conditional ATAs owned by a position PDA after resolution. `~position` is the
// position PDA address; `depositMints` must be non-empty, `numOutcomes` in 2..=6.
let closePositionTokenAccounts: (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~market: SolanaKit.address,
  ~position: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
  ~numOutcomes: int,
) => promise<SolanaKit.instruction>

// ═══ Operator / admin / lifecycle instructions ══════════════════════════════
// Builders that validate inputs return `result` (Validation errors); the rest
// are total.

// ── Signed-order params (matching instructions) ───────────────────────────────
// The payload carries no embedded signature, so params pair each order with its
// 64-byte signature explicitly.
module SignedOrder: {
  type t = {order: OrderPayload.t, signature: Uint8Array.t}
}

// One maker fill in a MatchOrdersMulti instruction.
module MatchMaker: {
  type t = {
    order: SignedOrder.t,
    makerFillAmount: bigint,
    takerFillAmount: bigint,
    // Full fills skip the maker's order-status account.
    isFullFill: bool,
  }
}

// One maker fill in a DepositAndSwap instruction.
module SwapMaker: {
  type t = {
    order: SignedOrder.t,
    makerFillAmount: bigint,
    takerFillAmount: bigint,
    isFullFill: bool,
    // Deposit from global (vs swapping existing conditional tokens).
    isDeposit: bool,
    // Only read when `isDeposit` is set.
    depositMint: SolanaKit.address,
  }
}

// Initialize (opcode 0) — creates the exchange singleton.
let initialize: (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
) => promise<SolanaKit.instruction>

// CreateMarket (opcode 1) — manager-only; validates outcome count + fee pair.
let createMarket: (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~marketId: bigint,
  ~numOutcomes: int,
  ~oracle: SolanaKit.address,
  ~questionId: Uint8Array.t,
  ~makerFeeBps: int,
  ~takerFeeBps: int,
) => promise<result<SolanaKit.instruction, SdkError.t>>

// AddDepositMint (opcode 2) — manager-only vault + conditional mint setup.
let addDepositMint: (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMint: SolanaKit.address,
  ~numOutcomes: int,
) => promise<result<SolanaKit.instruction, SdkError.t>>

// CancelOrder (opcode 5) — operator-side on-chain order-status cancellation.
let cancelOrder: (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~market: SolanaKit.address,
  ~order: OrderPayload.t,
  ~signature: Uint8Array.t,
) => promise<SolanaKit.instruction>

// SettleMarket (opcode 7) — oracle-only; numerators validated (count, sign, sum).
let settleMarket: (
  ~programId: SolanaKit.address,
  ~oracle: SolanaKit.address,
  ~marketId: bigint,
  ~payoutNumerators: array<int>,
) => promise<result<SolanaKit.instruction, SdkError.t>>

// Winner-takes-all payout numerators (1 at the winner, 0 elsewhere).
let winnerTakesAllNumerators: (
  ~winningOutcome: int,
  ~numOutcomes: int,
) => result<array<int>, SdkError.t>

// SetPaused (opcode 9) — admin pause/unpause.
let setPaused: (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
  ~paused: bool,
) => promise<SolanaKit.instruction>

// Role proposals (the change lands when the incoming holder accepts).
let setOperator: (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
  ~newOperator: SolanaKit.address,
) => promise<SolanaKit.instruction>
let setAuthority: (
  ~programId: SolanaKit.address,
  ~currentAuthority: SolanaKit.address,
  ~newAuthority: SolanaKit.address,
) => promise<SolanaKit.instruction>
let setManager: (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
  ~newManager: SolanaKit.address,
) => promise<SolanaKit.instruction>

// Role acceptance (opcodes 35–37).
let acceptAuthority: (
  ~programId: SolanaKit.address,
  ~incomingRole: SolanaKit.address,
) => promise<SolanaKit.instruction>
let acceptManager: (
  ~programId: SolanaKit.address,
  ~incomingRole: SolanaKit.address,
) => promise<SolanaKit.instruction>
let acceptOperator: (
  ~programId: SolanaKit.address,
  ~incomingRole: SolanaKit.address,
) => promise<SolanaKit.instruction>

// ActivateMarket (opcode 12) — manager: Pending → Active.
let activateMarket: (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~marketId: bigint,
) => promise<SolanaKit.instruction>

// SetOracle (opcode 33) — authority-only; rejects the zero pubkey.
let setOracle: (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
  ~market: SolanaKit.address,
  ~newOracle: SolanaKit.address,
) => promise<result<SolanaKit.instruction, SdkError.t>>

// SetMarketFees (opcode 29) — manager-only, one or more markets per instruction.
module MarketFeeUpdate: {
  type t = {
    market: SolanaKit.address,
    makerFeeBps: int,
    takerFeeBps: int,
  }
}

let setMarketFees: (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~updates: array<MarketFeeUpdate.t>,
) => promise<result<SolanaKit.instruction, SdkError.t>>

// SetFeeReceiver (opcode 30) — authority-only; the with-ATAs variant appends the
// receiver quote-ATA creation block (same opcode + data).
let setFeeReceiver: (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
  ~newFeeReceiver: SolanaKit.address,
) => promise<result<SolanaKit.instruction, SdkError.t>>
let setFeeReceiverWithAtas: (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
  ~newFeeReceiver: SolanaKit.address,
  ~quoteMints: array<SolanaKit.address>,
) => promise<result<SolanaKit.instruction, SdkError.t>>

// WhitelistDepositToken (opcode 16) / SetDepositTokenStatus (opcode 38).
let whitelistDepositToken: (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
  ~mint: SolanaKit.address,
) => promise<SolanaKit.instruction>
let setDepositTokenStatus: (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~active: bool,
) => promise<SolanaKit.instruction>

// Conditional metadata create/update (opcodes 31/32, MPL token metadata).
// Validates name ≤32 / symbol ≤10 / uri ≤200 bytes.
let createConditionalMetadata: (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMint: SolanaKit.address,
  ~outcomeIndex: int,
  ~name: string,
  ~symbol: string,
  ~uri: string,
) => promise<result<SolanaKit.instruction, SdkError.t>>
let updateConditionalMetadata: (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMint: SolanaKit.address,
  ~outcomeIndex: int,
  ~name: string,
  ~symbol: string,
  ~uri: string,
) => promise<result<SolanaKit.instruction, SdkError.t>>

// MatchOrdersMulti (opcode 13) — operator-only taker↔makers match (1–5 makers).
let matchOrdersMulti: (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~feeReceiver: SolanaKit.address,
  ~takerOrder: SignedOrder.t,
  ~takerIsFullFill: bool,
  ~makers: array<MatchMaker.t>,
) => promise<result<SolanaKit.instruction, SdkError.t>>

// CreateOrderbook (opcode 15) — manager-only; canonicalizes the mint pair.
module OrderbookMint: {
  type t = {
    mint: SolanaKit.address,
    depositMint: SolanaKit.address,
    outcomeIndex: int,
  }
}

let createOrderbook: (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mintA: OrderbookMint.t,
  ~mintB: OrderbookMint.t,
  ~baseIndex: int,
  ~feeReceiver: SolanaKit.address,
  ~recentSlot: bigint,
) => promise<result<SolanaKit.instruction, SdkError.t>>

// RefreshOrderbookAlt (opcode 34) — manager-only fee-receiver ATA refresh.
let refreshOrderbookAlt: (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~market: SolanaKit.address,
  ~orderbook: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~feeReceiver: SolanaKit.address,
) => promise<SolanaKit.instruction>

// DepositToGlobal with the user-deposit-ALT create/extend block (opcode 17).
module DepositToGlobalAltContext: {
  type t =
    | Create({recentSlot: bigint})
    | Extend({lookupTable: SolanaKit.address})
}

let depositToGlobalWithAlt: (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~altContext: DepositToGlobalAltContext.t,
) => promise<SolanaKit.instruction>

// DepositAndSwap (opcode 20) — unified deposit + swap execution (1–5 makers).
let depositAndSwap: (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~feeReceiver: SolanaKit.address,
  ~takerOrder: SignedOrder.t,
  ~takerIsFullFill: bool,
  ~takerIsDeposit: bool,
  ~takerDepositMint: SolanaKit.address,
  ~numOutcomes: int,
  ~makers: array<SwapMaker.t>,
) => promise<result<SolanaKit.instruction, SdkError.t>>

// CloseOrderStatus (opcode 24) — operator-only.
let closeOrderStatus: (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~orderHash: Uint8Array.t,
) => promise<SolanaKit.instruction>

// CloseOrderbookAlt (opcode 26) / CloseOrderbook (opcode 27) — operator-only.
let closeOrderbookAlt: (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~orderbook: SolanaKit.address,
  ~market: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
) => promise<SolanaKit.instruction>
let closeOrderbook: (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~orderbook: SolanaKit.address,
  ~market: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
) => promise<SolanaKit.instruction>
