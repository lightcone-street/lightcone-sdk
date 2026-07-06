// Read on-chain account state directly over RPC. Mirrors
// rust/examples/read_onchain.rs: fetch + decode the Exchange, Market, and
// Orderbook accounts, the user's nonce + position, and derive the related PDAs.
// No login — these are public on-chain reads (the market metadata comes from the
// REST API to locate the accounts). ReScript surface (result core); the compiled
// .res.mjs is the JS example. The TS port (ReadOnchain.ts) imports the same Rpc
// module since the on-chain read layer isn't part of the gentype facade.
let main = async () => {
  let client = Common__Example.client()
  // The user pubkey for the nonce + position reads (no signing needed).
  let keypair = await SolanaKitKeys.createKeyPairFromBytes(Common__Example.walletSecretKey())
  let user = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)

  switch await Market.Client.get(client, ~limit=1) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok({markets}) =>
    switch markets[0] {
    | None => Console.log("no markets found")
    | Some(market) =>
      switch market.orderbookPairs->Array.find(pair => pair.active)->Option.orElse(market.orderbookPairs[0]) {
      | None => Console.log("selected market has no orderbooks")
      | Some(pair) => {
          let marketAddress = SolanaKit.address(market.pubkey)
          let baseMint = SolanaKit.address(pair.base.mint)
          let quoteMint = SolanaKit.address(pair.quote.mint)
          let depositMint = SolanaKit.address(pair.quote.depositAsset)

          switch await Rpc.getExchange(client) {
          | Ok(exchange) => {
              let paused = exchange.paused ? "true" : "false"
              Console.log(`exchange: authority=${exchange.authority} operator=${exchange.operator} paused=${paused}`)
            }
          | Error(error) => Console.error(SdkError.toMessage(error))
          }

          switch await Rpc.getMarket(client, marketAddress) {
          | Ok(onchainMarket) => {
              Console.log(
                `market: id=${BigInt.toString(onchainMarket.marketId)} outcomes=${Float.toString(
                    onchainMarket.numOutcomes,
                  )} status=${Accounts.MarketStatus.toString(onchainMarket.status)}`,
              )
              let marketPda = await Rpc.marketPda(client, ~marketId=onchainMarket.marketId)
              Console.log(`market pda: ${SolanaKit.addressToString(marketPda)}`)
            }
          | Error(error) => Console.error(SdkError.toMessage(error))
          }

          switch await Rpc.getOrderbook(client, ~mintA=baseMint, ~mintB=quoteMint) {
          | Ok(onchainOrderbook) =>
            Console.log(
              `orderbook: lookup_table=${onchainOrderbook.lookupTable} base_index=${Float.toString(
                  onchainOrderbook.baseIndex,
                )} bump=${Float.toString(onchainOrderbook.bump)}`,
            )
          | Error(error) => Console.error(SdkError.toMessage(error))
          }

          switch await Rpc.getNonce(client, ~user) {
          | Ok(nonce) => Console.log(`user nonce: ${Float.toString(nonce)}`)
          | Error(error) => Console.error(SdkError.toMessage(error))
          }

          switch await Rpc.getPosition(client, ~owner=user, ~market=marketAddress) {
          | Ok(position) => Console.log(`position exists: ${Option.isSome(position) ? "true" : "false"}`)
          | Error(error) => Console.error(SdkError.toMessage(error))
          }

          let exchangePda = await Rpc.exchangePda(client)
          let positionPda = await Rpc.positionPda(client, ~owner=user, ~market=marketAddress)
          let globalDepositPda = await Rpc.globalDepositTokenPda(client, ~mint=depositMint)
          Console.log(
            `pdas: exchange=${SolanaKit.addressToString(exchangePda)} position=${SolanaKit.addressToString(
                positionPda,
              )} global_deposit=${SolanaKit.addressToString(globalDepositPda)}`,
          )
        }
      }
    }
  }
}

let _ = main()
