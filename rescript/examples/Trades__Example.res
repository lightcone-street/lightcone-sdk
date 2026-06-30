// Recent trades for the first orderbook of the first market. ReScript surface
// (result core); the compiled Trades.res.mjs is the JS example.
let main = async () => {
  let client = Common__Example.client()
  switch await Market.get(client, ~limit=1) {
  | Ok({markets}) =>
    switch markets[0] {
    | Some(market) =>
      switch market.orderbookPairs[0] {
      | Some(pair) =>
        switch await Trade.get(client, ~orderbookId=pair.orderbookId, ~limit=5) {
        | Ok(page) =>
          Console.log(`${Int.toString(Array.length(page.trades))} recent trades for ${pair.orderbookId}:`)
          page.trades->Array.forEach(trade =>
            Console.log(`  ${Shared.Side.toString(trade.side)} ${trade.size} @ ${trade.price}`)
          )
        | Error(error) => Console.error(SdkError.toMessage(error))
        }
      | None => Console.log("market has no orderbooks")
      }
    | None => Console.log("no markets found")
    }
  | Error(error) => Console.error(SdkError.toMessage(error))
  }
}

let _ = main()
