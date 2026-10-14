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
  ~depositMint: SolanaKit.address,
  ~amount: bigint,
  ~outcomeIndex: int,
) => promise<result<string, SdkError.t>>
let withdrawConditionalFromPosition: (
  Client.t,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMint: SolanaKit.address,
  ~amount: bigint,
  ~outcomeIndex: int,
) => promise<result<string, SdkError.t>>

let closePositionTokenAccounts: (
  Client.t,
  ~operator: SolanaKit.address,
  ~market: SolanaKit.address,
  ~position: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
  ~numOutcomes: int,
) => promise<result<string, SdkError.t>>

let unsignedTx: (
  ~feePayer: SolanaKit.address,
  ~instruction: SolanaKit.instruction,
  ~context: V1Transaction.Context.t,
) => result<V1Transaction.t, SdkError.t>
let initPositionTokensIx: (
  Client.t,
  ~payer: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
) => promise<result<SolanaKit.instruction, SdkError.t>>

let initPositionTokens: (
  Client.t,
  ~payer: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
) => promise<result<string, SdkError.t>>
