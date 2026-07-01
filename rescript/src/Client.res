// The SDK client — holds the HTTP transport, environment, on-chain program id,
// RPC handle, and the mutable trading state (deposit source, cached order nonce,
// signing strategy). Mirrors the Rust `LightconeClient`.
//
// Domain modules take a `Client.t` and never the reverse, so there is no module
// cycle. The grouped `client.markets()…` ergonomics live in the gentype facade
// (TypeScriptApi.res); the idiomatic ReScript surface is `Market.featured(client)`.

// How orders / cancels / transactions get signed. `None` ⇒ manual signing.
type signingStrategy =
  | NativeSigner({
      keypair: SolanaKit.cryptoKeyPair,
      signer: SolanaKit.keyPairSigner,
      address: SolanaKit.address,
    })

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
  rpcFailover: RpcFailover.state,
  mutable depositSource: Shared.DepositSource.t,
  mutable orderNonce: option<bigint>,
  mutable signingStrategy: option<signingStrategy>,
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
    signingStrategy: None,
  }
}

// ── Accessors / mutators ──────────────────────────────────────────────────────
let http = (client: t): Http.t => client.http
let depositSource = (client: t): Shared.DepositSource.t => client.depositSource
let setDepositSource = (client: t, source: Shared.DepositSource.t): unit =>
  client.depositSource = source

let orderNonce = (client: t): option<bigint> => client.orderNonce
let setOrderNonce = (client: t, nonce: bigint): unit => client.orderNonce = Some(nonce)

let signerAddress = (client: t): option<SolanaKit.address> =>
  switch client.signingStrategy {
  | Some(NativeSigner({address})) => Some(address)
  | None => None
  }

// Attach a native ed25519 signer from a 64-byte wallet secret ([seed||pubkey],
// the Solana id.json format). Async because key import goes through WebCrypto.
let useNativeSigner = async (client: t, secretKey: Uint8Array.t): unit => {
  let keypair = await SolanaKitKeys.createKeyPairFromBytes(secretKey)
  let signer = await SolanaKitKeys.createKeyPairSignerFromBytes(secretKey)
  client.signingStrategy = Some(NativeSigner({keypair, signer, address: SolanaKitKeys.signerAddress(signer)}))
}

let clearAuth = (client: t): unit => Http.clearAuthToken(client.http)
let authToken = (client: t): option<string> => Http.authToken(client.http)
