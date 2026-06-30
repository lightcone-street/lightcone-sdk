// Authenticated example — the signed-in user's SPL deposit-token balances, keyed
// by mint. Ported from the Rust `examples/deposit_token_balances.rs`. ReScript
// surface: log in with the wallet keypair, then read `Position.depositTokenBalances`
// (the auth cookie captured at login is replayed automatically). The compiled
// DepositTokenBalances.res.mjs is the JS example.
let main = async () => {
  let client = Common__Example.client()
  await Client.useNativeSigner(client, Common__Example.walletSecretKey())

  switch await Auth.login(client) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok(_) =>
    switch await Position.depositTokenBalances(client) {
    | Ok(balances) =>
      let entries = balances->Dict.valuesToArray
      Console.log(`tracked balances: ${Int.toString(Array.length(entries))}`)
      entries
      ->Array.toSorted((a, b) => String.compare(a.symbol, b.symbol))
      ->Array.forEach(balance =>
        Console.log(`  ${balance.symbol}  ${balance.mint}  idle=${balance.idle}`)
      )
      switch await Auth.logout(client) {
      | Ok() => ()
      | Error(error) => Console.error(SdkError.toMessage(error))
      }
    | Error(error) => Console.error(SdkError.toMessage(error))
    }
  }
}

let _ = main()
