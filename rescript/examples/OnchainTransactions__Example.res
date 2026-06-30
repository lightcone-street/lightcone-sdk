// Builds + signs + broadcasts a sequence of on-chain transactions via the program
// builders, reporting each transaction signature. Ported from
// rust/examples/onchain_transactions.rs (which sends a batch and confirms each).
// ReScript surface (result core); the compiled .res.mjs is the JS example.
//
// No TS facade equivalent: the builders take SolanaKit (@solana/kit) `address`
// values and rely on the native signer, neither surfaced on TypeScriptApi.gen.ts.
let main = async () => {
  let client = Common__Example.client()
  let secretKey = Common__Example.walletSecretKey()
  // The native signer is both the instruction signer and the fee payer; `user`
  // MUST be its address for the signed transactions to verify.
  await Client.useNativeSigner(client, secretKey)
  let keypair = await SolanaKitKeys.createKeyPairFromBytes(secretKey)
  let user = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)

  switch await Auth.login(client) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok(_) =>
    // A recent blockhash proves the @solana/kit RPC path before we send.
    switch await Rpc.getLatestBlockhash(client) {
    | Ok(blockhash) => Console.log(`latest blockhash: ${blockhash}`)
    | Error(error) => Console.error(SdkError.toMessage(error))
    }

    switch await Market.get(client, ~limit=1) {
    | Error(error) => Console.error(SdkError.toMessage(error))
    | Ok({markets}) =>
      switch markets[0]->Option.flatMap(market => market.orderbookPairs[0]) {
      | None => Console.log("no orderbook found")
      | Some(pair) =>
        let mint = SolanaKit.address(pair.quote.depositAsset)
        let amount = 1000000n // 1 unit at 6 decimals

        let run = async (label, action: unit => promise<result<string, SdkError.t>>) =>
          switch await action() {
          | Ok(signature) => Console.log(`${label}: confirmed ${signature}`)
          | Error(error) => Console.error(`${label}: ${SdkError.toMessage(error)}`)
          }

        // Send a net-neutral deposit then withdraw, reporting each tx signature.
        await run("deposit_to_global", () => PositionBuilders.depositToGlobal(client, ~user, ~mint, ~amount))
        await run("withdraw_from_global", () =>
          PositionBuilders.withdrawFromGlobal(client, ~user, ~mint, ~amount)
        )
      }
    }
  }
}

let _ = main()
