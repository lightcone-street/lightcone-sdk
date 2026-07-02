// Orders — submit, cancel, cancel-all, the user order/fill read paths, and the
// order wire types shared by REST and WS (user snapshot, order events). Public
// surface: the gentype types, the enums, the client functions, and the decoders
// consumed by `ws/Messages.res` / tests. The JSON body builders, signing helpers,
// and the remaining spice codecs are internal.

// ── OrderType ─────────────────────────────────────────────────────────────────
// Kind of order for display / classification; snake_case on the wire.
module OrderType: {
  @genType @spice
  type t =
    | @as("limit") @spice.as("limit") Limit
    | @as("market") @spice.as("market") Market
    | @as("deposit") @spice.as("deposit") Deposit
    | @as("merge") @spice.as("merge") Merge
    | @as("withdraw") @spice.as("withdraw") Withdraw
    | @as("stop_limit") @spice.as("stop_limit") StopLimit
    | @as("take_profit_limit") @spice.as("take_profit_limit") TakeProfitLimit

  // Human-readable label ("Stop Limit", …).
  let label: t => string
}

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
// The signed order, ready to POST. Built by `Envelope.buildLimitOrder` /
// `buildTriggerOrder` (a trigger order carries the two trigger fields).
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
  // Trigger-only fields (the wire sends `trigger_price` as a JSON number).
  triggerPrice?: float,
  triggerType?: Shared.TriggerType.t,
}

// ── Trigger orders ────────────────────────────────────────────────────────────
@genType
type triggerOrderResponse = {
  triggerOrderId: string,
  orderHash: string,
}

@genType
type cancelTriggerSuccess = {
  triggerOrderId: string,
}

@genType
type cancelTriggerBody = {triggerOrderId: string, maker: string, signatureHex: string}

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
// The backend returns limit + trigger orders in one array, discriminated by
// `order_type`, with the common fields flattened into the same object. Used by
// both REST `GET /api/users/orders` and the WS user snapshot.

// Fields shared by limit and trigger order snapshots.
@genType
type userSnapshotOrderCommon = {
  orderHash: string,
  marketPubkey: Shared.pubkeyStr,
  orderbookId: Shared.orderBookId,
  side: Shared.Side.t,
  amountIn: string,
  amountOut: string,
  remaining: string,
  filled: string,
  price: string,
  // Unix milliseconds.
  createdAt: float,
  expiration: float,
  baseMint: Shared.pubkeyStr,
  quoteMint: Shared.pubkeyStr,
  outcomeIndex: float,
  status: Shared.OrderStatus.t,
}

// Order snapshot — tagged on `order_type` ("limit" | "trigger").
module UserSnapshotOrder: {
  @genType
  type limit = {
    common: userSnapshotOrderCommon,
    txSignature?: string,
  }

  @genType
  type trigger = {
    common: userSnapshotOrderCommon,
    triggerOrderId: string,
    triggerPrice: string,
    triggerType: Shared.TriggerType.t,
    timeInForce?: Shared.TimeInForce.t,
  }

  @genType
  type t =
    | Limit(limit)
    | Trigger(trigger)

  // The fields shared by both variants.
  let common: t => userSnapshotOrderCommon

  // Spice-convention codecs (also embeddable in spice records; `t_encodeJson`
  // is spice ≥0.4's fixed-JSON-context alias of `t_encode`).
  let t_decode: JSON.t => result<t, Spice.decodeError>
  let t_encode: t => JSON.t
  let t_encodeJson: t => JSON.t
}

@genType
type userOutcomeBalance = {
  outcomeIndex: float,
  conditionalToken: Shared.pubkeyStr,
  balance: string,
  balanceIdle: string,
  balanceOnBook: string,
}

// Neither idle nor on-book balance is positive (Rust `UserOutcomeBalance::is_zero`).
let userOutcomeBalanceIsZero: userOutcomeBalance => bool

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

// Exposed for `ws/Messages.res` (the WS balance update carries the same tree).
let userMarketBalance_decode: JSON.t => result<userMarketBalance, Spice.decodeError>

@genType
type userOrdersResponse = {
  userPubkey: Shared.pubkeyStr,
  orders: array<UserSnapshotOrder.t>,
  marketBalances: array<userMarketBalance>,
  nextCursor?: string,
  hasMore: bool,
}

// ── User order fills (REST) ───────────────────────────────────────────────────
// Whether the user was the maker or taker on an order.
module Role: {
  @genType @spice
  type t = | @as("maker") @spice.as("maker") Maker | @as("taker") @spice.as("taker") Taker
}

// Status of a filled order, derived from DB state after the fact (distinct from
// `Shared.OrderStatus`, the engine's real-time state).
module FillStatus: {
  @genType @spice
  type t =
    | @as("filled") @spice.as("filled") Filled
    | @as("cancelled") @spice.as("cancelled") Cancelled
    | @as("partially_filled") @spice.as("partially_filled") PartiallyFilled
}

// A single fill event within an order.
@genType
type orderFillEvent = {
  fillAmount: string,
  txSignature: string,
  // Unix milliseconds.
  filledAt: float,
}

// An order the user participated in, with nested fill events.
@genType
type userOrderFill = {
  orderHash: string,
  marketPubkey: Shared.pubkeyStr,
  orderbookId: Shared.orderBookId,
  side: Shared.Side.t,
  role: Role.t,
  price: string,
  size: string,
  filledSize: string,
  remainingSize: string,
  baseMint: Shared.pubkeyStr,
  quoteMint: Shared.pubkeyStr,
  outcomeIndex: float,
  status: FillStatus.t,
  // Unix milliseconds.
  createdAt: float,
  fills: array<orderFillEvent>,
}

// Response from `GET /api/users/order-fills`.
@genType
type userOrderFillsResponse = {
  orders: array<userOrderFill>,
  nextCursor?: string,
  hasMore: bool,
}

// Exposed for tests (spice codec of the fills response).
let userOrderFillsResponse_decode: JSON.t => result<userOrderFillsResponse, Spice.decodeError>

// ── WS user order events ──────────────────────────────────────────────────────
// Payloads for the WS `user` channel's `order` / `snapshot` events; dispatched by
// `ws/Messages.res` and consumed by the stateful `OrderState` containers.

// Balance for a single conditional token, attached to WS order updates.
@genType
type conditionalBalance = {
  outcomeIndex: float,
  conditionalToken: Shared.pubkeyStr,
  idle: string,
  onBook: string,
}

@genType
type userOrderUpdateBalance = {outcomes: array<conditionalBalance>}

// Individual order within a WS update. `createdAt` is epoch milliseconds.
@genType
type wsOrder = {
  orderHash: string,
  price: string,
  isMaker: bool,
  remaining: string,
  filled: string,
  fillAmount: string,
  side: Shared.Side.t,
  createdAt: float,
  baseMint: Shared.pubkeyStr,
  quoteMint: Shared.pubkeyStr,
  outcomeIndex: float,
  status: Shared.OrderStatus.t,
  // Absent on cancellation events.
  balance?: userOrderUpdateBalance,
}

// WS limit-order update event. `timestamp` is ISO-8601.
@genType
type orderUpdate = {
  marketPubkey: Shared.pubkeyStr,
  orderbookId: Shared.orderBookId,
  timestamp: string,
  txSignature?: string,
  updateType: Shared.OrderUpdateType.t,
  order: wsOrder,
}

// Trigger-order WS update event on the user channel. Result fields are zero /
// absent until the order triggers.
@genType
type triggerOrderUpdate = {
  triggerOrderId: string,
  userPubkey: Shared.pubkeyStr,
  marketPubkey: Shared.pubkeyStr,
  orderbookId: Shared.orderBookId,
  triggerPrice: string,
  triggerAbove: bool,
  status: Shared.TriggerStatus.t,
  updateType: Shared.TriggerUpdateType.t,
  orderHash: string,
  side: Shared.Side.t,
  resultStatus?: Shared.TriggerResultStatus.t,
  resultFilled: string,
  resultRemaining: string,
  // ISO-8601.
  timestamp: string,
  makerAmount: string,
  takerAmount: string,
  tif: Shared.TimeInForce.t,
}

// WS order event — both limit and trigger updates arrive as `event_type: "order"`,
// discriminated by `order_type` at the same level.
module OrderEvent: {
  @genType
  type t =
    | Limit(orderUpdate)
    | Trigger(triggerOrderUpdate)

  let decode: JSON.t => result<t, Spice.decodeError>
}

// Global deposit balance for a single mint (WS snapshots).
@genType
type globalDepositBalance = {
  mint: Shared.pubkeyStr,
  balance: string,
}

// WS user snapshot — the full authenticated-user state pushed on subscribe.
@genType
type userSnapshot = {
  orders: array<UserSnapshotOrder.t>,
  marketBalances: array<userMarketBalance>,
  globalDeposits: array<globalDepositBalance>,
  notifications: array<Notification.notification>,
  nonce: float,
}

let userSnapshotDecode: JSON.t => result<userSnapshot, Spice.decodeError>

// Exposed for tests (the WS trigger update is hand-decoded).
let triggerOrderUpdateDecode: JSON.t => result<triggerOrderUpdate, Spice.decodeError>

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

// Submit a signed trigger order (request built by `Envelope.buildTriggerOrder`).
// Never retried.
let submitTrigger: (
  Client.t,
  submitOrderRequest,
) => promise<result<triggerOrderResponse, SdkError.t>>

// Sign a cancel for one trigger order (signs the trigger order id's UTF-8 bytes).
let cancelTriggerBodySigned: (
  ~triggerOrderId: string,
  ~maker: string,
  ~keypair: SolanaKit.cryptoKeyPair,
) => promise<cancelTriggerBody>

// Cancel a resting trigger order. Never retried.
let cancelTrigger: (
  Client.t,
  cancelTriggerBody,
) => promise<result<cancelTriggerSuccess, SdkError.t>>

// ── Strategy-signed cancels ───────────────────────────────────────────────────
// Each signs with the client's configured strategy (native keypair or external
// wallet adapter), the signer's address acting as maker.
let cancelSigned: (Client.t, ~orderHash: string) => promise<result<cancelSuccess, SdkError.t>>
let cancelAllSigned: (
  Client.t,
  ~orderbookId: string,
) => promise<result<cancelAllSuccess, SdkError.t>>
let cancelTriggerSigned: (
  Client.t,
  ~triggerOrderId: string,
) => promise<result<cancelTriggerSuccess, SdkError.t>>

// The authenticated user's open orders (wallet resolved server-side from the cookie).
let getUserOrders: (
  Client.t,
  ~limit: int=?,
  ~cursor: string=?,
  ~cookieHeader: string=?,
) => promise<result<userOrdersResponse, SdkError.t>>

// The authenticated user's filled orders with nested fill events (maker or taker;
// most recent fill first). Optionally filter by market.
let getUserOrderFills: (
  Client.t,
  ~marketPubkey: string=?,
  ~limit: int=?,
  ~cursor: string=?,
  ~cookieHeader: string=?,
) => promise<result<userOrderFillsResponse, SdkError.t>>

// Public variant: takes the wallet via the URL path, requires no auth.
let getUserOrderFillsByWallet: (
  Client.t,
  ~walletAddress: string,
  ~marketPubkey: string=?,
  ~limit: int=?,
  ~cursor: string=?,
) => promise<result<userOrderFillsResponse, SdkError.t>>

// The authenticated user's current on-chain nonce (0 if uninitialized).
let getNonce: (Client.t, ~user: SolanaKit.address) => promise<result<float, SdkError.t>>
