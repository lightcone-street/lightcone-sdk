// A user's positions: portfolio-wide plus scoped to one market. Authenticates
// with a wallet keypair to resolve the trading wallet, then queries the public
// path-based position endpoints. ReScript surface (result core); the compiled
// Positions.res.mjs is the JS example.
let main = async () => {
  let client = Common__Example.client()
  await Client.useNativeSigner(client, Common__Example.walletSecretKey())

  switch await Auth.Client.login(client) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok(session) =>
    let wallet = switch session.user.identity {
    | Wallet({address}) => address
    | Google({privy}) | X({privy}) => privy.wallet.address
    }
    switch await Market.Client.get(client, ~limit=1) {
    | Error(error) => Console.error(SdkError.toMessage(error))
    | Ok({markets}) =>
      switch markets[0] {
      | None => Console.log("no markets found")
      | Some(market) =>
        switch await Position.Client.get(client, ~userPubkey=wallet) {
        | Error(error) => Console.error(SdkError.toMessage(error))
        | Ok(all) =>
          switch await Position.Client.getForMarket(
            client,
            ~userPubkey=wallet,
            ~marketPubkey=market.pubkey,
          ) {
          | Error(error) => Console.error(SdkError.toMessage(error))
          | Ok(perMarket) =>
            Console.log(`wallet: ${wallet}`)
            Console.log(`markets with positions: ${Float.toString(all.totalMarkets)}`)
            Console.log(
              `positions in ${market.slug}: ${Int.toString(Array.length(perMarket.positions))}`,
            )
          }
        }
      }
    }
  }
}

let _ = main()
