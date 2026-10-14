// Push messages arrive via callbacks; keep the event loop alive for a window, then
// disconnect. (A setTimeout-backed promise — the only bit of glue an example needs.)
let delay: int => promise<unit> = %raw(`(ms) => new Promise((resolve) => setTimeout(resolve, ms))`)

let main = async () => {
  let client = Common__Example.client()
  await Client.useNativeSigner(client, Common__Example.walletSecretKey())

  switch await Auth.Client.login(client) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok(_session) =>
    switch Client.signerAddress(client) {
    | None => Console.log("no signer configured")
    | Some(address) =>
      switch await Market.Client.get(client, ~limit=1) {
      | Error(error) => Console.error(SdkError.toMessage(error))
      | Ok({markets}) =>
        switch markets[0] {
        | None => Console.log("no markets found")
        | Some(market) =>
          let walletAddress = SolanaKit.addressToString(address)
          let marketPubkey = market.pubkey
          let sawAuth = ref(false)
          let sawUser = ref(false)
          let sawMarket = ref(false)

          let connection = ServerApi.connect(
            client,
            ~onConnected=() => Console.log("connected — subscribing to user + market streams"),
            ~onError=error => Console.error(SdkError.toMessage(error)),
            ~onMessage=msg =>
              switch msg.kind {
              | Auth(update) =>
                sawAuth := true
                let label = switch update {
                | Authenticated(wallet) => `authenticated as ${wallet}`
                | Anonymous(reason) => `anonymous (${reason->Option.getOr("no reason")})`
                }
                Console.log(`auth: ${label}`)
              | User(update) =>
                sawUser := true
                let label = switch update {
                | Snapshot(snapshot) =>
                  let _openOrders = Order.State.fromSnapshotOrders(snapshot.orders)
                  let orders = Int.toString(Array.length(snapshot.orders))
                  let balances = Int.toString(Array.length(snapshot.marketBalances))
                  `snapshot: ${orders} orders, ${balances} market balances`
                | Order(Limit(update)) =>
                  `limit order ${update.order.orderHash} remaining=${update.order.remaining}`
                | BalanceUpdate(balance) => `balance update for ${balance.marketPubkey}`
                | GlobalDepositUpdate(deposit) =>
                  `global deposit ${deposit.balance} (${deposit.mint})`
                | NonceUpdate(nonce) => `nonce → ${Float.toString(nonce.newNonce)}`
                | NotificationPush(_) => "notification"
                }
                Console.log(`user: ${label}`)
              | Market(event) =>
                sawMarket := true
                let label = switch event {
                | Settled(pubkey) => `settled ${pubkey}`
                | Created(pubkey) => `created ${pubkey}`
                | Opened(pubkey) => `opened ${pubkey}`
                | Paused(pubkey) => `paused ${pubkey}`
                | OrderbookCreated(pubkey, orderbookId) =>
                  `orderbook ${orderbookId} created in ${pubkey}`
                }
                Console.log(`market: ${label}`)
              | _ => ()
              },
          )

          Ws.subscribe(connection, Subscriptions.SubscribeParams.User(walletAddress))->ignore
          Ws.subscribe(connection, Subscriptions.SubscribeParams.Market(marketPubkey))->ignore

          await delay(20000)
          Ws.disconnect(connection)
          if !sawAuth.contents && !sawUser.contents {
            Console.log("received no websocket events — connection may be unreachable")
          }
          Console.log(`market event received: ${sawMarket.contents ? "yes" : "no"}`)
        }
      }
    }
  }
}

let _ = main()
