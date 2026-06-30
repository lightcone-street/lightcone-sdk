// Auth — nonce + ed25519 signed-message login, session, logout. Public surface: the
// identity/session/signed-login types + the client functions. The internally-tagged
// hand-decoders, spice codecs, and signing-message helpers are internal.

module AuthMethod: {
  @genType
  type t =
    | @as("privy") Privy
    | @as("lightcone") Lightcone
}

module ChainType: {
  @genType
  type t =
    | @as("solana") Solana
    | @as("ethereum") Ethereum
}

@genType
type privyEmbeddedWallet = {
  privyId: string,
  chain: ChainType.t,
  address: string,
}

@genType
type userPrivyData = {
  id: string,
  wallet: privyEmbeddedWallet,
}

@genType
type xAccountData = {
  userId?: string,
  username: string,
  displayName?: string,
  avatarUrl?: string,
}

@genType
type googleAccountData = {
  email: string,
  name?: string,
  givenName?: string,
  familyName?: string,
  avatarUrl?: string,
}

@genType
type userIdentity =
  | Google({account: googleAccountData, privy: userPrivyData})
  | X({account: xAccountData, privy: userPrivyData})
  | Wallet({address: string, chain: ChainType.t, privy?: userPrivyData})

@genType
type user = {
  userId: string,
  identity: userIdentity,
  connectedX?: xAccountData,
}

@genType
type sessionResponse = {
  user: user,
  expiresAt: float,
  authMethod: AuthMethod.t,
  isBeta: bool,
}

@genType
type signedLogin = {
  message: string,
  signatureBs58: string,
  // 32-byte ed25519 public key.
  pubkeyBytes: array<int>,
}

// Sign the login message with a wallet keypair (ed25519 over the message's UTF-8 bytes).
let signLoginMessage: (SolanaKit.cryptoKeyPair, string) => promise<signedLogin>

let getNonce: Client.t => promise<result<string, SdkError.t>>

let loginWithMessage: (
  Client.t,
  signedLogin,
  ~useEmbeddedWallet: bool=?,
) => promise<result<sessionResponse, SdkError.t>>

// Convenience: nonce → sign → login, using the client's configured native signer.
let login: (Client.t, ~useEmbeddedWallet: bool=?) => promise<result<sessionResponse, SdkError.t>>

let checkSession: (
  Client.t,
  ~cookieHeader: string=?,
) => promise<result<sessionResponse, SdkError.t>>

let isAuthenticated: Client.t => bool

let logout: Client.t => promise<result<unit, SdkError.t>>
