// Orders — submit, cancel, cancel-all, plus the authenticated user's open-order
// snapshot and on-chain nonce. Public surface: the gentype response / request /
// body types, the `SubmitOrderStatus` enum, and the client functions. The JSON
// body builders (`bigToJson` / `bodyOfRequest`) and the cancel-message signing
// helpers (`utf8` / `randomUuid` / `nowSeconds` / `cancelAllMessage`) are internal.

// ── Response types ────────────────────────────────────────────────────────────
@genType
type fillInfo = {
  counterparty: Shared.pubkeyStr,
  counterpartyOrderHash: string,
  fillAmount: string,
  price: string,
  isMaker: bool,
}

module SubmitOrderStatus: {
  @genType
  type t =
    | @as("accepted") Accepted
    | @as("partial_fill") PartialFill
    | @as("filled") Filled
}

@genType
type submitOrderResponse = {
  orderHash: string,
  status: SubmitOrderStatus.t,
  remaining: string,
  filled: string,
  fills: array<fillInfo>,
}

@genType
type cancelSuccess = {
  orderHash: string,
  remaining: string,
}

@genType
type cancelAllSuccess = {
  cancelledOrderHashes: array<string>,
  count: float,
  userPubkey: Shared.pubkeyStr,
  orderbookId: Shared.orderBookId,
  message: string,
}

// ── Submit ────────────────────────────────────────────────────────────────────
// The signed order, ready to POST. Built by `Envelope.buildLimitOrder`.
@genType
type submitOrderRequest = {
  maker: string,
  nonce: bigint,
  salt: bigint,
  marketPubkey: string,
  baseToken: string,
  quoteToken: string,
  // 0 = Bid, 1 = Ask.
  side: int,
  amountIn: bigint,
  amountOut: bigint,
  expiration: bigint,
  // hex-encoded 64-byte ed25519 signature.
  signatureHex: string,
  orderbookId: string,
  timeInForce?: Shared.TimeInForce.t,
  depositSource?: Shared.DepositSource.t,
}

// ── Cancel ────────────────────────────────────────────────────────────────────
@genType
type cancelBody = {orderHash: string, maker: string, signatureHex: string}

@genType
type cancelAllBody = {
  userPubkey: string,
  orderbookId: string,
  signatureHex: string,
  // Unix seconds.
  timestamp: float,
  salt: string,
}

// ── User orders snapshot ──────────────────────────────────────────────────────
// A flat record decoding both limit + trigger orders (trigger-only fields optional).
@genType
type userSnapshotOrder = {
  orderType: string,
  orderHash: string,
  marketPubkey: Shared.pubkeyStr,
  orderbookId: Shared.orderBookId,
  side: Shared.Side.t,
  amountIn: string,
  amountOut: string,
  remaining: string,
  filled: string,
  price: string,
  createdAt: float,
  expiration: float,
  baseMint: Shared.pubkeyStr,
  quoteMint: Shared.pubkeyStr,
  outcomeIndex: float,
  txSignature?: string,
  triggerOrderId?: string,
  triggerPrice?: string,
}

@genType
type userOutcomeBalance = {
  outcomeIndex: float,
  conditionalToken: Shared.pubkeyStr,
  balance: string,
  balanceIdle: string,
  balanceOnBook: string,
}

@genType
type userDepositAssetBalance = {
  depositAsset: Shared.pubkeyStr,
  outcomes: array<userOutcomeBalance>,
}

@genType
type userMarketBalance = {
  marketPubkey: Shared.pubkeyStr,
  depositAssets: array<userDepositAssetBalance>,
}

@genType
type userOrdersResponse = {
  userPubkey: Shared.pubkeyStr,
  orders: array<userSnapshotOrder>,
  marketBalances: array<userMarketBalance>,
  nextCursor?: string,
  hasMore: bool,
}

// ── Client functions ──────────────────────────────────────────────────────────
// POST a signed order (built by `Envelope.buildLimitOrder`). Never retried.
let submit: (Client.t, submitOrderRequest) => promise<result<submitOrderResponse, SdkError.t>>

// Sign a cancel for one order (signs the order-hash hex string's UTF-8 bytes).
let cancelBodySigned: (
  ~orderHash: string,
  ~maker: string,
  ~keypair: SolanaKit.cryptoKeyPair,
) => promise<cancelBody>

// Sign a cancel-all for one orderbook (signs `cancel_all:…` UTF-8 bytes).
let cancelAllBodySigned: (
  ~userPubkey: string,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
) => promise<cancelAllBody>

// Cancel a single order. Never retried.
let cancel: (Client.t, cancelBody) => promise<result<cancelSuccess, SdkError.t>>

// Cancel every open order on one orderbook. Never retried.
let cancelAll: (Client.t, cancelAllBody) => promise<result<cancelAllSuccess, SdkError.t>>

// The authenticated user's open orders (wallet resolved server-side from the cookie).
let getUserOrders: (
  Client.t,
  ~limit: int=?,
  ~cursor: string=?,
  ~cookieHeader: string=?,
) => promise<result<userOrdersResponse, SdkError.t>>

// The authenticated user's current on-chain nonce (0 if uninitialized).
let getNonce: (Client.t, ~user: SolanaKit.address) => promise<result<float, SdkError.t>>
