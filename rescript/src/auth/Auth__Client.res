// Auth client — nonce + ed25519 signed-message login, session, logout.
// The login flow signs `"Sign in to Lightcone\nNonce: {nonce}"` with the
// client's configured signing strategy; the backend sets the `lightcone-token`
// cookie, which `Http` captures and replays on later requests. Wire decoding
// and the request body live in `Auth__Raw`; message building / signing in
// `Auth__Native`.

// Fetch a single-use nonce for the sign-in challenge.
let getNonce = async (client: Client.t): result<string, SdkError.t> =>
  (
    await Http.get(client.http, ~path="/api/auth/nonce", ~decode=Auth__Raw.Nonce.t_decode)
  )->Result.map(response => response.nonce)

let loginWithMessage = async (
  client: Client.t,
  signed: Auth__Model.SignedLogin.t,
  ~useEmbeddedWallet: option<bool>=?,
): result<Auth__Model.Session.t, SdkError.t> =>
  await Http.post(
    client.http,
    ~path="/api/auth/login_or_register_with_message",
    ~body=Auth__Raw.SignedLogin.toJson(signed, ~useEmbeddedWallet),
    ~decode=Auth__Raw.decodeSession,
  )

// Convenience: nonce → sign → login, using the client's configured signing
// strategy (native keypair or external wallet adapter).
let login = async (client: Client.t, ~useEmbeddedWallet: option<bool>=?): result<
  Auth__Model.Session.t,
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
      let message = Auth__Native.signinMessage(nonce)
      let messageBytes = SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), message)
      switch await Client.signMessageBytes(client, messageBytes) {
      | Error(error) => Error(error)
      | Ok(signature) =>
        let signed = Auth__Native.signedLogin(~message, ~signature, ~address)
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
  Auth__Model.Session.t,
  SdkError.t,
> =>
  await Http.get(client.http, ~path="/api/auth/me", ~cookieHeader?, ~decode=Auth__Raw.decodeSession)

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
