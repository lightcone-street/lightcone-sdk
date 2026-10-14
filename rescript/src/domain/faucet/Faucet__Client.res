// Faucet client — testnet SOL + whitelisted deposit-token claims. Testnet-only:
// the call succeeds only on environments whose backend has the faucet enabled
// (typically local / staging). A plain backend HTTP call (`POST /api/claim`),
// NOT an on-chain transaction — the backend performs the mint and returns the
// resulting signature.

// Request testnet SOL and whitelisted deposit tokens for `walletAddress`.
let claim = async (
  client: Client.t,
  ~walletAddress: string,
): result<Faucet__Raw.Response.t, SdkError.t> => {
  let body = JSON.Object(Dict.fromArray([("wallet_address", JSON.String(walletAddress))]))
  await Http.post(client.http, ~path="/api/claim", ~body, ~decode=Faucet__Raw.Response.t_decode)
}
