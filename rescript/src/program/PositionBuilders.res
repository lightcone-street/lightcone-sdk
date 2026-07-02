// High-level position transaction builders: build the instruction, assemble a
// v0 transaction, sign it with the client's native signer, and broadcast it.
// Covers every Rust `Positions` builder flow — deposit_to_global,
// withdraw_from_global, global_to_market_deposit, merge, redeem_winnings, the
// market-level deposit (mint complete set), withdraw_from_position,
// extend_position_tokens, and the close ops — as that fluent API's idiomatic
// ReScript counterpart: one labeled-argument function per flow (the Rust
// `deposit()`/`withdraw()` builders' Global-vs-Market dispatch is realized by
// picking the explicit function; `Client.depositSource` never gates here).
//
// Each function returns `promise<result<string, SdkError.t>>` where the Ok value
// is the transaction signature. Signing uses the client's signing strategy
// (`Client.useNativeSigner` or `Client.useExternalSigner`); with none configured
// the call returns `Error(Signing(_))`. The Rust `_ix` variants are
// `Instructions.res`; the `_tx` variants collapse to `unsignedTx` below (any
// instruction + fee payer → unsigned message).
//
// NOTE: `~user` (or `~operator` on the operator-signed ops) is both the
// instruction's signer account AND the fee payer, so it MUST be the address of
// the client's configured signer for the resulting transaction to verify.

// Sign + broadcast with the client's signing strategy (native keypair or
// external wallet adapter) — see `Transactions.signAndSubmit`.
let signAndSend = Transactions.signAndSubmit

// Deposit collateral from the user's token account into their global deposit PDA.
let depositToGlobal = (
  client: Client.t,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
): promise<result<string, SdkError.t>> =>
  signAndSend(client, () =>
    Instructions.depositToGlobal(~programId=client.programId, ~user, ~mint, ~amount)
  )

// Withdraw collateral from the user's global deposit PDA back to their wallet.
let withdrawFromGlobal = (
  client: Client.t,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
): promise<result<string, SdkError.t>> =>
  signAndSend(client, () =>
    Instructions.withdrawFromGlobal(~programId=client.programId, ~user, ~mint, ~amount)
  )

// Move collateral from the user's global deposit into a market vault, minting a
// complete set of conditional tokens into the user's position.
let globalToMarketDeposit = (
  client: Client.t,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~numOutcomes: int,
): promise<result<string, SdkError.t>> =>
  signAndSend(client, () =>
    Instructions.globalToMarketDeposit(
      ~programId=client.programId,
      ~user,
      ~market,
      ~mint,
      ~amount,
      ~numOutcomes,
    )
  )

// Burn a complete set of conditional tokens from the position and release the
// underlying collateral back to the user's token account.
let merge = (
  client: Client.t,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~numOutcomes: int,
): promise<result<string, SdkError.t>> =>
  signAndSend(client, () =>
    Instructions.mergeCompleteSet(
      ~programId=client.programId,
      ~user,
      ~market,
      ~mint,
      ~amount,
      ~numOutcomes,
    )
  )

// Redeem winning conditional tokens of `outcomeIndex` from the position for
// collateral (after the market has resolved).
let redeemWinnings = (
  client: Client.t,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~outcomeIndex: int,
): promise<result<string, SdkError.t>> =>
  signAndSend(client, () =>
    Instructions.redeemWinnings(
      ~programId=client.programId,
      ~user,
      ~market,
      ~mint,
      ~amount,
      ~outcomeIndex,
    )
  )

// Market-level direct deposit (mint complete set): collateral moves from the
// user's wallet ATA into the market vault, minting a complete conditional-token
// set into the position. (The Rust `deposit()` builder with the Market source.)
let deposit = (
  client: Client.t,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~numOutcomes: int,
): promise<result<string, SdkError.t>> =>
  signAndSend(client, () =>
    Instructions.deposit(~programId=client.programId, ~user, ~market, ~mint, ~amount, ~numOutcomes)
  )

// Withdraw conditional tokens from the position's ATA to the user's own ATA
// (`~mint` is the conditional mint). (The Rust `withdraw()` builder with the
// Market source / `withdraw_from_position()`.)
let withdrawFromPosition = (
  client: Client.t,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~outcomeIndex: int,
): promise<result<string, SdkError.t>> =>
  signAndSend(client, () =>
    Instructions.withdrawFromPosition(
      ~programId=client.programId,
      ~user,
      ~market,
      ~mint,
      ~amount,
      ~outcomeIndex,
    )
  )

// Append newly-added deposit mints' ATAs to an existing position ALT (from
// initPositionTokens). Operator-signed: `~operator` must be the native signer.
let extendPositionTokens = (
  client: Client.t,
  ~operator: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
  ~numOutcomes: int,
): promise<result<string, SdkError.t>> =>
  switch depositMints {
  | [] => Promise.resolve(Error(SdkError.Validation("deposit_mints is required")))
  | _ =>
    signAndSend(client, () =>
      Instructions.extendPositionTokens(
        ~programId=client.programId,
        ~operator,
        ~user,
        ~market,
        ~lookupTable,
        ~depositMints,
        ~numOutcomes,
      )
    )
  }

// Deactivate an active position ALT, or close an already-deactivated one.
// Operator-signed; `~position` is the position PDA address itself.
let closePositionAlt = (
  client: Client.t,
  ~operator: SolanaKit.address,
  ~position: SolanaKit.address,
  ~market: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
): promise<result<string, SdkError.t>> =>
  signAndSend(client, () =>
    Instructions.closePositionAlt(
      ~programId=client.programId,
      ~operator,
      ~position,
      ~market,
      ~lookupTable,
    )
  )

// Close empty SPL conditional ATAs owned by a position PDA after market
// resolution (non-empty accounts are skipped by the program). Operator-signed;
// `~position` is the position PDA address itself.
let closePositionTokenAccounts = (
  client: Client.t,
  ~operator: SolanaKit.address,
  ~market: SolanaKit.address,
  ~position: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
  ~numOutcomes: int,
): promise<result<string, SdkError.t>> =>
  if Array.length(depositMints) == 0 {
    Promise.resolve(Error(SdkError.Validation("deposit_mints is required")))
  } else if numOutcomes < Constants.minOutcomes || numOutcomes > Constants.maxOutcomes {
    Promise.resolve(
      Error(SdkError.Validation(`invalid outcome count: ${Int.toString(numOutcomes)}`)),
    )
  } else {
    signAndSend(client, () =>
      Instructions.closePositionTokenAccounts(
        ~programId=client.programId,
        ~operator,
        ~market,
        ~position,
        ~depositMints,
        ~numOutcomes,
      )
    )
  }

// ── Unsigned transaction assembly ─────────────────────────────────────────────
// The kit counterpart of the Rust builders' `build_tx` (`Transaction::new_with_payer`):
// wrap one instruction in an unsigned v0 message with the fee payer set (no
// lifetime). The caller appends further instructions if needed, sets the
// blockhash lifetime, and signs with their own signer.
let unsignedTx = (~feePayer: SolanaKit.address, ~instruction: SolanaKit.instruction): SolanaKitTx.message =>
  SolanaKitTx.create({"version": 0})
  ->SolanaKitTx.setFeePayer(feePayer, _)
  ->SolanaKitTx.appendInstruction(instruction, _)
