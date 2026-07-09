// Auth domain types — how a session authenticated (`Method`), the leaf account
// shapes (Privy / X / Google), the login identity tree, the session envelope,
// and the signed-login material. `@spice` is restated where `Auth__Raw` decodes
// across the file boundary; `@genType` exports the TS shapes.

// How a session authenticated, as reported by the backend.
module Method: {
  @genType @spice
  type t =
    | @as("privy") @spice.as("privy") Privy
    | @as("lightcone") @spice.as("lightcone") Lightcone
}

module ChainType: {
  @genType @spice
  type t =
    | @as("solana") @spice.as("solana") Solana
    | @as("ethereum") @spice.as("ethereum") Ethereum
}

// A Privy-managed embedded wallet.
module PrivyEmbeddedWallet: {
  @genType
  type t = {
    privyId: string,
    chain: ChainType.t,
    address: string,
  }
}

// Privy account data attached to an identity.
module PrivyData: {
  @genType @spice
  type t = {
    id: string,
    wallet: PrivyEmbeddedWallet.t,
  }
}

// X account data — the same shape whether X is the login identity or a
// connected account on another identity.
module XAccount: {
  @genType @spice
  type t = {
    @spice.key("user_id") userId?: string,
    username: string,
    @spice.key("display_name") displayName?: string,
    @spice.key("avatar_url") avatarUrl?: string,
  }
}

// Google account data for a Google login identity.
module GoogleAccount: {
  @genType @spice
  type t = {
    email: string,
    name?: string,
    @spice.key("given_name") givenName?: string,
    @spice.key("family_name") familyName?: string,
    @spice.key("avatar_url") avatarUrl?: string,
  }
}

// The login identity, internally tagged on the wire (`type`: google | x | wallet).
module Identity: {
  @genType
  type t =
    | Google({account: GoogleAccount.t, privy: PrivyData.t})
    | X({account: XAccount.t, privy: PrivyData.t})
    | Wallet({address: string, chain: ChainType.t, privy?: PrivyData.t})
}

// The authenticated user: id + login identity (+ optionally connected X account).
module User: {
  @genType
  type t = {
    userId: string,
    identity: Identity.t,
    connectedX?: XAccount.t,
  }
}

// The session envelope returned by login / check-session.
module Session: {
  @genType
  type t = {
    user: User.t,
    expiresAt: float,
    authMethod: Method.t,
    isBeta: bool,
  }
}

// Signed login material ready to pass to `Auth__Client.loginWithMessage`.
module SignedLogin: {
  @genType
  type t = {
    message: string,
    signatureBs58: string,
    // 32-byte ed25519 public key.
    pubkeyBytes: array<int>,
  }
}
