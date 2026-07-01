// WebSocket: maintain a LIVE order book (OrderbookState) and a rolling price-history series
// (PriceHistoryState) for the first orderbook of the first market — printing best bid / best
// ask / mid / spread as snapshot frames arrive. Demonstrates the stateful WS containers
// (ported from rust `*/state.rs`). ReScript surface — the domain containers directly.

let sleep = (ms: int): promise<unit> =>
  Promise.make((resolve, _reject) => setTimeout(() => resolve(), ms)->ignore)

let main = async () => {
  let client = Common__Example.client()

  switch await Market.get(client, ~limit=1) {
  | Ok({markets}) =>
    switch markets[0]->Option.flatMap(market => market.orderbookPairs[0]) {
    | Some(pair) =>
      let orderbookId = pair.orderbookId
      let book = OrderbookState.make(orderbookId)
      let history = PriceHistoryState.make()
      let hits = ref(0)
      // The resync handler re-subscribes, so it needs the connection — threaded back via a ref.
      let connectionRef: ref<option<Ws.connection>> = ref(None)

      let show = value => value->Option.getOr("-")

      let connection = Ws.connect(
        ~url=client.wsUrl,
        ~onConnected=() => Console.log(`connected — subscribing to ${orderbookId}`),
        ~onError=error => Console.error(SdkError.toMessage(error)),
        ~onMessage=msg =>
          switch msg.kind {
          | BookUpdate(frame) =>
            switch OrderbookState.apply(book, frame) {
            | Applied =>
              hits := hits.contents + 1
              Console.log(
                `book: bid=${show(OrderbookState.bestBid(book))} ask=${show(
                    OrderbookState.bestAsk(book),
                  )} mid=${show(OrderbookState.midPrice(book))} spread=${show(
                    OrderbookState.spread(book),
                  )}`,
              )
            | RefreshRequired(ServerResync) =>
              // Book fell out of sync — re-subscribe to pull a fresh seq-0 snapshot.
              connectionRef.contents->Option.forEach(conn => {
                Ws.unsubscribe(
                  conn,
                  Subscriptions.UnsubscribeParams.Books({orderbookIds: [orderbookId]}),
                )->ignore
                Ws.subscribe(
                  conn,
                  Subscriptions.SubscribeParams.Books({orderbookIds: [orderbookId]}),
                )->ignore
              })
            }
          | PriceHistory(event) =>
            switch event {
            | Snapshot(snap) =>
              hits := hits.contents + 1
              PriceHistoryState.applySnapshot(
                history,
                ~orderbookId=snap.orderbookId,
                ~resolution=snap.resolution,
                ~candles=snap.prices,
              )
              Console.log(`price-history snapshot: ${Int.toString(Array.length(snap.prices))} candles`)
            | Update(upd) =>
              PriceHistoryState.applyUpdate(
                history,
                ~orderbookId=upd.orderbookId,
                ~resolution=upd.resolution,
                ~candle=upd.candle,
              )
            | Heartbeat(_) => ()
            }
          | _ => ()
          },
        (),
      )
      connectionRef := Some(connection)

      Ws.subscribe(
        connection,
        Subscriptions.SubscribeParams.Books({orderbookIds: [orderbookId]}),
      )->ignore
      Ws.subscribe(
        connection,
        Subscriptions.SubscribeParams.PriceHistory({
          orderbookId,
          resolution: Hour1,
          includeOhlcv: true,
        }),
      )->ignore

      await sleep(15000)
      Ws.disconnect(connection)

      switch PriceHistoryState.get(history, ~orderbookId, ~resolution=Hour1) {
      | Some(series) => Console.log(`final price-history series: ${Int.toString(Array.length(series))} points`)
      | None => ()
      }
      if hits.contents == 0 {
        Console.log("received no websocket events — connection may be unreachable")
      }
    | None => Console.log("no orderbooks found")
    }
  | Error(error) => Console.error(SdkError.toMessage(error))
  }
}

let _ = main()
