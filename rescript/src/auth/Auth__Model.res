// Auth domain types — how a session authenticated (`Method`), the leaf account
// shapes (Privy / X / Google), the login identity tree, the session envelope,
// and the signed-login material. `Identity`/`User`/`Session` are internally
// tagged (`type`) or composite, which spice can't auto-derive — they are
// hand-decoded in `Auth__Raw` — while the leaf account types use spice.

// ── Enums ─────────────────────────────────────────────────────────────────────
// How a session authenticated, as reported by the backend.
module Method = {
  @spice
  type t =
    | @as("privy") @spice.as("privy") Privy
    | @as("lightcone") @spice.as("lightcone") Lightcone
}

module ChainType = {
  @spice
  type t =
    | @as("solana") @spice.as("solana") Solana
    | @as("ethereum") @spice.as("ethereum") Ethereum
}

// ── Leaf account types (spice) ────────────────────────────────────────────────
// A Privy-managed embedded wallet.
module PrivyEmbeddedWallet = {
  @spice
  type t = {
    @spice.key("privy_id") privyId: string,
    chain: ChainType.t,
    address: string,
  }
}

// Privy account data attached to an identity.
module PrivyData = {
  @spice
  type t = {
    id: string,
    wallet: PrivyEmbeddedWallet.t,
  }
}

// X account data — the same shape whether X is the login identity or a
// connected account on another identity.
module XAccount = {
  @spice
  type t = {
    @spice.key("user_id") userId?: string,
    username: string,
    @spice.key("display_name") displayName?: string,
    @spice.key("avatar_url") avatarUrl?: string,
  }
}

// Google account data for a Google login identity.
module GoogleAccount = {
  @spice
  type t = {
    email: string,
    name?: string,
    @spice.key("given_name") givenName?: string,
    @spice.key("family_name") familyName?: string,
    @spice.key("avatar_url") avatarUrl?: string,
  }
}

// ── Identity / user / session (hand-decoded in Auth__Raw) ─────────────────────
// The login identity, internally tagged on the wire (`type`: google | x | wallet).
module Identity = {
  type t =
    | Google({account: GoogleAccount.t, privy: PrivyData.t})
    | X({account: XAccount.t, privy: PrivyData.t})
    | Wallet({address: string, chain: ChainType.t, privy?: PrivyData.t})
}

// The authenticated user: id + login identity (+ optionally connected X account).
module User = {
  type t = {
    userId: string,
    identity: Identity.t,
    connectedX?: XAccount.t,
  }
}

// The session envelope returned by login / check-session.
module Session = {
  type t = {
    user: User.t,
    expiresAt: float,
    authMethod: Method.t,
    isBeta: bool,
  }
}

// ── Signing ───────────────────────────────────────────────────────────────────
// Signed login material ready to pass to `Auth__Client.loginWithMessage`.
module SignedLogin = {
  type t = {
    message: string,
    signatureBs58: string,
    // 32-byte ed25519 public key.
    pubkeyBytes: array<int>,
  }
}
