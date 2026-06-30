// Batch BBO + midpoint per active orderbook (the REST form of the WS ticker
// stream), optionally filtered to a deposit asset passed as a CLI arg. ReScript
// surface (result core); the compiled OrderbookTickers.res.mjs is the JS example.
@val @scope("process") external argv: array<string> = "argv"

let main = async () => {
  let client = Common__Example.client()
  let depositAsset = argv[2]

  switch await Metrics.orderbookTickers(client, ~depositAsset?) {
  | Ok(response) =>
    Console.log(`orderbooks with tickers: ${Int.toString(Array.length(response.tickers))}`)
    response.tickers
    ->Array.slice(~start=0, ~end=10)
    ->Array.forEach(entry => {
      let mid = entry.midpoint->Option.getOr("—")
      let outcome = entry.outcomeIndex->Option.mapOr("—", value => Float.toString(value))
      Console.log(
        `  ${entry.orderbookId} (market ${entry.marketPubkey}, outcome ${outcome}) mid=${mid}`,
      )
    })
  | Error(error) => Console.error(SdkError.toMessage(error))
  }
}

let _ = main()
