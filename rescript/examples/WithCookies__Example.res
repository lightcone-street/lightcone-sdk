// Per-call cookie forwarding for SSR / server-function consumers. Ported from the
// Rust `examples/with_cookies.rs`.
//
// The read endpoints accept an optional `~cookieHeader`: it BYPASSES the SDK's
// process-wide auth-token store and forwards the supplied raw `Cookie` header for
// that single call only — so a server can relay whatever auth cookies the browser
// sent (`lightcone-token` and/or `privy-token`). In a real SSR / server-function
// context the header is built from the incoming request's cookie jar. Here we mimic
// that by: (1) logging in once (the SDK captures `lightcone-token` internally),
// (2) reading the token off the client, (3) clearing the SDK's internal token to
// prove the cookie-forwarding path doesn't depend on it, (4) forwarding the
// captured header on each call.
let main = async () => {
  let client = Common__Example.client()
  await Client.useNativeSigner(client, Common__Example.walletSecretKey())

  switch await Auth.login(client) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok(_) =>
    switch Client.authToken(client) {
    | None => Console.error("auth token not set after login — the SDK should have captured it")
    | Some(token) =>
      // Clear the SDK's internal token, then forward the captured one explicitly.
      Client.clearAuth(client)
      let cookieHeader = `lightcone-token=${token}`

      switch await Position.depositTokenBalances(client, ~cookieHeader) {
      | Ok(balances) =>
        Console.log(`tracked deposit balances: ${Int.toString(Array.length(Dict.keysToArray(balances)))}`)
      | Error(error) => Console.error(SdkError.toMessage(error))
      }

      switch await Order.getUserOrders(client, ~limit=50, ~cookieHeader) {
      | Ok(orders) => Console.log(`open orders: ${Int.toString(Array.length(orders.orders))}`)
      | Error(error) => Console.error(SdkError.toMessage(error))
      }
    }
  }
}

let _ = main()
