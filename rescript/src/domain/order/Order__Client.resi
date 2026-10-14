// Order REST client — submit, cancel, cancel-all, and the user order/fill read
// paths over the `Order__Raw` wire types. The signing-message builders and the
// query helper are internal.

// ── Submit ────────────────────────────────────────────────────────────────────
// POST a signed order (built by `Envelope.buildLimitOrder`). Never retried.
let submit: (
  Client.t,
  Order__Raw.SubmitRequest.t,
) => promise<result<Order__Raw.SubmitResponse.t, SdkError.t>>

let cancelBodySigned: (
  ~orderHash: string,
  ~maker: string,
  ~keypair: SolanaKit.cryptoKeyPair,
) => promise<Order__Raw.CancelBody.t>

// Sign a cancel-all for one orderbook (signs `cancel_all:…` UTF-8 bytes).
let cancelAllBodySigned: (
  ~userPubkey: string,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
) => promise<Order__Raw.CancelAllBody.t>

// Cancel a single order. Never retried.
let cancel: (
  Client.t,
  Order__Raw.CancelBody.t,
) => promise<result<Order__Raw.CancelSuccess.t, SdkError.t>>

// Cancel every open order on one orderbook. Never retried.
let cancelAll: (
  Client.t,
  Order__Raw.CancelAllBody.t,
) => promise<result<Order__Raw.CancelAllSuccess.t, SdkError.t>>

let cancelSigned: (
  Client.t,
  ~orderHash: string,
) => promise<result<Order__Raw.CancelSuccess.t, SdkError.t>>
let cancelAllSigned: (
  Client.t,
  ~orderbookId: string,
) => promise<result<Order__Raw.CancelAllSuccess.t, SdkError.t>>
let getUserOrders: (
  Client.t,
  ~limit: int=?,
  ~cursor: string=?,
  ~cookieHeader: string=?,
) => promise<result<Order__Raw.UserOrdersResponse.t, SdkError.t>>

// The authenticated user's filled orders with nested fill events (maker or taker;
// most recent fill first). Optionally filter by market.
let getUserOrderFills: (
  Client.t,
  ~marketPubkey: string=?,
  ~limit: int=?,
  ~cursor: string=?,
  ~cookieHeader: string=?,
) => promise<result<Order__Raw.UserFillsResponse.t, SdkError.t>>

// Public variant: takes the wallet via the URL path, requires no auth.
let getUserOrderFillsByWallet: (
  Client.t,
  ~walletAddress: string,
  ~marketPubkey: string=?,
  ~limit: int=?,
  ~cursor: string=?,
) => promise<result<Order__Raw.UserFillsResponse.t, SdkError.t>>

// ── Onchain reads ─────────────────────────────────────────────────────────────
// The authenticated user's current on-chain nonce (0 if uninitialized).
let getNonce: (Client.t, ~user: SolanaKit.address) => promise<result<float, SdkError.t>>

// An order's on-chain OrderStatus account (`None` once fully filled + closed,
// or never created). `~orderHash` is the 32-byte keccak digest.
let getStatus: (
  Client.t,
  ~orderHash: Uint8Array.t,
) => promise<result<option<Accounts.OrderStatus.t>, SdkError.t>>
