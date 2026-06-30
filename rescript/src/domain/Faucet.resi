// Faucet domain — testnet SOL + whitelisted deposit-token claims. Public surface:
// the response types (gentype-exported) and `claim`. The spice codecs are internal.

@genType
type faucetToken = {
  symbol: string,
  amount: float,
}

@genType
type faucetResponse = {
  // Signature of the on-chain mint transaction.
  signature: string,
  // Testnet SOL transferred to the wallet.
  sol: float,
  // Per-token amounts minted (e.g. USDC, cbBTC).
  tokens: array<faucetToken>,
}

// Request testnet SOL + whitelisted deposit tokens for `walletAddress`
// (`POST /api/claim`). Succeeds only where the backend faucet is enabled.
let claim: (Client.t, ~walletAddress: string) => promise<result<faucetResponse, SdkError.t>>
