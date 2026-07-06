// WebSocket: the ticker (best bid / ask / mid) and price-history candles (1-minute
// resolution) for the first orderbook of the first market. WS is ReScript-first —
// the inbound message types carry JSON.t TODO arms and are not gentype-exported, so
// there is no TypeScript twin (only WsTickerAndPrices.res.mjs).

// Push messages arrive via callbacks; keep the event loop alive for a window, then
// disconnect. (A setTimeout-backed promise — the only bit of glue an example needs.)
let delay: int => promise<unit> = %raw(`(ms) => new Promise((resolve) => setTimeout(resolve, ms))`)

let show = (value: option<string>): string => value->Option.getOr("-")

let main = async () => {
  let client = Common__Example.client()

  switch await Market.Client.get(client, ~limit=1) {
  | Ok({markets}) =>
    switch markets[0] {
    | Some(market) =>
      switch market.orderbookPairs[0] {
      | Some(pair) =>
        let orderbookId = pair.orderbookId
        let hits = ref(0)

        let connection = Ws.connect(
          ~url=client.wsUrl,
          ~onConnected=() => Console.log(`connected — subscribing to ${orderbookId}`),
          ~onError=error => Console.error(SdkError.toMessage(error)),
          ~onMessage=msg =>
            switch msg.kind {
            | Ticker(ticker) =>
              hits := hits.contents + 1
              Console.log(`ticker: bid=${show(ticker.bestBid)} ask=${show(ticker.bestAsk)} mid=${show(ticker.mid)}`)
            // price_history is internally tagged: an initial snapshot, then per-candle
            // updates, with periodic heartbeats. Qualify the nested constructors —
            // `Snapshot`/`Update`/`Heartbeat` are reused across WS payload modules.
            | PriceHistory(Messages.WsPriceHistory.Snapshot(snapshot)) =>
              hits := hits.contents + 1
              Console.log(`price snapshot: ${Int.toString(Array.length(snapshot.prices))} candle(s)`)
            | PriceHistory(Messages.WsPriceHistory.Update(update)) =>
              hits := hits.contents + 1
              Console.log(`latest candle: t=${Float.toString(update.candle.t)} mid=${show(update.candle.m)}`)
            | PriceHistory(Messages.WsPriceHistory.Heartbeat(heartbeat)) =>
              hits := hits.contents + 1
              Console.log(`heartbeat: ${Float.toString(heartbeat.serverTime)}`)
            | _ => ()
            },
          (),
        )

        Ws.subscribe(connection, Subscriptions.SubscribeParams.Ticker([orderbookId]))->ignore
        Ws.subscribe(
          connection,
          Subscriptions.SubscribeParams.PriceHistory({
            orderbookId,
            resolution: Shared.Resolution.Minute1,
            includeOhlcv: false,
          }),
        )->ignore

        await delay(15000)
        Ws.disconnect(connection)
        if hits.contents == 0 {
          Console.log("received no websocket events — connection may be unreachable")
        }
      | None => Console.log("market has no orderbooks")
      }
    | None => Console.log("no markets found")
    }
  | Error(error) => Console.error(SdkError.toMessage(error))
  }
}

let _ = main()
