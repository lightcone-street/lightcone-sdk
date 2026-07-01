// Orders — submit, cancel, cancel-all. The order is signed off-chain (OrderPayload)
// and POSTed; cancels sign a message and POST the hex signature. Mirrors the Rust
// domain/order write path.
//
// Note: `getNonce` reads the on-chain UserNonce PDA via the RPC/Accounts layer
// (delegates to `Rpc.getNonce`). The envelope still defaults nonce to 0 when the
// caller doesn't supply one — fetch a fresh value via `getNonce` for live submission.

// ── Response types ────────────────────────────────────────────────────────────
@spice
type fillInfo = {
  counterparty: Shared.pubkeyStr,
  @spice.key("counterparty_order_hash") counterpartyOrderHash: string,
  @spice.key("fill_amount") fillAmount: string,
  price: string,
  @spice.key("is_maker") isMaker: bool,
}

module SubmitOrderStatus = {
  @spice
  type t =
    | @as("accepted") @spice.as("accepted") Accepted
    | @as("partial_fill") @spice.as("partial_fill") PartialFill
    | @as("filled") @spice.as("filled") Filled
}

@spice
type submitOrderResponse = {
  @spice.key("order_hash") orderHash: string,
  status: SubmitOrderStatus.t,
  remaining: string,
  filled: string,
  fills: array<fillInfo>,
}

@spice
type cancelSuccess = {
  @spice.key("order_hash") orderHash: string,
  remaining: string,
}

@spice
type cancelAllSuccess = {
  @spice.key("cancelled_order_hashes") cancelledOrderHashes: array<string>,
  count: float,
  @spice.key("user_pubkey") userPubkey: Shared.pubkeyStr,
  @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
  message: string,
}

// ── Submit ────────────────────────────────────────────────────────────────────
// The signed order, ready to POST. Built by `Envelope.buildLimitOrder`.
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

let bigToJson = (value: bigint): JSON.t => JSON.Number(BigInt.toFloat(value))

let bodyOfRequest = (request: submitOrderRequest): JSON.t => {
  let fields = [
    ("maker", JSON.String(request.maker)),
    ("nonce", bigToJson(request.nonce)),
    ("salt", bigToJson(request.salt)),
    ("market_pubkey", JSON.String(request.marketPubkey)),
    ("base_token", JSON.String(request.baseToken)),
    ("quote_token", JSON.String(request.quoteToken)),
    ("side", JSON.Number(Int.toFloat(request.side))),
    ("amount_in", bigToJson(request.amountIn)),
    ("amount_out", bigToJson(request.amountOut)),
    ("expiration", bigToJson(request.expiration)),
    ("signature", JSON.String(request.signatureHex)),
    ("orderbook_id", JSON.String(request.orderbookId)),
  ]
  request.timeInForce->Option.forEach(tif => fields->Array.push(("tif", Shared.TimeInForce.t_encode(tif))))
  request.depositSource->Option.forEach(source =>
    fields->Array.push(("deposit_source", Shared.DepositSource.t_encode(source)))
  )
  JSON.Object(Dict.fromArray(fields))
}

let submit = async (client: Client.t, request: submitOrderRequest): result<submitOrderResponse, SdkError.t> =>
  await Http.post(
    client.http,
    ~path="/api/orders/submit",
    ~body=bodyOfRequest(request),
    ~retry=NoRetry,
    ~decode=submitOrderResponse_decode,
  )

// ── Cancel ────────────────────────────────────────────────────────────────────
type cancelBody = {orderHash: string, maker: string, signatureHex: string}

type cancelAllBody = {
  userPubkey: string,
  orderbookId: string,
  signatureHex: string,
  // Unix seconds.
  timestamp: float,
  salt: string,
}

let utf8 = text => SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), text)
let nowSeconds = (): float => Math.floor(Date.now() /. 1000.0)

// The cancel signing messages (UTF-8 bytes, ed25519-signed, hex-encoded).
let cancelAllMessage = (~userPubkey, ~orderbookId, ~timestamp: float, ~salt): string =>
  `cancel_all:${userPubkey}:${orderbookId}:${Float.toString(timestamp)}:${salt}`

// Sign a cancel for one order: signs the order-hash hex string's UTF-8 bytes.
let cancelBodySigned = async (
  ~orderHash: string,
  ~maker: string,
  ~keypair: SolanaKit.cryptoKeyPair,
): cancelBody => {
  let signature = await SolanaKitKeys.signBytes(keypair.privateKey, utf8(orderHash))
  {orderHash, maker, signatureHex: OrderPayload.signatureHex(signature)}
}

let cancelAllBodySigned = async (
  ~userPubkey: string,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
): cancelAllBody => {
  let timestamp = nowSeconds()
  let salt = WebCrypto.randomUUID()
  let message = utf8(cancelAllMessage(~userPubkey, ~orderbookId, ~timestamp, ~salt))
  let signature = await SolanaKitKeys.signBytes(keypair.privateKey, message)
  {userPubkey, orderbookId, signatureHex: OrderPayload.signatureHex(signature), timestamp, salt}
}

let cancel = async (client: Client.t, body: cancelBody): result<cancelSuccess, SdkError.t> => {
  let json = JSON.Object(
    Dict.fromArray([
      ("order_hash", JSON.String(body.orderHash)),
      ("maker", JSON.String(body.maker)),
      ("signature", JSON.String(body.signatureHex)),
    ]),
  )
  await Http.post(client.http, ~path="/api/orders/cancel", ~body=json, ~retry=NoRetry, ~decode=cancelSuccess_decode)
}

let cancelAll = async (client: Client.t, body: cancelAllBody): result<cancelAllSuccess, SdkError.t> => {
  let json = JSON.Object(
    Dict.fromArray([
      ("user_pubkey", JSON.String(body.userPubkey)),
      ("orderbook_id", JSON.String(body.orderbookId)),
      ("signature", JSON.String(body.signatureHex)),
      ("timestamp", JSON.Number(body.timestamp)),
      ("salt", JSON.String(body.salt)),
    ]),
  )
  await Http.post(
    client.http,
    ~path="/api/orders/cancel-all",
    ~body=json,
    ~retry=NoRetry,
    ~decode=cancelAllSuccess_decode,
  )
}

// ── User orders snapshot ──────────────────────────────────────────────────────
// The backend returns limit + trigger orders in one array, discriminated by
// `order_type` with the common fields flattened — so a single flat record decodes
// both (trigger-only fields are optional).
@spice
type userSnapshotOrder = {
  @spice.key("order_type") orderType: string,
  @spice.key("order_hash") orderHash: string,
  @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
  @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
  side: Shared.Side.t,
  @spice.key("amount_in") amountIn: string,
  @spice.key("amount_out") amountOut: string,
  @spice.default("0") remaining: string,
  @spice.default("0") filled: string,
  @spice.default("0") price: string,
  @spice.key("created_at") createdAt: float,
  @spice.default(0.0) expiration: float,
  @spice.key("base_mint") baseMint: Shared.pubkeyStr,
  @spice.key("quote_mint") quoteMint: Shared.pubkeyStr,
  @spice.key("outcome_index") outcomeIndex: float,
  @spice.key("tx_signature") txSignature?: string,
  @spice.key("trigger_order_id") triggerOrderId?: string,
  @spice.key("trigger_price") triggerPrice?: string,
}

@spice
type userOutcomeBalance = {
  @spice.key("outcome_index") outcomeIndex: float,
  @spice.key("conditional_token") conditionalToken: Shared.pubkeyStr,
  balance: string,
  @spice.key("balance_idle") balanceIdle: string,
  @spice.key("balance_on_book") balanceOnBook: string,
}

@spice
type userDepositAssetBalance = {
  @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
  outcomes: array<userOutcomeBalance>,
}

@spice
type userMarketBalance = {
  @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
  @spice.key("deposit_assets") depositAssets: array<userDepositAssetBalance>,
}

@spice
type userOrdersResponse = {
  @spice.key("user_pubkey") userPubkey: Shared.pubkeyStr,
  orders: array<userSnapshotOrder>,
  @spice.key("market_balances") marketBalances: array<userMarketBalance>,
  @spice.key("next_cursor") nextCursor?: string,
  @spice.default(false) @spice.key("has_more") hasMore: bool,
}

// The authenticated user's open orders (wallet resolved server-side from the cookie).
let getUserOrders = async (
  client: Client.t,
  ~limit: option<int>=?,
  ~cursor: option<string>=?,
  ~cookieHeader: option<string>=?,
): result<userOrdersResponse, SdkError.t> => {
  let query = []
  limit->Option.forEach(value => query->Array.push(("limit", Int.toString(value))))
  cursor->Option.forEach(value => query->Array.push(("cursor", value)))
  await Http.get(client.http, ~path="/api/users/orders", ~query, ~cookieHeader?, ~decode=userOrdersResponse_decode)
}

// The authenticated user's current on-chain nonce (0 if uninitialized).
let getNonce = (client: Client.t, ~user: SolanaKit.address): promise<result<float, SdkError.t>> =>
  Rpc.getNonce(client, ~user)
