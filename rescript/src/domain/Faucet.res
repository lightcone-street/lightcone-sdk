// Faucet domain — testnet SOL + whitelisted deposit-token claims. Mirrors the
// Rust `domain/faucet` plus the root client's `claim`. Testnet-only: the call
// only succeeds on environments whose backend has the faucet enabled (typically
// local / staging).
//
// This is a plain backend HTTP call (`POST /api/claim`), NOT an on-chain
// transaction — the backend performs the mint and returns the resulting
// signature, so we DO port it here.

// ── Wire / domain types ──────────────────────────────────────────────────────
// No wire/domain split: the Rust `FaucetResponse` is returned as-is. Integer
// amounts are floats (JS numbers); `amount` is a u64 of small token units.
@spice
type faucetToken = {
  symbol: string,
  amount: float,
}

@spice
type faucetResponse = {
  // Signature of the on-chain mint transaction.
  signature: string,
  // Testnet SOL transferred to the wallet.
  sol: float,
  // Per-token amounts minted (e.g. USDC, cbBTC).
  tokens: array<faucetToken>,
}

// ── Client function ──────────────────────────────────────────────────────────
// Request testnet SOL and whitelisted deposit tokens for `walletAddress`.
// `POST /api/claim` with body `{ wallet_address }`.
let claim = async (client: Client.t, ~walletAddress: string): result<faucetResponse, SdkError.t> => {
  let body = JSON.Object(Dict.fromArray([("wallet_address", JSON.String(walletAddress))]))
  await Http.post(client.http, ~path="/api/claim", ~body, ~decode=faucetResponse_decode)
}
