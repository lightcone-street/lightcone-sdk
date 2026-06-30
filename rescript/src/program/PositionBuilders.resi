// High-level position transaction builders: build the instruction, assemble a v0
// transaction, sign it with the client's native signer, and broadcast it. Each
// returns the transaction signature on success. `~user` must be the configured
// native signer (it is both the instruction signer and the fee payer). The
// blockhash projector and the shared `signAndSend` helper are internal.

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
