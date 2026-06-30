// Live orderbook depth (REST) plus a Hyperliquid-style aggregated view, for the
// first orderbook of the first market. ReScript surface (result core); the
// compiled Orderbook.res.mjs is the JS example.
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
        let orderbookId = pair.orderbookId

        // Depth is capped server-side at 20 levels per side.
        switch await Orderbook.get(client, ~orderbookId, ~depth=10) {
        | Error(error) => Console.error(SdkError.toMessage(error))
        | Ok(depth) =>
          Console.log(`market: ${market.slug}`)
          Console.log(`orderbook: ${orderbookId}`)
          Console.log(
            `best bid: ${depth.bestBid->Option.getOr("—")}, best ask: ${depth.bestAsk->Option.getOr("—")}`,
          )
          Console.log(
            `levels: ${Int.toString(Array.length(depth.bids))} bids / ${Int.toString(
                Array.length(depth.asks),
              )} asks`,
          )
          Console.log(
            `token decimals: base=${Float.toString(pair.base.decimals)}, quote=${Float.toString(
                pair.quote.decimals,
              )}`,
          )
          switch depth.decimals {
          | Some(depthDecimals) =>
            Console.log(
              `depth decimals: price=${Float.toString(depthDecimals.price)}, size=${Float.toString(
                  depthDecimals.size,
                )}`,
            )
          | None => ()
          }

          // Hyperliquid-style aggregation: 5 significant figures, mantissa 2.
          // Bids bucket by flooring, asks by ceiling.
          switch Orderbook.BookAggregation.validate(Some(5), Some(2)) {
          | Error(message) => Console.error(message)
          | Ok(aggregation) =>
            switch await Orderbook.get(client, ~orderbookId, ~aggregation) {
            | Error(error) => Console.error(SdkError.toMessage(error))
            | Ok(grouped) =>
              Console.log(
                `grouped (${Orderbook.BookAggregation.keySuffix(aggregation)}): ${Int.toString(
                    Array.length(grouped.bids),
                  )} bids / ${Int.toString(Array.length(grouped.asks))} asks`,
              )
            }
          }
        }
      }
    }
  }
}

let _ = main()
