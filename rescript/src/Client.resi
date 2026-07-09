// The SDK client — HTTP transport, environment, program id, RPC handles, failover state, and
// mutable trading state. Exported structurally to TypeScript (`@genType`); only the foreign
// handles it holds are individually opaque (the HTTP transport, the kit RPC clients, and the
// signer's WebCrypto keys / kit signer). Construct with `make`; ergonomics live in the facade.

// An external wallet signer (browser wallet adapter): the wallet's address plus
// its message- and transaction-signing callbacks. `signMessage` returns the raw
// 64-byte ed25519 signature; `signTransaction` takes the serialized unsigned
// wire transaction and returns the fully signed wire bytes.
module ExternalSigner: {
  @genType
  type t = {
    address: SolanaKit.address,
    signMessage: Uint8Array.t => promise<Uint8Array.t>,
    signTransaction: Uint8Array.t => promise<Uint8Array.t>,
  }
}

// How orders / cancels / transactions get signed. `None` ⇒ manual signing.
// (The `ExternalSigner` constructor and the module above deliberately share the name.)
module SigningStrategy: {
  @genType
  type t =
    | NativeSigner({
        keypair: SolanaKit.cryptoKeyPair,
        signer: SolanaKit.keyPairSigner,
        address: SolanaKit.address,
      })
    | ExternalSigner(ExternalSigner.t)
}

@genType
type t = {
  http: Http.t,
  env: Env.t,
  programId: SolanaKit.address,
  wsUrl: string,
  rpcUrl: string,
  backupRpcUrl: option<string>,
  rpc: SolanaKitRpc.t,
  backupRpc: SolanaKitRpc.t,
  rpcFailover: RpcFailover.t,
  mutable depositSource: Shared.DepositSource.t,
  mutable orderNonce: option<bigint>,
  mutable signingStrategy: option<SigningStrategy.t>,
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
let clearOrderNonce: t => unit
let signerAddress: t => option<SolanaKit.address>

// Attach a native ed25519 signer from a 64-byte wallet secret (Solana id.json format).
let useNativeSigner: (t, Uint8Array.t) => promise<unit>

// Attach an external wallet signer (browser wallet adapter).
let useExternalSigner: (
  t,
  ~address: SolanaKit.address,
  ~signMessage: Uint8Array.t => promise<Uint8Array.t>,
  ~signTransaction: Uint8Array.t => promise<Uint8Array.t>,
) => unit

let clearSigningStrategy: t => unit

// Sign raw message bytes with the configured strategy (orders, cancels, and the
// login message all sign this way).
let signMessageBytes: (t, Uint8Array.t) => promise<result<Uint8Array.t, SdkError.t>>

let clearAuth: t => unit
let authToken: t => option<string>
