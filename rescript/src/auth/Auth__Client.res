let install = (client: Client.t, result) => {
  result->Result.forEach(session =>
    client.credentials = Some(Auth__Model.credentialsFromSession(session))
  )
  result
}

// Auth client — nonce + ed25519 signed-message login, session, logout.
// The login flow signs `"Sign in to Lightcone\nNonce: {nonce}"` with the
// client's configured signing strategy; the backend sets the `lightcone-token`
// cookie, which `Http` captures and replays on later requests. Wire decoding
// and the request body live in `Auth__Raw`; message building / signing in
// `Auth__Native`.

// Fetch a single-use nonce for the sign-in challenge.
let getNonce = async (client: Client.t): result<string, SdkError.t> =>
  (
    await Http.get(
      client.http,
      ~path="/api/auth/nonce",
      ~retry=NoRetry,
      ~restoreCredentials=false,
      ~decode=Auth__Raw.Nonce.t_decode,
    )
  )->Result.map(response => response.nonce)

let loginWithMessage = async (
  client: Client.t,
  signed: Auth__Model.SignedLogin.t,
  ~useEmbeddedWallet: option<bool>=?,
): result<Auth__Model.Session.t, SdkError.t> =>
  if Http.isBrowser() {
    Error(
      SdkError.Validation("signed-message login is available only on the server/native surface"),
    )
  } else {
    install(
      client,
      await Http.post(
        client.http,
        ~restoreCredentials=false,
        ~path="/api/auth/login_or_register_with_message",
        ~body=Auth__Raw.SignedLogin.toJson(signed, ~useEmbeddedWallet),
        ~decode=Auth__Raw.decodeSession,
      ),
    )
  }

// Convenience: nonce → sign → login, using the client's configured signing
// strategy (native keypair or external wallet adapter).
let login = async (client: Client.t, ~useEmbeddedWallet: option<bool>=?): result<
  Auth__Model.Session.t,
  SdkError.t,
> =>
  if Http.isBrowser() {
    Error(SdkError.Validation("signed-message login is unavailable in browsers"))
  } else {
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
  }

// Every interactive Privy authentication synchronizes and installs the session.
let registerPrivy = async (client: Client.t, request: Auth__Model.RegisterPrivyRequest.t) =>
  install(
    client,
    await Http.post(
      client.http,
      ~path="/api/auth/register-privy",
      ~retry=Idempotent,
      ~body=Auth__Raw.registerPrivyJson(request),
      ~decode=Auth__Raw.decodeSession,
    ),
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

let checkSession = async (client: Client.t, ~cookieHeader=?) => {
  let result = await Http.get(
    client.http,
    ~path="/api/auth/me",
    ~cookieHeader?,
    ~decode=Auth__Raw.decodeSession,
  )
  if cookieHeader->Option.isNone {
    switch result {
    | Ok(_) => install(client, result)->ignore
    | Error(error) =>
      if !SdkError.isPrivyVerificationUnavailable(error) {
        client.credentials = None
      }
    }
  }
  result
}
let credentials = (client: Client.t) => client.credentials
let nowSeconds: unit => float = %raw(`() => Date.now() / 1000`)
let isAuthenticated = (client: Client.t) =>
  client.credentials->Option.mapOr(false, credentials => credentials.expiresAt > nowSeconds())
let updateMaxSlippagePreference = async (client: Client.t, value: string) => {
  switch ExactDecimal.scaledInteger(value, 28) {
  | Error(_) => Error(SdkError.Validation("max slippage preference must be exact decimal text"))
  | Ok(raw) if raw <= 0n || raw >= 100000000000000000000000000000n =>
    Error(
      SdkError.Validation(
        "max slippage preference must be greater than zero and less than ten percent",
      ),
    )
  | Ok(_) =>
    await Http.post(
      client.http,
      ~path="/api/auth/max_slippage_preference",
      ~retry=Idempotent,
      ~body=JSON.Object(Dict.fromArray([("max_slippage_preference", JSON.String(value))])),
      ~decode=json =>
        switch json {
        | JSON.Object(dict) =>
          switch Dict.get(dict, "max_slippage_preference") {
          | Some(JSON.String(value)) => Ok(value)
          | _ => Spice.error("missing exact preference", json)
          }
        | _ => Spice.error("invalid preference response", json)
        },
    )
  }
}

let logout = async (client: Client.t): result<unit, SdkError.t> => {
  let result = await Http.post(
    client.http,
    ~path="/api/auth/logout",
    ~restoreCredentials=false,
    ~body=JSON.Object(Dict.make()),
    ~decode=_ => Ok(),
  )
  Client.clearAuth(client)
  switch result {
  | Error(error) if SdkError.isUnauthorized(error) => Ok()
  | result => result
  }
}
