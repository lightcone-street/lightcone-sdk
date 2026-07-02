// Cancel the first open limit order for the authenticated wallet. Mirrors
// rust/examples/cancel_order.rs (the single-order cancel path): authenticate,
// fetch the user's order snapshot, sign a cancel for one order, and POST it.
//
// The Rust example also cancels-all and withdraws the released collateral back
// from the global pool to keep the submit/cancel cycle net-neutral; see
// GlobalDepositWithdrawal.res for the withdraw flow. ReScript surface (result
// core); the compiled .res.mjs is the JS example.
//
// No TS facade equivalent: signing the cancel needs SolanaKit (@solana/kit)
// address/keypair helpers not surfaced on TypeScriptApi.gen.ts.
let main = async () => {
  let client = Common__Example.client()
  let secretKey = Common__Example.walletSecretKey()
  await Client.useNativeSigner(client, secretKey)

  // The keypair signs the cancel message; `maker` is its base58 address.
  let keypair = await SolanaKitKeys.createKeyPairFromBytes(secretKey)
  let maker = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)

  switch await Auth.login(client) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok(_) =>
    switch await Order.getUserOrders(client, ~limit=50) {
    | Error(error) => Console.error(SdkError.toMessage(error))
    | Ok(snapshot) =>
      let firstLimit = snapshot.orders->Array.findMap(order =>
        switch order {
        | Order.UserSnapshotOrder.Limit(limit) => Some(limit)
        | Trigger(_) => None
        }
      )
      switch firstLimit {
      | None => Console.log("No open limit orders to cancel.")
      | Some(order) => {
          let body = await Order.cancelBodySigned(
            ~orderHash=order.common.orderHash,
            ~maker=SolanaKit.addressToString(maker),
            ~keypair,
          )
          switch await Order.cancel(client, body) {
          | Ok(cancelled) => Console.log(`cancelled: ${cancelled.orderHash} remaining=${cancelled.remaining}`)
          | Error(error) => Console.error(SdkError.toMessage(error))
          }
        }
      }
    }
  }
}

let _ = main()
