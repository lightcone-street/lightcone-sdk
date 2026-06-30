// The SDK client — HTTP transport, environment, program id, RPC handle, and mutable
// trading state. It is the shared context threaded through every SDK module, so the
// record stays exposed to ReScript; `@genType.opaque` keeps it an opaque handle to
// TypeScript. Construct with `make`; the grouped ergonomics live in the facade.

// How orders / cancels / transactions get signed. `None` ⇒ manual signing.
type signingStrategy =
  | NativeSigner({
      keypair: SolanaKit.cryptoKeyPair,
      signer: SolanaKit.keyPairSigner,
      address: SolanaKit.address,
    })

@genType.opaque
type t = {
  http: Http.t,
  env: Env.t,
  programId: SolanaKit.address,
  wsUrl: string,
  rpcUrl: string,
  backupRpcUrl: option<string>,
  rpc: SolanaKitRpc.t,
  mutable depositSource: Shared.DepositSource.t,
  mutable orderNonce: option<bigint>,
  mutable signingStrategy: option<signingStrategy>,
}

// Build a client. `env` defaults to Prod; per-field overrides win, then `SDK_*` env
// vars (inside `Env`), then the built-in defaults.
let make: (
  ~env: Env.t=?,
  ~baseUrl: string=?,
  ~wsUrl: string=?,
  ~rpcUrl: string=?,
  ~backupRpcUrl: string=?,
  ~programId: string=?,
  ~depositSource: Shared.DepositSource.t=?,
  unit,
) => t

let http: t => Http.t
let depositSource: t => Shared.DepositSource.t
let setDepositSource: (t, Shared.DepositSource.t) => unit
let orderNonce: t => option<bigint>
let setOrderNonce: (t, bigint) => unit
let signerAddress: t => option<SolanaKit.address>

// Attach a native ed25519 signer from a 64-byte wallet secret (Solana id.json format).
let useNativeSigner: (t, Uint8Array.t) => promise<unit>

let clearAuth: t => unit
let authToken: t => option<string>
