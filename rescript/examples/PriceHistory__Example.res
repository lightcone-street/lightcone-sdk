// Orderbook price history (midpoint candles) over the last 7 days at 1h
// resolution, for the first orderbook of the first market. ReScript surface
// (result core); the compiled PriceHistory.res.mjs is the JS example.
let main = async () => {
  let client = Common__Example.client()
  switch await Market.get(client, ~limit=1) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok({markets}) =>
    switch markets[0] {
    | None => Console.log("no markets found")
    | Some(market) =>
      switch market.orderbookPairs[0] {
      | None => Console.log("market has no orderbooks")
      | Some(pair) =>
        let toMs = Date.now()
        let fromMs = toMs -. 7.0 *. 24.0 *. 60.0 *. 60.0 *. 1000.0
        switch await PriceHistory.get(
          client,
          ~orderbookId=pair.orderbookId,
          ~resolution=Shared.Resolution.Hour1,
          ~fromMs,
          ~toMs,
        ) {
        | Error(error) => Console.error(SdkError.toMessage(error))
        | Ok(history) =>
          Console.log(`market: ${market.slug}`)
          Console.log(`orderbook: ${history.orderbookId}`)
          Console.log(
            `${Shared.Resolution.toString(history.resolution)} candles: ${Int.toString(
                Array.length(history.prices),
              )} (has_more=${history.hasMore ? "true" : "false"})`,
          )
          Console.log(
            `decimals: price=${Float.toString(history.decimals.price)}, volume=${Float.toString(
                history.decimals.volume,
              )}`,
          )
          history.prices
          ->Array.slice(~start=0, ~end=5)
          ->Array.forEach(candle =>
            Console.log(`  t=${Float.toString(candle.t)} mid=${candle.m->Option.getOr("—")}`)
          )
        }
      }
    }
  }
}

let _ = main()
