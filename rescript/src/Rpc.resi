// On-chain read sub-client over the client's kit RPC (@solana/kit). Public surface:
// latest blockhash, raw account data, PDA derivation, the typed account fetchers, and
// the user nonce — the blockhash + account reads fail over primary→backup. The
// response-shape `%raw` helpers, base64 decoding, and per-endpoint variants are internal.

// The endpoint currently serving reads (Primary until a failover flips it to Backup).
let activeRpc: Client.t => RpcFailover.activeRpc

let getLatestBlockhash: Client.t => promise<result<string, SdkError.t>>
// Account DATA bytes, or `None` when the account doesn't exist.
let getAccountData: (Client.t, SolanaKit.address) => promise<result<option<Uint8Array.t>, SdkError.t>>

// PDA accessors (return just the address; use `Pda.*` directly when you need the bump).
let exchangePda: Client.t => promise<SolanaKit.address>
let marketPda: (Client.t, ~marketId: bigint) => promise<SolanaKit.address>
let positionPda: (
  Client.t,
  ~owner: SolanaKit.address,
  ~market: SolanaKit.address,
) => promise<SolanaKit.address>
let userNoncePda: (Client.t, ~user: SolanaKit.address) => promise<SolanaKit.address>
let orderbookPda: (
  Client.t,
  ~mintA: SolanaKit.address,
  ~mintB: SolanaKit.address,
) => promise<SolanaKit.address>
let globalDepositTokenPda: (Client.t, ~mint: SolanaKit.address) => promise<SolanaKit.address>
let userGlobalDepositPda: (
  Client.t,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
) => promise<SolanaKit.address>

// Typed account fetchers (fetch + decode; verify discriminator + byte layout).
let getExchange: Client.t => promise<result<Accounts.exchange, SdkError.t>>
let getMarket: (Client.t, SolanaKit.address) => promise<result<Accounts.market, SdkError.t>>
let getOrderbook: (
  Client.t,
  ~mintA: SolanaKit.address,
  ~mintB: SolanaKit.address,
) => promise<result<Accounts.orderbook, SdkError.t>>
let getPosition: (
  Client.t,
  ~owner: SolanaKit.address,
  ~market: SolanaKit.address,
) => promise<result<option<Accounts.position>, SdkError.t>>

// Current on-chain nonce for a user; 0 when the account is uninitialized.
let getNonce: (Client.t, ~user: SolanaKit.address) => promise<result<float, SdkError.t>>
