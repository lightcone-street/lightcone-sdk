// Order REST client — submit, cancel, cancel-all, and the user order/fill read
// paths. The order is signed off-chain (OrderPayload) and POSTed; cancels sign a
// message and POST the hex signature. Functions take a `Client.t` and return
// `promise<result<_, SdkError.t>>` over the `Order__Raw` wire types.
//
// Note: `getNonce` reads the on-chain UserNonce PDA via the RPC/Accounts layer
// (delegates to `Rpc.getNonce`). The envelope still defaults nonce to 0 when the
// caller doesn't supply one — fetch a fresh value via `getNonce` for live submission.

let utf8 = text => SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), text)
let nowSeconds = (): float => Math.floor(Date.now() /. 1000.0)

// ── Submit ────────────────────────────────────────────────────────────────────
let submit = async (
  client: Client.t,
  request: Order__Raw.SubmitRequest.t,
): result<Order__Raw.SubmitResponse.t, SdkError.t> =>
  await Http.post(
    client.http,
    ~path="/api/orders/submit",
    ~body=Order__Raw.SubmitRequest.toJson(request),
    ~retry=NoRetry,
    ~decode=Order__Raw.SubmitResponse.t_decode,
  )

// Submit a signed trigger order (built by `Envelope.buildTriggerOrder` — the
// request must carry `triggerPrice` + `triggerType`; the backend discriminates
// on them and answers with the trigger order's ids). Never retried.
let submitTrigger = async (
  client: Client.t,
  request: Order__Raw.SubmitRequest.t,
): result<Order__Raw.TriggerResponse.t, SdkError.t> =>
  await Http.post(
    client.http,
    ~path="/api/orders/submit",
    ~body=Order__Raw.SubmitRequest.toJson(request),
    ~retry=NoRetry,
    ~decode=Order__Raw.TriggerResponse.t_decode,
  )

// ── Cancel ────────────────────────────────────────────────────────────────────
// The cancel signing messages (UTF-8 bytes, ed25519-signed, hex-encoded).
let cancelAllMessage = (~userPubkey, ~orderbookId, ~timestamp: float, ~salt): string =>
  `cancel_all:${userPubkey}:${orderbookId}:${Float.toString(timestamp)}:${salt}`

// Sign a cancel for one order: signs the order-hash hex string's UTF-8 bytes.
let cancelBodySigned = async (
  ~orderHash: string,
  ~maker: string,
  ~keypair: SolanaKit.cryptoKeyPair,
): Order__Raw.CancelBody.t => {
  let signature = await SolanaKitKeys.signBytes(keypair.privateKey, utf8(orderHash))
  {orderHash, maker, signatureHex: OrderPayload.signatureHex(signature)}
}

let cancelAllBodySigned = async (
  ~userPubkey: string,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
): Order__Raw.CancelAllBody.t => {
  let timestamp = nowSeconds()
  let salt = WebCrypto.randomUUID()
  let message = utf8(cancelAllMessage(~userPubkey, ~orderbookId, ~timestamp, ~salt))
  let signature = await SolanaKitKeys.signBytes(keypair.privateKey, message)
  {userPubkey, orderbookId, signatureHex: OrderPayload.signatureHex(signature), timestamp, salt}
}

let cancel = async (
  client: Client.t,
  body: Order__Raw.CancelBody.t,
): result<Order__Raw.CancelSuccess.t, SdkError.t> => {
  let json = JSON.Object(
    Dict.fromArray([
      ("order_hash", JSON.String(body.orderHash)),
      ("maker", JSON.String(body.maker)),
      ("signature", JSON.String(body.signatureHex)),
    ]),
  )
  await Http.post(
    client.http,
    ~path="/api/orders/cancel",
    ~body=json,
    ~retry=NoRetry,
    ~decode=Order__Raw.CancelSuccess.t_decode,
  )
}

let cancelAll = async (
  client: Client.t,
  body: Order__Raw.CancelAllBody.t,
): result<Order__Raw.CancelAllSuccess.t, SdkError.t> => {
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
    ~decode=Order__Raw.CancelAllSuccess.t_decode,
  )
}

// Sign a cancel for one trigger order: signs the trigger order id's UTF-8 bytes.
let cancelTriggerBodySigned = async (
  ~triggerOrderId: string,
  ~maker: string,
  ~keypair: SolanaKit.cryptoKeyPair,
): Order__Raw.CancelTriggerBody.t => {
  let signature = await SolanaKitKeys.signBytes(keypair.privateKey, utf8(triggerOrderId))
  {triggerOrderId, maker, signatureHex: OrderPayload.signatureHex(signature)}
}

// Cancel a resting trigger order (same endpoint as limit cancels; the backend
// discriminates on `trigger_order_id`). Never retried.
let cancelTrigger = async (
  client: Client.t,
  body: Order__Raw.CancelTriggerBody.t,
): result<Order__Raw.CancelTriggerSuccess.t, SdkError.t> => {
  let json = JSON.Object(
    Dict.fromArray([
      ("trigger_order_id", JSON.String(body.triggerOrderId)),
      ("maker", JSON.String(body.maker)),
      ("signature", JSON.String(body.signatureHex)),
    ]),
  )
  await Http.post(
    client.http,
    ~path="/api/orders/cancel",
    ~body=json,
    ~retry=NoRetry,
    ~decode=Order__Raw.CancelTriggerSuccess.t_decode,
  )
}

// ── Strategy-signed cancels ───────────────────────────────────────────────────
// Each signs with the client's configured strategy (native keypair or external
// wallet adapter), the signer's address acting as maker.

let signerAddressOrError = (client: Client.t): result<string, SdkError.t> =>
  switch Client.signerAddress(client) {
  | Some(address) => Ok(SolanaKit.addressToString(address))
  | None =>
    Error(
      SdkError.Signing(
        "no signing strategy configured; call Client.useNativeSigner or Client.useExternalSigner first",
      ),
    )
  }

// Cancel one order, signing with the client's strategy.
let cancelSigned = async (
  client: Client.t,
  ~orderHash: string,
): result<Order__Raw.CancelSuccess.t, SdkError.t> =>
  switch signerAddressOrError(client) {
  | Error(error) => Error(error)
  | Ok(maker) =>
    switch await Client.signMessageBytes(client, utf8(orderHash)) {
    | Error(error) => Error(error)
    | Ok(signature) =>
      await cancel(client, {orderHash, maker, signatureHex: OrderPayload.signatureHex(signature)})
    }
  }

// Cancel every open order on one orderbook, signing with the client's strategy.
let cancelAllSigned = async (
  client: Client.t,
  ~orderbookId: string,
): result<Order__Raw.CancelAllSuccess.t, SdkError.t> =>
  switch signerAddressOrError(client) {
  | Error(error) => Error(error)
  | Ok(userPubkey) =>
    let timestamp = nowSeconds()
    let salt = WebCrypto.randomUUID()
    let message = utf8(cancelAllMessage(~userPubkey, ~orderbookId, ~timestamp, ~salt))
    switch await Client.signMessageBytes(client, message) {
    | Error(error) => Error(error)
    | Ok(signature) =>
      await cancelAll(
        client,
        {userPubkey, orderbookId, signatureHex: OrderPayload.signatureHex(signature), timestamp, salt},
      )
    }
  }

// Cancel a resting trigger order, signing with the client's strategy.
let cancelTriggerSigned = async (
  client: Client.t,
  ~triggerOrderId: string,
): result<Order__Raw.CancelTriggerSuccess.t, SdkError.t> =>
  switch signerAddressOrError(client) {
  | Error(error) => Error(error)
  | Ok(maker) =>
    switch await Client.signMessageBytes(client, utf8(triggerOrderId)) {
    | Error(error) => Error(error)
    | Ok(signature) =>
      await cancelTrigger(
        client,
        {triggerOrderId, maker, signatureHex: OrderPayload.signatureHex(signature)},
      )
    }
  }

// ── User orders / fills (read paths) ──────────────────────────────────────────
// The authenticated user's open orders (wallet resolved server-side from the cookie).
let getUserOrders = async (
  client: Client.t,
  ~limit: option<int>=?,
  ~cursor: option<string>=?,
  ~cookieHeader: option<string>=?,
): result<Order__Raw.UserOrdersResponse.t, SdkError.t> => {
  let query = []
  limit->Option.forEach(value => query->Array.push(("limit", Int.toString(value))))
  cursor->Option.forEach(value => query->Array.push(("cursor", value)))
  await Http.get(
    client.http,
    ~path="/api/users/orders",
    ~query,
    ~cookieHeader?,
    ~decode=Order__Raw.UserOrdersResponse.t_decode,
  )
}

let orderFillsQuery = (~marketPubkey, ~limit, ~cursor) => {
  let query = []
  marketPubkey->Option.forEach(value => query->Array.push(("market_pubkey", value)))
  limit->Option.forEach(value => query->Array.push(("limit", Int.toString(value))))
  cursor->Option.forEach(value => query->Array.push(("cursor", value)))
  query
}

// The authenticated user's filled orders with nested fill events (maker or taker;
// most recent fill first). Optionally filter by market.
let getUserOrderFills = async (
  client: Client.t,
  ~marketPubkey: option<string>=?,
  ~limit: option<int>=?,
  ~cursor: option<string>=?,
  ~cookieHeader: option<string>=?,
): result<Order__Raw.UserFillsResponse.t, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/users/order-fills",
    ~query=orderFillsQuery(~marketPubkey, ~limit, ~cursor),
    ~cookieHeader?,
    ~decode=Order__Raw.UserFillsResponse.t_decode,
  )

// Public variant: takes the wallet via the URL path, requires no auth.
let getUserOrderFillsByWallet = async (
  client: Client.t,
  ~walletAddress: string,
  ~marketPubkey: option<string>=?,
  ~limit: option<int>=?,
  ~cursor: option<string>=?,
): result<Order__Raw.UserFillsResponse.t, SdkError.t> =>
  await Http.get(
    client.http,
    ~path=`/api/users/${walletAddress}/order-fills`,
    ~query=orderFillsQuery(~marketPubkey, ~limit, ~cursor),
    ~decode=Order__Raw.UserFillsResponse.t_decode,
  )

// ── Onchain reads ─────────────────────────────────────────────────────────────
// The authenticated user's current on-chain nonce (0 if uninitialized).
let getNonce = (client: Client.t, ~user: SolanaKit.address): promise<result<float, SdkError.t>> =>
  Rpc.getNonce(client, ~user)

// An order's on-chain OrderStatus account (`None` once fully filled + closed,
// or never created). `~orderHash` is the 32-byte keccak digest.
let getStatus = (
  client: Client.t,
  ~orderHash: Uint8Array.t,
): promise<result<option<Accounts.OrderStatus.t>, SdkError.t>> =>
  Rpc.getOrderStatus(client, ~orderHash)
