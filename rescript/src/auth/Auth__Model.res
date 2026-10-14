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
module EmailAccount = {
  @genType @spice type t = {email: string}
}
module LinkedIdentity = {
  @genType
  type t =
    | Email(EmailAccount.t)
    | Google(GoogleAccount.t)
    | X(XAccount.t)
    | Wallet({address: string, chain: ChainType.t})
}
module LinkedIdentitySelector = {
  @genType
  type t =
    | Email(string)
    | Google(string)
    | X(string)
    | Wallet({address: string, chain: ChainType.t})
}
module RegisterPrivyRequest = {
  @genType type t = {attemptedIdentity: LinkedIdentitySelector.t}
}
module Credentials = {
  @genType type t = {userId: string, walletAddress: string, expiresAt: float}
}
module RegisterPrivyConflict = {
  @genType
  type t =
    | IdentityOwned(option<SdkError.LinkedIdentityType.t>)
    | MultipleAccounts
    | WalletOwned(option<SdkError.LinkedIdentityType.t>)
}
module Identity = {
  type t =
    | Email({account: EmailAccount.t, privy: PrivyData.t})
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
    linkedIdentities: array<LinkedIdentity.t>,
    maxSlippagePreference?: string,
    telegramInviteUrl?: string,
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

let tradingWallet = (user: User.t, authMethod: Method.t) =>
  switch user.identity {
  | Email({privy}) | Google({privy}) | X({privy}) => privy.wallet.address
  | Wallet({address, ?privy}) =>
    switch (authMethod, privy) {
    | (Privy, Some(privy)) => privy.wallet.address
    | _ => address
    }
  }
let credentialsFromSession = (session: Session.t): Credentials.t => {
  userId: session.user.userId,
  walletAddress: tradingWallet(session.user, session.authMethod),
  expiresAt: session.expiresAt,
}
let walletDisplayName = (user, method) => Fmt__Str.shorten(tradingWallet(user, method), ~qty=8)
let displayName = (user: User.t) =>
  switch user.identity {
  | Email({account}) =>
    if String.length(account.email) <= 20 {
      account.email
    } else {
      String.slice(account.email, ~start=0, ~end=8) ++
      "..." ++
      String.slice(
        account.email,
        ~start=String.length(account.email) - 9,
        ~end=String.length(account.email),
      )
    }
  | Google({account}) => account.name->Option.getOr(account.email)
  | X({account}) => account.displayName->Option.getOr(account.username)
  | Wallet({address}) => Fmt__Str.shorten(address, ~qty=8)
  }
let avatarUrl = (user: User.t) =>
  switch user.identity {
  | Google({account}) => account.avatarUrl
  | X({account}) => account.avatarUrl
  | _ => None
  }
let classifyRegisterPrivyConflict = (error: SdkError.t): option<RegisterPrivyConflict.t> =>
  switch error {
  | ApiRejected(details) =>
    switch details.errorCode {
    | Some("IDENTITY_OWNED_BY_ANOTHER_ACCOUNT") => Some(IdentityOwned(details.existingMethod))
    | Some("IDENTITIES_OWNED_BY_MULTIPLE_ACCOUNTS") => Some(MultipleAccounts)
    | Some("WALLET_OWNED_BY_ANOTHER_ACCOUNT") => Some(WalletOwned(details.existingMethod))
    | _ => None
    }
  | _ => None
  }
