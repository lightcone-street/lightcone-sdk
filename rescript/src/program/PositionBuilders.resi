// High-level position transaction builders: build the instruction, assemble a v0
// transaction, sign it with the client's native signer, and broadcast it. Each
// returns the transaction signature on success. `~user` (or `~operator` on the
// operator-signed ops) must be the configured native signer (it is both the
// instruction signer and the fee payer). The Rust `_ix` variants live in
// `Instructions.res`; `unsignedTx` is the `_tx` counterpart. The blockhash
// projector and the shared `signAndSend` helper are internal.

// Deposit collateral from the user's token account into their global deposit PDA.
let depositToGlobal: (
  Client.t,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
) => promise<result<string, SdkError.t>>

// Withdraw collateral from the user's global deposit PDA back to their wallet.
let withdrawFromGlobal: (
  Client.t,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
) => promise<result<string, SdkError.t>>

// Move collateral from the global deposit into a market vault, minting a complete
// set of conditional tokens into the user's position.
let globalToMarketDeposit: (
  Client.t,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~numOutcomes: int,
) => promise<result<string, SdkError.t>>

// Burn a complete set of conditional tokens and release the collateral.
let merge: (
  Client.t,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~numOutcomes: int,
) => promise<result<string, SdkError.t>>

// Redeem winning conditional tokens of `outcomeIndex` for collateral (post-resolution).
let redeemWinnings: (
  Client.t,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~outcomeIndex: int,
) => promise<result<string, SdkError.t>>

// Market-level direct deposit (mint complete set): wallet ATA → market vault,
// minting a complete conditional-token set into the position.
let deposit: (
  Client.t,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~numOutcomes: int,
) => promise<result<string, SdkError.t>>

// Withdraw conditional tokens from the position's ATA to the user's own ATA
// (`~mint` is the conditional mint).
let withdrawFromPosition: (
  Client.t,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~outcomeIndex: int,
) => promise<result<string, SdkError.t>>

// Append newly-added deposit mints' ATAs to an existing position ALT.
// Operator-signed; `depositMints` must be non-empty (Validation error otherwise).
let extendPositionTokens: (
  Client.t,
  ~operator: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
  ~numOutcomes: int,
) => promise<result<string, SdkError.t>>

// Deactivate an active position ALT, or close an already-deactivated one.
// Operator-signed; `~position` is the position PDA address itself.
let closePositionAlt: (
  Client.t,
  ~operator: SolanaKit.address,
  ~position: SolanaKit.address,
  ~market: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
) => promise<result<string, SdkError.t>>

// Close empty SPL conditional ATAs owned by a position PDA after resolution.
// Operator-signed; `depositMints` must be non-empty and `numOutcomes` in 2..=6
// (Validation error otherwise).
let closePositionTokenAccounts: (
  Client.t,
  ~operator: SolanaKit.address,
  ~market: SolanaKit.address,
  ~position: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
  ~numOutcomes: int,
) => promise<result<string, SdkError.t>>

// Wrap one instruction in an unsigned v0 message with the fee payer set (the kit
// counterpart of the Rust builders' `build_tx`). The caller appends further
// instructions if needed, sets the blockhash lifetime, and signs.
let unsignedTx: (
  ~feePayer: SolanaKit.address,
  ~instruction: SolanaKit.instruction,
) => SolanaKitTx.message
