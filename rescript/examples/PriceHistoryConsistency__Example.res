// Compares price-history points from the REST API (getLineData) against the WS
// PriceHistory snapshot for the same orderbook + resolution. Ported from
// rust/examples/price_history_consistency.rs. ReScript surface (result core);
// the compiled .res.mjs is the JS example.
let delay: int => promise<unit> = %raw(`(ms) => new Promise((resolve) => setTimeout(resolve, ms))`)

let main = async () => {
  let client = Common__Example.client()
  switch await Market.Client.get(client, ~limit=1) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok({markets}) =>
    switch markets[0]->Option.flatMap(market => market.orderbookPairs[0]) {
    | None => Console.log("no orderbook found")
    | Some(pair) =>
      let orderbookId = pair.orderbookId
      let resolution = Shared.Resolution.Minute5

      // 1. REST API — the same line-data the app's server function fetches.
      switch await PriceHistory.Client.getLineData(client, ~orderbookId, ~resolution, ~limit=1000.0) {
      | Error(error) => Console.error(SdkError.toMessage(error))
      | Ok(restPoints) =>
        Console.log("=== REST API ===")
        Console.log(`  points: ${Int.toString(Array.length(restPoints))}`)
      }

      // 2. WebSocket snapshot for the same orderbook + resolution.
      let wsFrames = ref(0)
      let connection = Ws.connect(
        ~url=client.wsUrl,
        ~onMessage=message =>
          switch message.kind {
          | PriceHistory(_) => wsFrames := wsFrames.contents + 1
          | _ => ()
          },
        (),
      )
      Ws.subscribe(
        connection,
        Subscriptions.SubscribeParams.PriceHistory({orderbookId, resolution, includeOhlcv: false}),
      )->ignore
      await delay(5000)
      Console.log("=== WebSocket ===")
      Console.log(`  price_history frames: ${Int.toString(wsFrames.contents)}`)
      Ws.disconnect(connection)
    }
  }
}

let _ = main()
