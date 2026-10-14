// Faucet wire types — the claim response is returned as-is. `@spice` is kept so
// the codecs are public (the client decodes across the file boundary).

// Per-token amount minted by a claim (e.g. USDC, cbBTC).
module Token: {
  @genType @spice
  type t = {
    symbol: string,
    amount: float,
  }
}

// Response of `POST /api/claim`.
module Response: {
  @genType @spice
  type t = {
    // Signature of the on-chain mint transaction.
    signature: string,
    // Testnet SOL transferred to the wallet.
    sol: float,
    // Per-token amounts minted.
    tokens: array<Token.t>,
  }
}
