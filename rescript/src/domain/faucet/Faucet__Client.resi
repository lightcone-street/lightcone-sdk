// Faucet client — testnet SOL + whitelisted deposit-token claims (testnet-only).

// Request testnet SOL and whitelisted deposit tokens for `walletAddress`.
let claim: (
  Client.t,
  ~walletAddress: string,
) => promise<result<Faucet__Raw.Response.t, SdkError.t>>
