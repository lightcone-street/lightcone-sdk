// WebSocket (authenticated): a user's private stream (auth handshake, balance,
// order, deposit, and nonce events) alongside a market's lifecycle stream. The
// user channel requires login first — the cookie set by `Auth.login` authenticates
// the socket. WS is ReScript-first — the inbound message types carry JSON.t TODO
// arms and are not gentype-exported, so there is no TypeScript twin (only
// WsUserAndMarket.res.mjs).

// Push messages arrive via callbacks; keep the event loop alive for a window, then
// disconnect. (A setTimeout-backed promise — the only bit of glue an example needs.)
let delay: int => promise<unit> = %raw(`(ms) => new Promise((resolve) => setTimeout(resolve, ms))`)

let main = async () => {
  let client = Common__Example.client()
  await Client.useNativeSigner(client, Common__Example.walletSecretKey())

  switch await Auth.login(client) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok(_session) =>
    switch Client.signerAddress(client) {
    | None => Console.log("no signer configured")
    | Some(address) =>
      switch await Market.get(client, ~limit=1) {
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

          let connection = Ws.connect(
            ~url=client.wsUrl,
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
                | Snapshot(_) => "snapshot"
                | Order(_) => "order"
                | BalanceUpdate(balance) => `balance update for ${balance.marketPubkey}`
                | GlobalDepositUpdate(deposit) => `global deposit ${deposit.balance} (${deposit.mint})`
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
                | OrderbookCreated(pubkey, orderbookId) => `orderbook ${orderbookId} created in ${pubkey}`
                }
                Console.log(`market: ${label}`)
              | _ => ()
              },
            (),
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
