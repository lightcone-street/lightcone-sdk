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
// set into the position.
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
// (`~mint` is the conditional mint).
// Validate against the current on-chain market before deriving or signing.
let withdrawConditionalFromPosition = async (
  client: Client.t,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMint: SolanaKit.address,
  ~amount: bigint,
  ~outcomeIndex: int,
): result<string, SdkError.t> => {
  switch await Rpc.getMarket(client, market) {
  | Error(error) => Error(error)
  | Ok(info) if outcomeIndex < 0 || Int.toFloat(outcomeIndex) >= info.numOutcomes =>
    Error(SdkError.Validation("outcome index is outside this market"))
  | Ok(_) =>
    await signAndSend(client, () =>
      Instructions.withdrawConditionalFromPosition(
        ~programId=client.programId,
        ~user,
        ~market,
        ~depositMint,
        ~amount,
        ~outcomeIndex,
      )
    )
  }
}
let withdrawFromPosition = withdrawConditionalFromPosition

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

let unsignedTx = (~feePayer, ~instruction, ~context) =>
  V1Transaction.compile(~payer=feePayer, ~instructions=[instruction], ~context)

// Read the market and registration entries once, then preserve ascending GDT
// order. This is safe for initial, repeated, partial, and additional-group calls.
let initPositionTokensIx = async (client: Client.t, ~payer, ~user, ~market, ~depositMints) => {
  switch await Rpc.getMarket(client, market) {
  | Error(error) => Error(error)
  | Ok(info) =>
    let entries = await Promise.all(
      depositMints->Array.map(async mint =>
        (await Rpc.getGlobalDepositToken(client, ~mint))->Result.map(entry => (mint, entry.index))
      ),
    )
    switch entries->Array.findMap(result =>
      switch result {
      | Error(error) => Some(error)
      | Ok(_) => None
      }
    ) {
    | Some(error) => Error(error)
    | None =>
      let ordered =
        entries
        ->Array.filterMap(result => result->Result.mapOr(None, value => Some(value)))
        ->Array.toSorted(((_, a), (_, b)) => Float.compare(a, b))
      let indices = Dict.make()
      ordered->Array.forEach(((_, index)) => Dict.set(indices, Float.toString(index), true))
      if Dict.toArray(indices)->Array.length != Array.length(ordered) {
        Error(SdkError.Program(ProgramError.InvalidDepositMintOrder))
      } else {
        let sorted = ordered->Array.map(((mint, _)) => mint)
        await Instructions.initPositionTokens(
          ~programId=client.programId,
          ~payer,
          ~user,
          ~market,
          ~depositMints=sorted,
          ~numOutcomes=Float.toInt(info.numOutcomes),
        )
      }
    }
  }
}

let initPositionTokens = async (client: Client.t, ~payer, ~user, ~market, ~depositMints) => {
  switch await initPositionTokensIx(client, ~payer, ~user, ~market, ~depositMints) {
  | Error(error) => Error(error)
  | Ok(instruction) =>
    switch await Transactions.build(client, ~payer, ~instructions=[instruction]) {
    | Error(error) => Error(error)
    | Ok(transaction) => await Transactions.signAndSubmitPrepared(client, transaction)
    }
  }
}
