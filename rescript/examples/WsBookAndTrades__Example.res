// WebSocket: live order book (a full-precision view + a grouped 5-sig-fig view on
// one connection) and the trade tape for the first orderbook of the first market.
// WS is ReScript-first — the inbound message types carry JSON.t TODO arms and are
// not gentype-exported, so there is no TypeScript twin (only WsBookAndTrades.res.mjs).

// Push messages arrive via callbacks; keep the event loop alive for a window, then
// disconnect. (A setTimeout-backed promise — the only bit of glue an example needs.)
let delay: int => promise<unit> = %raw(`(ms) => new Promise((resolve) => setTimeout(resolve, ms))`)

// Best price on a side of a snapshot frame — every book frame carries the full
// top-of-book, so the first level is the best level (no stateful book needed here).
let topPrice = (levels: array<Orderbook.Raw.WsLevel.t>): string =>
  switch levels[0] {
  | Some(level) => level.price
  | None => "-"
  }

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
        // The handler re-subscribes on a resync, so it needs the connection — which
        // is created with the handler. Thread it back in through a ref.
        let connectionRef: ref<option<Ws.t>> = ref(None)

        let connection = Ws.connect(
          ~url=client.wsUrl,
          ~onConnected=() => Console.log(`connected — subscribing to ${orderbookId}`),
          ~onError=error => Console.error(SdkError.toMessage(error)),
          ~onMessage=msg =>
            switch msg.kind {
            | BookUpdate(book) =>
              // Untagged frames are the full-precision view; grouped frames carry
              // n_sig_figs/mantissa. Key local handling by the frame's aggregation.
              let aggregation = Orderbook.Raw.Book.toAggregation(book)
              if book.resync {
                // The book fell out of sync: re-subscribe the SAME view to pull a
                // fresh seq-0 snapshot (last-write-wins).
                connectionRef.contents->Option.forEach(conn => {
                  Ws.unsubscribe(
                    conn,
                    Subscriptions.UnsubscribeParams.Books({
                      orderbookIds: [book.id],
                      nSigFigs: ?aggregation.nSigFigs,
                      mantissa: ?aggregation.mantissa,
                    }),
                  )->ignore
                  Ws.subscribe(
                    conn,
                    Subscriptions.SubscribeParams.Books({
                      orderbookIds: [book.id],
                      nSigFigs: ?aggregation.nSigFigs,
                      mantissa: ?aggregation.mantissa,
                    }),
                  )->ignore
                })
              } else {
                hits := hits.contents + 1
                Console.log(
                  `book[${Orderbook.Aggregation.keySuffix(aggregation)}]: seq=${Float.toString(
                      book.seq,
                    )} bid=${topPrice(book.bids)} ask=${topPrice(book.asks)}`,
                )
              }
            | Trades(trade) =>
              hits := hits.contents + 1
              Console.log(
                `trade: ${trade.size} ${Shared.Side.toString(trade.side)} @ ${trade.price} seq=${Float.toString(
                    trade.sequence,
                  )}`,
              )
            | _ => ()
            },
          (),
        )
        connectionRef := Some(connection)

        // One connection, two book views: full precision (pricing) + a grouped view
        // (5 sig figs, mantissa 2 — display), plus the trade tape. Subscriptions are
        // tracked and (re)sent on open, so issuing them now is fine.
        Ws.subscribe(connection, Subscriptions.SubscribeParams.Books({orderbookIds: [orderbookId]}))->ignore
        Ws.subscribe(
          connection,
          Subscriptions.SubscribeParams.Books({orderbookIds: [orderbookId], nSigFigs: 5, mantissa: 2}),
        )->ignore
        Ws.subscribe(connection, Subscriptions.SubscribeParams.Trades([orderbookId]))->ignore

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
