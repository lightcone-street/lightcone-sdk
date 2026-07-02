// Auth — nonce + ed25519 signed-message login, session, logout. Mirrors the Rust
// `auth` module. The login flow signs `"Sign in to Lightcone\nNonce: {nonce}"`
// with the wallet keypair (kit `signBytes`); the backend sets the `lightcone-token`
// cookie, which `Http` captures and replays on later requests.
//
// `UserIdentity`/`User`/`SessionResponse` are internally-tagged (serde `tag="type"`)
// or composite, which spice can't auto-derive — they are hand-decoded here, while
// the leaf account types use spice.

// ── Enums ─────────────────────────────────────────────────────────────────────
module AuthMethod = {
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
@spice
type privyEmbeddedWallet = {
  @spice.key("privy_id") privyId: string,
  chain: ChainType.t,
  address: string,
}

@spice
type userPrivyData = {
  id: string,
  wallet: privyEmbeddedWallet,
}

@spice
type xAccountData = {
  @spice.key("user_id") userId?: string,
  username: string,
  @spice.key("display_name") displayName?: string,
  @spice.key("avatar_url") avatarUrl?: string,
}

@spice
type googleAccountData = {
  email: string,
  name?: string,
  @spice.key("given_name") givenName?: string,
  @spice.key("family_name") familyName?: string,
  @spice.key("avatar_url") avatarUrl?: string,
}

// ── Identity (internally-tagged → hand-decoded) ───────────────────────────────

type userIdentity =
  | Google({account: googleAccountData, privy: userPrivyData})
  | X({account: xAccountData, privy: userPrivyData})
  | Wallet({address: string, chain: ChainType.t, privy?: userPrivyData})

let field = (dict, key) => Dict.get(dict, key)

let userIdentityDecode = (json: JSON.t): result<userIdentity, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch field(dict, "type") {
    | Some(JSON.String("google")) =>
      switch (field(dict, "account"), field(dict, "privy")) {
      | (Some(account), Some(privy)) =>
        switch (googleAccountData_decode(account), userPrivyData_decode(privy)) {
        | (Ok(account), Ok(privy)) => Ok(Google({account, privy}))
        | (Error(error), _) | (_, Error(error)) => Error(error)
        }
      | _ => Spice.error("google identity missing account/privy", json)
      }
    | Some(JSON.String("x")) =>
      switch (field(dict, "account"), field(dict, "privy")) {
      | (Some(account), Some(privy)) =>
        switch (xAccountData_decode(account), userPrivyData_decode(privy)) {
        | (Ok(account), Ok(privy)) => Ok(X({account, privy}))
        | (Error(error), _) | (_, Error(error)) => Error(error)
        }
      | _ => Spice.error("x identity missing account/privy", json)
      }
    | Some(JSON.String("wallet")) =>
      switch (field(dict, "address"), field(dict, "chain")) {
      | (Some(JSON.String(address)), Some(chainJson)) =>
        switch ChainType.t_decode(chainJson) {
        | Ok(chain) =>
          let privy =
            field(dict, "privy")->Option.flatMap(json =>
              userPrivyData_decode(json)->Result.mapOr(None, value => Some(value))
            )
          Ok(Wallet({address, chain, ?privy}))
        | Error(error) => Error(error)
        }
      | _ => Spice.error("wallet identity missing address/chain", json)
      }
    | _ => Spice.error("unknown identity type", json)
    }
  | _ => Spice.error("identity is not an object", json)
  }

// ── User + session (composite → hand-decoded) ─────────────────────────────────

type user = {
  userId: string,
  identity: userIdentity,
  connectedX?: xAccountData,
}

let userDecode = (json: JSON.t): result<user, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch (field(dict, "user_id"), field(dict, "identity")) {
    | (Some(JSON.String(userId)), Some(identityJson)) =>
      switch userIdentityDecode(identityJson) {
      | Ok(identity) =>
        let connectedX =
          field(dict, "connected_x")->Option.flatMap(json =>
            xAccountData_decode(json)->Result.mapOr(None, value => Some(value))
          )
        Ok({userId, identity, ?connectedX})
      | Error(error) => Error(error)
      }
    | _ => Spice.error("user missing user_id/identity", json)
    }
  | _ => Spice.error("user is not an object", json)
  }

type sessionResponse = {
  user: user,
  expiresAt: float,
  authMethod: AuthMethod.t,
  isBeta: bool,
}

let sessionResponseDecode = (json: JSON.t): result<sessionResponse, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch (
      field(dict, "user"),
      field(dict, "expires_at"),
      field(dict, "auth_method"),
      field(dict, "is_beta"),
    ) {
    | (
        Some(userJson),
        Some(JSON.Number(expiresAt)),
        Some(authMethodJson),
        Some(JSON.Boolean(isBeta)),
      ) =>
      switch (userDecode(userJson), AuthMethod.t_decode(authMethodJson)) {
      | (Ok(user), Ok(authMethod)) => Ok({user, expiresAt, authMethod, isBeta})
      | (Error(error), _) | (_, Error(error)) => Error(error)
      }
    | _ => Spice.error("session response missing fields", json)
    }
  | _ => Spice.error("session response is not an object", json)
  }

@spice
type nonceResponse = {nonce: string}

// ── Signing ───────────────────────────────────────────────────────────────────

type signedLogin = {
  message: string,
  signatureBs58: string,
  // 32-byte ed25519 public key.
  pubkeyBytes: array<int>,
}

let bytesToIntArray: Uint8Array.t => array<int> = %raw(`(bytes) => Array.from(bytes)`)

let signinMessage = (nonce: string): string => `Sign in to Lightcone\nNonce: ${nonce}`

// Sign the login message with a wallet keypair (ed25519 over the message's UTF-8
// bytes); returns the base58 signature + 32-byte public key.
let signLoginMessage = async (keypair: SolanaKit.cryptoKeyPair, nonce: string): signedLogin => {
  let message = signinMessage(nonce)
  let messageBytes = SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), message)
  let signature = await SolanaKitKeys.signBytes(keypair.privateKey, messageBytes)
  let address = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)
  {
    message,
    signatureBs58: SolanaKitCodec.decode(SolanaKitCodec.getBase58Decoder(), signature),
    pubkeyBytes: bytesToIntArray(
      SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), address),
    ),
  }
}

// ── Client functions ──────────────────────────────────────────────────────────
let getNonce = async (client: Client.t): result<string, SdkError.t> =>
  (
    await Http.get(client.http, ~path="/api/auth/nonce", ~decode=nonceResponse_decode)
  )->Result.map(response => response.nonce)

let loginWithMessage = async (
  client: Client.t,
  signed: signedLogin,
  ~useEmbeddedWallet: option<bool>=?,
): result<sessionResponse, SdkError.t> => {
  let fields = [
    ("message", JSON.String(signed.message)),
    ("signature_bs58", JSON.String(signed.signatureBs58)),
    (
      "pubkey_bytes",
      JSON.Array(signed.pubkeyBytes->Array.map(byte => JSON.Number(Int.toFloat(byte)))),
    ),
  ]
  useEmbeddedWallet->Option.forEach(value =>
    fields->Array.push(("use_embedded_wallet", JSON.Boolean(value)))
  )
  let body = JSON.Object(Dict.fromArray(fields))
  await Http.post(
    client.http,
    ~path="/api/auth/login_or_register_with_message",
    ~body,
    ~decode=sessionResponseDecode,
  )
}

// Convenience: nonce → sign → login, using the client's configured signing
// strategy (native keypair or external wallet adapter).
let login = async (client: Client.t, ~useEmbeddedWallet: option<bool>=?): result<
  sessionResponse,
  SdkError.t,
> =>
  switch Client.signerAddress(client) {
  | None =>
    Error(
      Signing(
        "no signing strategy configured; call Client.useNativeSigner or Client.useExternalSigner first",
      ),
    )
  | Some(address) =>
    switch await getNonce(client) {
    | Error(error) => Error(error)
    | Ok(nonce) =>
      let message = signinMessage(nonce)
      let messageBytes = SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), message)
      switch await Client.signMessageBytes(client, messageBytes) {
      | Error(error) => Error(error)
      | Ok(signature) =>
        let signed = {
          message,
          signatureBs58: SolanaKitCodec.decode(SolanaKitCodec.getBase58Decoder(), signature),
          pubkeyBytes: bytesToIntArray(
            SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), address),
          ),
        }
        await loginWithMessage(client, signed, ~useEmbeddedWallet?)
      }
    }
  }

// Register a Privy-authenticated user in the backend DB. Called after Privy
// login when `is_new_user: true`; idempotent.
let registerPrivy = async (client: Client.t): result<unit, SdkError.t> =>
  await Http.post(
    client.http,
    ~path="/api/auth/register-privy",
    ~body=JSON.Object(Dict.make()),
    ~decode=_ => Ok(),
  )

// Disconnect the user's linked X (Twitter) account.
let disconnectX = async (client: Client.t): result<unit, SdkError.t> =>
  await Http.post(
    client.http,
    ~path="/api/auth/disconnect_x",
    ~body=JSON.Object(Dict.make()),
    ~decode=_ => Ok(),
  )

// The URL for linking an X (Twitter) account via OAuth.
let connectXUrl = (client: Client.t): string => `${Http.baseUrl(client.http)}/api/auth/oauth/link/x`

let checkSession = async (client: Client.t, ~cookieHeader: option<string>=?): result<
  sessionResponse,
  SdkError.t,
> =>
  await Http.get(client.http, ~path="/api/auth/me", ~cookieHeader?, ~decode=sessionResponseDecode)

let isAuthenticated = (client: Client.t): bool => Client.authToken(client)->Option.isSome

let logout = async (client: Client.t): result<unit, SdkError.t> => {
  let result = await Http.post(
    client.http,
    ~path="/api/auth/logout",
    ~body=JSON.Object(Dict.make()),
    ~decode=_ => Ok(),
  )
  Client.clearAuth(client)
  result
}
