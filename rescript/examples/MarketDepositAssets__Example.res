// Deposit assets registered for a market, each with its conditional mints.
// ReScript surface (result core); the compiled MarketDepositAssets.res.mjs is
// the JS example.
let main = async () => {
  let client = Common__Example.client()
  switch await Market.get(client, ~limit=1) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok({markets}) =>
    switch markets[0] {
    | None => Console.log("no markets found")
    | Some(market) =>
      switch await Market.getDepositMints(client, ~marketPubkey=market.pubkey) {
      | Error(error) => Console.error(SdkError.toMessage(error))
      | Ok(response) =>
        Console.log(
          `market ${market.slug} (${response.marketPubkey}): ${Float.toString(
              response.total,
            )} deposit assets`,
        )
        response.depositAssets->Array.forEach(asset =>
          Console.log(
            `  - ${asset.symbol->Option.getOr("?")} (${asset.depositAsset}) — ${Int.toString(
                Array.length(asset.conditionalMints),
              )} conditional mints`,
          )
        )
      }
    }
  }
}

let _ = main()
