// On-chain position flows against the first market's first orderbook. Mirrors
// rust/examples/global_deposit_withdrawal.rs: deposit collateral into the global
// pool, move some into a market (minting a complete set of conditional tokens),
// withdraw from the global pool, then merge the conditional set back to collateral
// — a net-neutral deposit/withdraw/merge cycle.
//
// Each Position.Builders call builds the instruction, signs it with the client's
// native signer, and broadcasts it, returning the transaction signature. ReScript
// surface (result core); the compiled .res.mjs is the JS example.
//
// No TS facade equivalent: the position builders take SolanaKit (@solana/kit)
// `address` values and rely on the native signer, neither surfaced on TypeScriptApi.gen.ts.
let main = async () => {
  let client = Common__Example.client()
  let secretKey = Common__Example.walletSecretKey()
  // The native signer is both the instruction signer and the fee payer; `user`
  // MUST be its address for the signed transactions to verify.
  await Client.useNativeSigner(client, secretKey)
  let keypair = await SolanaKitKeys.createKeyPairFromBytes(secretKey)
  let user = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)

  switch await Auth.Client.login(client) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok(_) =>
    switch await Market.Client.get(client, ~limit=1) {
    | Error(error) => Console.error(SdkError.toMessage(error))
    | Ok({markets}) =>
      switch markets[0] {
      | None => Console.log("no markets found")
      | Some(market) =>
        switch market.orderbookPairs
        ->Array.find(pair => pair.active)
        ->Option.orElse(market.orderbookPairs[0]) {
        | None => Console.log("selected market has no orderbooks")
        | Some(pair) => {
            let marketAddress = SolanaKit.address(pair.marketPubkey)
            let mint = SolanaKit.address(pair.quote.depositAsset)
            let numOutcomes = Array.length(market.outcomes)
            let amount = 1000000n // 1 unit at 6 decimals
            let depositAmount = 2000000n // deposit extra so global has funds after the market transfer

            // Sign + broadcast one builder, logging the signature (or the error).
            let run = async (label: string, action: unit => promise<result<string, SdkError.t>>) =>
              switch await action() {
              | Ok(signature) => Console.log(`${label}: confirmed ${signature}`)
              | Error(error) => Console.error(`${label}: ${SdkError.toMessage(error)}`)
              }

            // 1. Fund the global pool with collateral.
            await run("deposit_to_global", () =>
              Position.Builders.depositToGlobal(client, ~user, ~mint, ~amount=depositAmount)
            )
            // 2. Move capital into the market (mints a complete conditional set).
            await run("global_to_market_deposit", () =>
              Position.Builders.globalToMarketDeposit(
                client,
                ~user,
                ~market=marketAddress,
                ~mint,
                ~amount,
                ~numOutcomes,
              )
            )
            // 3. Pull collateral back out of the global pool.
            await run("withdraw_from_global", () =>
              Position.Builders.withdrawFromGlobal(client, ~user, ~mint, ~amount)
            )
            // 4. Burn the conditional set, releasing collateral (closes the position).
            await run("merge", () =>
              Position.Builders.merge(
                client,
                ~user,
                ~market=marketAddress,
                ~mint,
                ~amount,
                ~numOutcomes,
              )
            )
          }
        }
      }
    }
  }
}

let _ = main()
