// The SDK client — holds the HTTP transport, environment, on-chain program id,
// RPC handle, and the mutable trading state (deposit source, cached order nonce,
// signing strategy).
//
// Domain modules take a `Client.t` and never the reverse, so there is no module
// cycle. The grouped `client.markets()…` ergonomics live in the gentype facade
// (TypeScriptApi.res); the idiomatic ReScript surface is `Market.Client.featured(client)`.

// An external wallet signer (browser wallet adapter) — the SDK calls these when
// the strategy is `ExternalSigner`.
// `signMessage` returns the raw 64-byte ed25519 signature over the message
// bytes; `signTransaction` takes the serialized unsigned wire transaction and
// returns the fully signed wire bytes.
module ExternalSigner = {
  type t = {
    address: SolanaKit.address,
    signMessage: Uint8Array.t => promise<Uint8Array.t>,
    signTransaction: Uint8Array.t => promise<Uint8Array.t>,
  }
}

// How orders / cancels / transactions get signed. `None` ⇒ manual signing.
// (The `ExternalSigner` constructor and the module above deliberately share the name.)
module SigningStrategy = {
  type t =
    | NativeSigner({
        keypair: SolanaKit.cryptoKeyPair,
        signer: SolanaKit.keyPairSigner,
        address: SolanaKit.address,
      })
    | ExternalSigner(ExternalSigner.t)
}

// Opaque to TypeScript: a client is a handle created by `make`, passed to the
// SDK functions. Its internals (HTTP transport, signer, RPC) aren't TS-facing.
type t = {
  http: Http.t,
  env: Env.t,
  programId: SolanaKit.address,
  wsUrl: string,
  rpcUrl: string,
  backupRpcUrl: option<string>,
  rpc: SolanaKitRpc.t,
  // Backup kit RPC `Rpc` fails over to on infra errors; equals `rpc` when no
  // `backupRpcUrl` is set (a kit RPC is a Proxy and must never be wrapped in an
  // `option` — it answers truthy for every key and corrupts the option tag). Use
  // `backupRpcUrl->Option.isSome` to tell whether a distinct backup exists.
  backupRpc: SolanaKitRpc.t,
  rpcFailover: RpcFailover.t,
  mutable depositSource: Shared.DepositSource.t,
  mutable orderNonce: option<bigint>,
  mutable credentials: option<Auth__Model.Credentials.t>,
  orderbookRules: Dict.t<promise<result<Scaling.OrderbookRules.t, SdkError.t>>>,
  mutable transactionResources: option<V1Transaction.Resources.t>,
  mutable sponsorshipEnabled: bool,
  mutable signingStrategy: option<SigningStrategy.t>,
}

// Build a client. `env` defaults to Prod; per-field URL/programId overrides win,
// then the `SDK_*` env vars (handled inside `Env`), then the built-in defaults.
let make = (
  ~env: Env.t=Prod,
  ~baseUrl: option<string>=?,
  ~wsUrl: option<string>=?,
  ~rpcUrl: option<string>=?,
  ~backupRpcUrl: option<string>=?,
  ~programId: option<string>=?,
  ~depositSource: Shared.DepositSource.t=Global,
  ~transactionResources: option<V1Transaction.Resources.t>=?,
  ~sponsorshipEnabled=false,
  (),
): t => {
  let resolvedRpcUrl = rpcUrl->Option.getOr(Env.rpcUrl(env))
  let rpc = SolanaKitRpc.make(resolvedRpcUrl)
  {
    http: Http.make(baseUrl->Option.getOr(Env.apiUrl(env))),
    env,
    programId: SolanaKit.address(programId->Option.getOr(Env.programId(env))),
    wsUrl: wsUrl->Option.getOr(Env.wsUrl(env)),
    rpcUrl: resolvedRpcUrl,
    backupRpcUrl,
    rpc,
    backupRpc: backupRpcUrl->Option.mapOr(rpc, url => SolanaKitRpc.make(url)),
    rpcFailover: RpcFailover.make(),
    depositSource,
    orderNonce: None,
    credentials: None,
    orderbookRules: Dict.make(),
    signingStrategy: None,
    transactionResources,
    sponsorshipEnabled,
  }
}

// ── Accessors / mutators ──────────────────────────────────────────────────────
let http = (client: t): Http.t => client.http
let depositSource = (client: t): Shared.DepositSource.t => client.depositSource
let setDepositSource = (client: t, source: Shared.DepositSource.t): unit =>
  client.depositSource = source

let orderNonce = (client: t): option<bigint> => client.orderNonce
let setOrderNonce = (client: t, nonce: bigint): unit => client.orderNonce = Some(nonce)
let clearOrderNonce = (client: t): unit => client.orderNonce = None

let signerAddress = (client: t): option<SolanaKit.address> =>
  switch client.signingStrategy {
  | Some(NativeSigner({address})) => Some(address)
  | Some(ExternalSigner({address})) => Some(address)
  | None => None
  }

// Attach a native ed25519 signer from a 64-byte wallet secret ([seed||pubkey],
// the Solana id.json format). Async because key import goes through WebCrypto.
let useNativeSigner = async (client: t, secretKey: Uint8Array.t): unit => {
  let keypair = await SolanaKitKeys.createKeyPairFromBytes(secretKey)
  let signer = await SolanaKitKeys.createKeyPairSignerFromBytes(secretKey)
  client.signingStrategy = Some(
    NativeSigner({keypair, signer, address: SolanaKitKeys.signerAddress(signer)}),
  )
}

// Attach an external wallet signer (browser wallet adapter): the wallet's
// address plus its message- and transaction-signing callbacks.
let useExternalSigner = (
  client: t,
  ~address: SolanaKit.address,
  ~signMessage: Uint8Array.t => promise<Uint8Array.t>,
  ~signTransaction: Uint8Array.t => promise<Uint8Array.t>,
): unit => client.signingStrategy = Some(ExternalSigner({address, signMessage, signTransaction}))

let clearSigningStrategy = (client: t): unit => client.signingStrategy = None

// Sign raw message bytes with the configured strategy (orders, cancels, and the
// login message all sign this way). External-signer exceptions surface as
// `Signing` errors.
let copyMessage: Uint8Array.t => Uint8Array.t = %raw(`value => Uint8Array.from(value)`)
let signMessageBytes = async (client: t, message: Uint8Array.t): result<
  Uint8Array.t,
  SdkError.t,
> => {
  let message = copyMessage(message)
  switch client.signingStrategy {
  | None =>
    Error(
      SdkError.Signing(
        "no signing strategy configured; call Client.useNativeSigner or Client.useExternalSigner first",
      ),
    )
  | Some(NativeSigner({keypair})) => Ok(await SolanaKitKeys.signBytes(keypair.privateKey, message))
  | Some(ExternalSigner({signMessage, address})) =>
    switch await signMessage(copyMessage(message)) {
    | signature =>
      let publicKey = SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), address)
      if await StrictEd25519.verify(signature, message, publicKey) {
        Ok(signature)
      } else {
        Error(SdkError.Signing("wallet returned an invalid message signature"))
      }
    | exception JsExn(error) =>
      Error(
        SdkError.Signing(
          error->JsExn.message->Option.getOr("external signer failed to sign message"),
        ),
      )
    }
  }
}

let clearAuth = (client: t): unit => {
  Http.clearAuthToken(client.http)
  client.credentials = None
}
let authToken = (client: t): option<string> => Http.authToken(client.http)
