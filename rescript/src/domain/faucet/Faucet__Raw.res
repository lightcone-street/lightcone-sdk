// Faucet wire types — the claim response is returned as-is (no domain
// conversion). Integer amounts are floats (JS numbers); `amount` is a u64 of
// small token units.

// Per-token amount minted by a claim (e.g. USDC, cbBTC).
module Token = {
  @spice
  type t = {
    symbol: string,
    amount: float,
  }
}

// Response of `POST /api/claim`.
module Response = {
  @spice
  type t = {
    // Signature of the on-chain mint transaction.
    signature: string,
    // Testnet SOL transferred to the wallet.
    sol: float,
    // Per-token amounts minted.
    tokens: array<Token.t>,
  }
}
