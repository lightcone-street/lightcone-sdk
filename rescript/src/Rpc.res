// On-chain read sub-client over the client's kit RPC. Mirrors rust/src/rpc.rs (the
// `Rpc` sub-client) plus the per-domain `get_onchain` / `current_nonce` fetchers.
// Every call returns `promise<result<_, SdkError.t>>`; JS/RPC exceptions are caught
// and wrapped as `SdkError.Other`, decode failures propagate the `SdkError.Program`
// from `Accounts`.
//
// The two transport primitives (`getLatestBlockhash` / `getAccountData`) route
// through `RpcFailover.withFailover`: try the active endpoint, fast-retry, then fail
// over to `client.backupRpc` (mirrors rust/src/rpc_failover.rs). Their only failures
// are transport-level, so every error is treated as infrastructure; the typed
// fetchers below build on `getAccountData` and inherit failover for free.

// ── Response navigation (kit returns typed objects, not plain JSON) ────────────
// getAccountInfo(encoding=base64) → { value: { data: [base64, "base64"], … } | null }.
// Returns the base64 data string, or `None` (→ undefined) when the account is
// absent (`value === null`).
let accountDataBase64: JSON.t => option<string> = %raw(`function (response) {
  const value = response && response.value;
  if (value == null) return undefined;
  const data = value.data;
  return (Array.isArray(data) && typeof data[0] === "string") ? data[0] : undefined;
}`)

// getLatestBlockhash → { value: { blockhash: string, lastValidBlockHeight: bigint } }.
let blockhashOf: JSON.t => option<string> = %raw(`function (response) {
  const value = response && response.value;
  return (value && typeof value.blockhash === "string") ? value.blockhash : undefined;
}`)

// base64 → bytes (Node/Bun `Buffer`, browser `atob` fallback).
let base64ToBytes: string => Uint8Array.t = %raw(`function (b64) {
  if (typeof Buffer !== "undefined") return new Uint8Array(Buffer.from(b64, "base64"));
  const binary = atob(b64);
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) out[i] = binary.charCodeAt(i);
  return out;
}`)

let exnMessage = (error: JsExn.t): string => error->JsExn.message->Option.getOr("rpc error")

// ── Endpoint selection + failover ───────────────────────────────────────────────
// The kit RPC for a failover target. `backupRpc` is always a real handle (it falls
// back to the primary when no backup URL is set), so this never produces `None` —
// and a kit RPC must never be wrapped in an `option` anyway: it is a Proxy that
// answers truthy for every property, which corrupts ReScript's boxed-option tag.
let rpcFor = (client: Client.t, target: RpcFailover.activeRpc): SolanaKitRpc.t =>
  switch target {
  | Primary => client.rpc
  | Backup => client.backupRpc
  }

// Which endpoint is currently live (Primary until a failover flips it to Backup).
let activeRpc = (client: Client.t): RpcFailover.activeRpc => RpcFailover.active(client.rpcFailover)

// ── Blockhash ──────────────────────────────────────────────────────────────────
// Latest blockhash for transaction building (with primary→backup failover).
let getLatestBlockhashOn = async (rpc: SolanaKitRpc.t): result<string, SdkError.t> =>
  switch await SolanaKitRpc.getLatestBlockhash(rpc)->SolanaKitRpc.send {
  | response =>
    switch blockhashOf(response) {
    | Some(blockhash) => Ok(blockhash)
    | None => Error(SdkError.Other("getLatestBlockhash: response missing blockhash"))
    }
  | exception JsExn(error) => Error(SdkError.Other(`getLatestBlockhash failed: ${exnMessage(error)}`))
  }

let getLatestBlockhash = (client: Client.t): promise<result<string, SdkError.t>> =>
  RpcFailover.withFailover(
    client.rpcFailover,
    ~hasBackup=client.backupRpcUrl->Option.isSome,
    ~tryOn=target => getLatestBlockhashOn(rpcFor(client, target)),
  )

// ── Raw account data ────────────────────────────────────────────────────────────
// Account DATA bytes, or `None` when the account doesn't exist (with failover).
let getAccountDataOn = async (
  rpc: SolanaKitRpc.t,
  address: SolanaKit.address,
): result<option<Uint8Array.t>, SdkError.t> =>
  switch await SolanaKitRpc.getAccountInfo(rpc, address, {"encoding": "base64"})->SolanaKitRpc.send {
  | response =>
    switch accountDataBase64(response) {
    | Some(base64) => Ok(Some(base64ToBytes(base64)))
    | None => Ok(None)
    }
  | exception JsExn(error) => Error(SdkError.Other(`getAccountInfo failed: ${exnMessage(error)}`))
  }

let getAccountData = (
  client: Client.t,
  address: SolanaKit.address,
): promise<result<option<Uint8Array.t>, SdkError.t>> =>
  RpcFailover.withFailover(
    client.rpcFailover,
    ~hasBackup=client.backupRpcUrl->Option.isSome,
    ~tryOn=target => getAccountDataOn(rpcFor(client, target), address),
  )

// ── PDA accessors (thin wrappers binding `client.programId`) ──────────────────
// Async (kit derives PDAs via WebCrypto SHA-256); each returns just the address,
// matching the Rust `Rpc::get_*_pda` helpers (the bump is dropped — use `Pda.*`
// directly when you need it).
let exchangePda = async (client: Client.t): SolanaKit.address => {
  let (address, _bump) = await Pda.exchange(client.programId)
  address
}

let marketPda = async (client: Client.t, ~marketId: bigint): SolanaKit.address => {
  let (address, _bump) = await Pda.market(client.programId, ~marketId)
  address
}

let positionPda = async (
  client: Client.t,
  ~owner: SolanaKit.address,
  ~market: SolanaKit.address,
): SolanaKit.address => {
  let (address, _bump) = await Pda.position(client.programId, ~owner, ~market)
  address
}

let userNoncePda = async (client: Client.t, ~user: SolanaKit.address): SolanaKit.address => {
  let (address, _bump) = await Pda.userNonce(client.programId, ~user)
  address
}

let orderbookPda = async (
  client: Client.t,
  ~mintA: SolanaKit.address,
  ~mintB: SolanaKit.address,
): SolanaKit.address => {
  let (address, _bump) = await Pda.orderbook(client.programId, ~mintA, ~mintB)
  address
}

let globalDepositTokenPda = async (client: Client.t, ~mint: SolanaKit.address): SolanaKit.address => {
  let (address, _bump) = await Pda.globalDepositToken(client.programId, ~mint)
  address
}

let userGlobalDepositPda = async (
  client: Client.t,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
): SolanaKit.address => {
  let (address, _bump) = await Pda.userGlobalDeposit(client.programId, ~user, ~mint)
  address
}

// ── Typed account fetchers ────────────────────────────────────────────────────
// Fetch + decode the singleton Exchange account.
let getExchange = async (client: Client.t): result<Accounts.exchange, SdkError.t> => {
  let pda = await exchangePda(client)
  switch await getAccountData(client, pda) {
  | Error(error) => Error(error)
  | Ok(None) => Error(SdkError.Program("Exchange: account not found"))
  | Ok(Some(bytes)) => Accounts.decodeExchange(bytes)
  }
}

// Fetch + decode a Market by its on-chain pubkey.
let getMarket = async (client: Client.t, market: SolanaKit.address): result<Accounts.market, SdkError.t> =>
  switch await getAccountData(client, market) {
  | Error(error) => Error(error)
  | Ok(None) => Error(SdkError.Program("Market: account not found"))
  | Ok(Some(bytes)) => Accounts.decodeMarket(bytes)
  }

// Fetch + decode an Orderbook by its (canonical-sorted) conditional mints.
let getOrderbook = async (
  client: Client.t,
  ~mintA: SolanaKit.address,
  ~mintB: SolanaKit.address,
): result<Accounts.orderbook, SdkError.t> => {
  let pda = await orderbookPda(client, ~mintA, ~mintB)
  switch await getAccountData(client, pda) {
  | Error(error) => Error(error)
  | Ok(None) => Error(SdkError.Program("Orderbook: account not found"))
  | Ok(Some(bytes)) => Accounts.decodeOrderbook(bytes)
  }
}

// Fetch + decode a Position; `None` when it doesn't exist (matches Rust).
let getPosition = async (
  client: Client.t,
  ~owner: SolanaKit.address,
  ~market: SolanaKit.address,
): result<option<Accounts.position>, SdkError.t> => {
  let pda = await positionPda(client, ~owner, ~market)
  switch await getAccountData(client, pda) {
  | Error(error) => Error(error)
  | Ok(None) => Ok(None)
  | Ok(Some(bytes)) => Accounts.decodePosition(bytes)->Result.map(position => Some(position))
  }
}

// ── Nonce ───────────────────────────────────────────────────────────────────────
// Current on-chain nonce for a user; 0 when the account is uninitialized
// (matches the Rust `Orders::get_nonce`).
let getNonce = async (client: Client.t, ~user: SolanaKit.address): result<float, SdkError.t> => {
  let pda = await userNoncePda(client, ~user)
  switch await getAccountData(client, pda) {
  | Error(error) => Error(error)
  | Ok(None) => Ok(0.0)
  | Ok(Some(bytes)) =>
    switch Accounts.decodeUserNonce(bytes) {
    | Error(error) => Error(error)
    | Ok(userNonce) => Ok(BigInt.toFloat(userNonce.nonce))
    }
  }
}
