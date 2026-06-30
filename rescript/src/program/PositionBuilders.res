// High-level position transaction builders: build the instruction, assemble a
// v0 transaction, sign it with the client's native signer, and broadcast it.
// Mirrors the Rust `Positions` builder methods (deposit_to_global,
// withdraw_from_global, global_to_market_deposit, merge, redeem_winnings) plus
// their `.sign_and_submit()` flow.
//
// Each function returns `promise<result<string, SdkError.t>>` where the Ok value
// is the transaction signature. Signing requires a native signer configured via
// `Client.useNativeSigner`; without one the call returns `Error(Signing(_))`.
//
// NOTE: `~user` is both the instruction's signer account AND the fee payer, so it
// MUST be the address of the client's configured native signer for the resulting
// transaction to verify.

// kit's `getLatestBlockhash().send()` resolves to {context, value: {blockhash,
// lastValidBlockHeight}}. `setTransactionMessageLifetimeUsingBlockhash` wants the
// inner `value` object verbatim, so we project it out. (The RPC layer types the
// response loosely as JSON.t; the runtime shape is exactly the lifetime object.)
let blockhashLifetimeOfResponse: JSON.t => SolanaKitTx.blockhashLifetime = %raw(`function (response) {
  return response.value;
}`)

// Sign + broadcast a single-instruction v0 transaction with the client's native
// signer as fee payer. `makeInstruction` is a thunk so any throwing PDA/ATA
// derivation is captured under the same error boundary as the RPC/sign calls.
let signAndSend = async (
  client: Client.t,
  makeInstruction: unit => promise<SolanaKit.instruction>,
): result<string, SdkError.t> =>
  switch client.signingStrategy {
  | None =>
    Error(
      SdkError.Signing("no native signer configured on the client; call Client.useNativeSigner first"),
    )
  | Some(Client.NativeSigner({signer})) =>
    let run = async () => {
      let instruction = await makeInstruction()
      let blockhashResponse = await SolanaKitRpc.getLatestBlockhash(client.rpc)->SolanaKitRpc.send
      let lifetime = blockhashLifetimeOfResponse(blockhashResponse)
      let message =
        SolanaKitTx.create({"version": 0})
        ->SolanaKitTx.setFeePayerSigner(signer, _)
        ->SolanaKitTx.appendInstruction(instruction, _)
        ->SolanaKitTx.setLifetimeUsingBlockhash(lifetime, _)
      let signedTransaction = await SolanaKitTx.signWithSigners(message)
      let wire = SolanaKitTx.base64Wire(signedTransaction)
      await SolanaKitRpc.sendTransaction(client.rpc, wire, {"encoding": "base64"})->SolanaKitRpc.send
    }
    switch await run() {
    | signature => Ok(signature)
    | exception JsExn(error) =>
      Error(SdkError.Other(error->JsExn.message->Option.getOr("failed to sign and send transaction")))
    }
  }

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
