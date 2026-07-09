// Cancel the first open limit order for the authenticated wallet. Mirrors
// rust/examples/cancel_order.rs: authenticate, fetch the user's order snapshot,
// sign a cancel for one order, POST it, then withdraw the released collateral
// back out of the global pool. ReScript surface (result core); the compiled
// .res.mjs is the JS example.

// Mirrors the constant in SubmitOrder__Example: when we cancel the order that
// example left open, we withdraw the same quote amount back from the global
// pool so the deposit/submit/cancel/withdraw cycle is net-neutral across runs.
let orderQuoteAmount = 1100000n // 0.55 * 2 USDC

// Withdraw the released collateral (the first market pair's quote deposit asset).
let withdrawCollateral = async (client, ~user) =>
  switch await Market.Client.get(client, ~limit=1) {
  | Error(error) => Console.error(`withdraw_from_global: ${SdkError.toMessage(error)}`)
  | Ok({markets}) =>
    switch markets[0]->Option.flatMap(market =>
      market.orderbookPairs
      ->Array.find(pair => pair.active)
      ->Option.orElse(market.orderbookPairs[0])
    ) {
    | None => Console.log("withdraw_from_global: no orderbook pair found")
    | Some(pair) =>
      switch await Position.Builders.withdrawFromGlobal(
        client,
        ~user,
        ~mint=SolanaKit.address(pair.quote.depositAsset),
        ~amount=orderQuoteAmount,
      ) {
      | Ok(signature) => Console.log(`withdraw_from_global: confirmed ${signature}`)
      | Error(error) => Console.error(`withdraw_from_global: ${SdkError.toMessage(error)}`)
      }
    }
  }

let main = async () => {
  let client = Common__Example.client()
  let secretKey = Common__Example.walletSecretKey()
  await Client.useNativeSigner(client, secretKey)

  // The keypair signs the cancel message; `maker` is its base58 address.
  let keypair = await SolanaKitKeys.createKeyPairFromBytes(secretKey)
  let maker = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)

  switch await Auth.Client.login(client) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok(_) =>
    switch await Order.Client.getUserOrders(client, ~limit=50) {
    | Error(error) => Console.error(SdkError.toMessage(error))
    | Ok(snapshot) =>
      let firstLimit = snapshot.orders->Array.findMap(order =>
        switch order {
        | Order.Raw.SnapshotOrder.Limit(limit) => Some(limit)
        | Trigger(_) => None
        }
      )
      switch firstLimit {
      | None => Console.log("No open limit orders to cancel.")
      | Some(order) => {
          let body = await Order.Client.cancelBodySigned(
            ~orderHash=order.common.orderHash,
            ~maker=SolanaKit.addressToString(maker),
            ~keypair,
          )
          switch await Order.Client.cancel(client, body) {
          | Ok(cancelled) => {
              Console.log(`cancelled: ${cancelled.orderHash} remaining=${cancelled.remaining}`)
              await withdrawCollateral(client, ~user=maker)
            }
          | Error(error) => Console.error(SdkError.toMessage(error))
          }
        }
      }
    }
  }
}

let _ = main()
