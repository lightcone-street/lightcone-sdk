// Authenticate with a wallet keypair, then inspect the session.
let main = async () => {
  let client = Common__Example.client()
  await Client.useNativeSigner(client, Common__Example.walletSecretKey())

  switch await Auth.Client.login(client) {
  | Ok(session) =>
    Console.log(`Logged in as user ${session.user.userId}`)
    let method = switch session.authMethod {
    | Privy => "privy"
    | Lightcone => "lightcone"
    }
    Console.log(`Auth method: ${method}, beta: ${session.isBeta ? "yes" : "no"}`)
  | Error(error) => Console.error(SdkError.toMessage(error))
  }
}

let _ = main()
