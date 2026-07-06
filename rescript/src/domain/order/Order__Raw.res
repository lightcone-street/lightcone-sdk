// Order wire types — the exact JSON the backend sends/accepts for order
// submit/cancel (REST), the user order/fill read paths, and the WS `user`
// channel's `snapshot` / `order` payloads (like `Orderbook__Raw.Book.t` does for
// `book_update`): `ws/Messages.res` dispatches into them and the stateful
// `Order__State` containers consume them via the wire→domain conversions here.
//
// Wire conventions: Decimal fields → strings; `created_at` / `filled_at` →
// epoch milliseconds (float); `timestamp` → ISO-8601 string. Internally-tagged
// payloads (`order_type`) and field aliases are hand-decoded (spice can't tag or
// alias); plain records lean on spice. Both REST and WS payloads are null-stripped
// upstream (`SdkError.parseApiResponse` / `Messages.decodeMessage`), so optional
// `field?` types tolerate absent and null alike.

// ── Decode helpers (hand-decoded tagged/aliased payloads) ─────────────────────
let field = (dict, key) => Dict.get(dict, key)

let optString = (dict, key) =>
  switch Dict.get(dict, key) {
  | Some(JSON.String(value)) => Some(value)
  | _ => None
  }

let optFloat = (dict, key) =>
  switch Dict.get(dict, key) {
  | Some(JSON.Number(value)) => Some(value)
  | _ => None
  }

// A Decimal wire string that defaults to zero when absent.
let decimalOr0 = (dict, key) => optString(dict, key)->Option.getOr("0")

// `status` field defaulting to Open when absent.
let orderStatusFieldDecode = (dict): result<Shared.OrderStatus.t, Spice.decodeError> =>
  switch field(dict, "status") {
  | Some(json) => Shared.OrderStatus.t_decode(json)
  | None => Ok(Shared.OrderStatus.Open)
  }

// The backend sends TIF numerically in trigger payloads: 0=GTC 1=IOC 2=FOK 3=ALO;
// absent defaults to GTC.
let tifNumericDecode = (dict, key, json): result<Shared.TimeInForce.t, Spice.decodeError> =>
  switch field(dict, key) {
  | None => Ok(Shared.TimeInForce.Gtc)
  | Some(JSON.Number(0.0)) => Ok(Shared.TimeInForce.Gtc)
  | Some(JSON.Number(1.0)) => Ok(Shared.TimeInForce.Ioc)
  | Some(JSON.Number(2.0)) => Ok(Shared.TimeInForce.Fok)
  | Some(JSON.Number(3.0)) => Ok(Shared.TimeInForce.Alo)
  | Some(_) => Spice.error("unknown numeric tif value", json)
  }

// Optional numeric TIF: absent → None; present must be a valid numeric TIF.
let tifNumericOptDecode = (dict, key, json): result<option<Shared.TimeInForce.t>, Spice.decodeError> =>
  switch field(dict, key) {
  | None => Ok(None)
  | Some(_) => tifNumericDecode(dict, key, json)->Result.map(value => Some(value))
  }

let tifToNumber = (tif: Shared.TimeInForce.t): float =>
  switch tif {
  | Gtc => 0.0
  | Ioc => 1.0
  | Fok => 2.0
  | Alo => 3.0
  }

// `amount_in`/`amount_out` accept the legacy `maker_amount`/`taker_amount` aliases.
let amountWithAlias = (dict, key, alias) =>
  switch optString(dict, key) {
  | Some(value) => Some(value)
  | None => optString(dict, alias)
  }

let bigToJson = (value: bigint): JSON.t => JSON.Number(BigInt.toFloat(value))

// Positive-balance test parsing the Decimal wire strings (malformed → zero).
let decimalIsPositive = (value: string): bool =>
  switch Decimal.fromString(value) {
  | decimal => Decimal.gt(decimal, Decimal.fromInt(0))
  | exception JsExn(_) => false
  }

// Tolerant Decimal-string parse: malformed → zero.
let decimalOrZero = (value: string): Decimal.t =>
  switch Decimal.fromString(value) {
  | decimal => decimal
  | exception JsExn(_) => Decimal.fromInt(0)
  }

// The original order size (filled + remaining), as a Decimal string.
let sizeOf = (~filled: string, ~remaining: string): string =>
  Decimal.plus(decimalOrZero(filled), decimalOrZero(remaining))->Decimal.toString

// ISO-8601 → epoch ms (NaN for malformed input, standard JS Date semantics).
let epochMs = (timestamp: string): float => Date.fromString(timestamp)->Date.getTime

// ── Submit / cancel responses ─────────────────────────────────────────────────
// A single fill reported on a submit response.
module FillInfo = {
  @spice
  type t = {
    counterparty: Shared.pubkeyStr,
    @spice.key("counterparty_order_hash") counterpartyOrderHash: string,
    @spice.key("fill_amount") fillAmount: string,
    price: string,
    @spice.key("is_maker") isMaker: bool,
  }
}

module SubmitStatus = {
  @spice
  type t =
    | @as("accepted") @spice.as("accepted") Accepted
    | @as("partial_fill") @spice.as("partial_fill") PartialFill
    | @as("filled") @spice.as("filled") Filled
}

module SubmitResponse = {
  @spice
  type t = {
    @spice.key("order_hash") orderHash: string,
    status: SubmitStatus.t,
    remaining: string,
    filled: string,
    fills: array<FillInfo.t>,
  }
}

// ── Submit request ────────────────────────────────────────────────────────────
// The signed order, ready to POST. Built by `Envelope.buildLimitOrder` /
// `buildTriggerOrder` — a trigger order is the same signed payload plus the
// `triggerPrice` / `triggerType` fields (the backend discriminates on them).
module SubmitRequest = {
  type t = {
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

  // The POST body (snake_case keys; bigints as JSON numbers).
  let toJson = (request: t): JSON.t => {
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
    request.triggerPrice->Option.forEach(price => fields->Array.push(("trigger_price", JSON.Number(price))))
    request.triggerType->Option.forEach(triggerType =>
      fields->Array.push(("trigger_type", Shared.TriggerType.t_encode(triggerType)))
    )
    JSON.Object(Dict.fromArray(fields))
  }
}

// ── Trigger / cancel responses ────────────────────────────────────────────────
// Trigger orders use the same submit endpoint; the trigger fields on the request
// select the trigger path and the backend answers with the trigger order's ids.
module TriggerResponse = {
  @spice
  type t = {
    @spice.key("trigger_order_id") triggerOrderId: string,
    @spice.key("order_hash") orderHash: string,
  }
}

module CancelSuccess = {
  @spice
  type t = {
    @spice.key("order_hash") orderHash: string,
    remaining: string,
  }
}

module CancelAllSuccess = {
  @spice
  type t = {
    @spice.key("cancelled_order_hashes") cancelledOrderHashes: array<string>,
    count: float,
    @spice.key("user_pubkey") userPubkey: Shared.pubkeyStr,
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    message: string,
  }
}

module CancelTriggerSuccess = {
  @spice
  type t = {
    @spice.key("trigger_order_id") triggerOrderId: string,
  }
}

// ── Cancel bodies (signed by `Order__Client`) ─────────────────────────────────
module CancelBody = {
  type t = {orderHash: string, maker: string, signatureHex: string}
}

module CancelAllBody = {
  type t = {
    userPubkey: string,
    orderbookId: string,
    signatureHex: string,
    // Unix seconds.
    timestamp: float,
    salt: string,
  }
}

module CancelTriggerBody = {
  type t = {triggerOrderId: string, maker: string, signatureHex: string}
}

// ── User orders snapshot ──────────────────────────────────────────────────────
// The backend returns limit + trigger orders in one array, discriminated by
// `order_type`, with the common fields flattened into the same object. Used by
// both REST `GET /api/users/orders` and the WS user snapshot.

// Fields shared by limit and trigger order snapshots.
module SnapshotCommon = {
  type t = {
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

  let t_decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch (
        optString(dict, "order_hash"),
        optString(dict, "market_pubkey"),
        optString(dict, "orderbook_id"),
        optString(dict, "base_mint"),
        optString(dict, "quote_mint"),
        field(dict, "side"),
      ) {
      | (
          Some(orderHash),
          Some(marketPubkey),
          Some(orderbookId),
          Some(baseMint),
          Some(quoteMint),
          Some(sideJson),
        ) =>
        switch (
          Shared.Side.t_decode(sideJson),
          orderStatusFieldDecode(dict),
          amountWithAlias(dict, "amount_in", "maker_amount"),
          amountWithAlias(dict, "amount_out", "taker_amount"),
          optFloat(dict, "created_at"),
          optFloat(dict, "outcome_index"),
        ) {
        | (Ok(side), Ok(status), Some(amountIn), Some(amountOut), Some(createdAt), Some(outcomeIndex)) =>
          Ok({
            orderHash,
            marketPubkey,
            orderbookId,
            side,
            amountIn,
            amountOut,
            remaining: decimalOr0(dict, "remaining"),
            filled: decimalOr0(dict, "filled"),
            price: decimalOr0(dict, "price"),
            createdAt,
            expiration: optFloat(dict, "expiration")->Option.getOr(0.0),
            baseMint,
            quoteMint,
            outcomeIndex,
            status,
          })
        | (Error(error), _, _, _, _, _) | (_, Error(error), _, _, _, _) => Error(error)
        | _ => Spice.error("snapshot order missing amount_in/amount_out/created_at/outcome_index", json)
        }
      | _ => Spice.error("snapshot order missing required fields", json)
      }
    | _ => Spice.error("snapshot order is not an object", json)
    }

  // The shared fields as JSON pairs (used by `SnapshotOrder.t_encode`).
  let commonFields = (common: t): array<(string, JSON.t)> => [
    ("order_hash", JSON.String(common.orderHash)),
    ("market_pubkey", JSON.String(common.marketPubkey)),
    ("orderbook_id", JSON.String(common.orderbookId)),
    ("side", Shared.Side.t_encode(common.side)),
    ("amount_in", JSON.String(common.amountIn)),
    ("amount_out", JSON.String(common.amountOut)),
    ("remaining", JSON.String(common.remaining)),
    ("filled", JSON.String(common.filled)),
    ("price", JSON.String(common.price)),
    ("created_at", JSON.Number(common.createdAt)),
    ("expiration", JSON.Number(common.expiration)),
    ("base_mint", JSON.String(common.baseMint)),
    ("quote_mint", JSON.String(common.quoteMint)),
    ("outcome_index", JSON.Number(common.outcomeIndex)),
    ("status", Shared.OrderStatus.t_encode(common.status)),
  ]
}

// The limit arm of a snapshot order.
module SnapshotLimit = {
  type t = {
    common: SnapshotCommon.t,
    txSignature?: string,
  }

  // Snapshot limit arm → domain order.
  let toLimit = (snapshot: t): Order__Model.Limit.t => {
    marketPubkey: snapshot.common.marketPubkey,
    orderbookId: snapshot.common.orderbookId,
    txSignature: ?snapshot.txSignature,
    baseMint: snapshot.common.baseMint,
    quoteMint: snapshot.common.quoteMint,
    orderHash: snapshot.common.orderHash,
    side: snapshot.common.side,
    size: sizeOf(~filled=snapshot.common.filled, ~remaining=snapshot.common.remaining),
    price: snapshot.common.price,
    filledSize: snapshot.common.filled,
    remainingSize: snapshot.common.remaining,
    createdAt: snapshot.common.createdAt,
    status: snapshot.common.status,
    outcomeIndex: snapshot.common.outcomeIndex,
  }
}

// The trigger arm of a snapshot order.
module SnapshotTrigger = {
  type t = {
    common: SnapshotCommon.t,
    triggerOrderId: string,
    triggerPrice: string,
    triggerType: Shared.TriggerType.t,
    timeInForce?: Shared.TimeInForce.t,
  }

  // Snapshot trigger arm → domain order; an absent TIF defaults to GTC.
  let toTrigger = (snapshot: t): Order__Model.Trigger.t => {
    triggerOrderId: snapshot.triggerOrderId,
    orderHash: snapshot.common.orderHash,
    marketPubkey: snapshot.common.marketPubkey,
    orderbookId: snapshot.common.orderbookId,
    triggerPrice: snapshot.triggerPrice,
    triggerType: snapshot.triggerType,
    side: snapshot.common.side,
    amountIn: snapshot.common.amountIn,
    amountOut: snapshot.common.amountOut,
    timeInForce: snapshot.timeInForce->Option.getOr(Shared.TimeInForce.Gtc),
    createdAt: snapshot.common.createdAt,
  }
}

// Order snapshot — tagged on `order_type` ("limit" | "trigger"). The manual
// `t_decode`/`t_encode` follow the spice naming convention so spice records can
// embed `SnapshotOrder.t` directly.
module SnapshotOrder = {
  type t =
    | Limit(SnapshotLimit.t)
    | Trigger(SnapshotTrigger.t)

  // The fields shared by both variants.
  let common = (order: t): SnapshotCommon.t =>
    switch order {
    | Limit(order) => order.common
    | Trigger(order) => order.common
    }

  let t_decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch field(dict, "order_type") {
      | Some(JSON.String("limit")) =>
        SnapshotCommon.t_decode(json)->Result.map(common => Limit({
          common,
          txSignature: ?optString(dict, "tx_signature"),
        }))
      | Some(JSON.String("trigger")) =>
        switch (
          SnapshotCommon.t_decode(json),
          optString(dict, "trigger_order_id"),
          optString(dict, "trigger_price"),
          field(dict, "trigger_type")->Option.map(Shared.TriggerType.t_decode),
          tifNumericOptDecode(dict, "time_in_force", json),
        ) {
        | (Ok(common), Some(triggerOrderId), Some(triggerPrice), Some(Ok(triggerType)), Ok(timeInForce)) =>
          Ok(Trigger({common, triggerOrderId, triggerPrice, triggerType, timeInForce: ?timeInForce}))
        | (Error(error), _, _, _, _) => Error(error)
        | (_, _, _, Some(Error(error)), _) => Error(error)
        | (_, _, _, _, Error(error)) => Error(error)
        | _ => Spice.error("trigger snapshot order missing trigger fields", json)
        }
      | _ => Spice.error("unknown snapshot order_type", json)
      }
    | _ => Spice.error("snapshot order is not an object", json)
    }

  let t_encode = (order: t): JSON.t =>
    switch order {
    | Limit(order) =>
      let fields = [("order_type", JSON.String("limit"))]
      fields->Array.pushMany(SnapshotCommon.commonFields(order.common))
      order.txSignature->Option.forEach(value => fields->Array.push(("tx_signature", JSON.String(value))))
      JSON.Object(Dict.fromArray(fields))
    | Trigger(order) =>
      let fields = [("order_type", JSON.String("trigger"))]
      fields->Array.pushMany(SnapshotCommon.commonFields(order.common))
      fields->Array.push(("trigger_order_id", JSON.String(order.triggerOrderId)))
      fields->Array.push(("trigger_price", JSON.String(order.triggerPrice)))
      fields->Array.push(("trigger_type", Shared.TriggerType.t_encode(order.triggerType)))
      order.timeInForce->Option.forEach(tif =>
        fields->Array.push(("time_in_force", JSON.Number(tifToNumber(tif))))
      )
      JSON.Object(Dict.fromArray(fields))
    }

  // spice ≥0.4 references `_encodeJson` in fixed JSON contexts (record fields,
  // array elements); for a non-option type it coincides with `t_encode`.
  let t_encodeJson = t_encode
}

// ── User balances ─────────────────────────────────────────────────────────────
module UserOutcomeBalance = {
  @spice
  type t = {
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("conditional_token") conditionalToken: Shared.pubkeyStr,
    balance: string,
    @spice.key("balance_idle") balanceIdle: string,
    @spice.key("balance_on_book") balanceOnBook: string,
  }

  // Neither idle nor on-book balance is positive.
  let isZero = (balance: t): bool =>
    !(decimalIsPositive(balance.balanceIdle) || decimalIsPositive(balance.balanceOnBook))
}

module UserDepositAssetBalance = {
  @spice
  type t = {
    @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
    outcomes: array<UserOutcomeBalance.t>,
  }
}

module UserMarketBalance = {
  @spice
  type t = {
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("deposit_assets") depositAssets: array<UserDepositAssetBalance.t>,
  }
}

// Global deposit balance for a single mint (WS snapshots).
module GlobalDepositBalance = {
  @spice
  type t = {
    mint: Shared.pubkeyStr,
    balance: string,
  }
}

// ── User orders response (REST `GET /api/users/orders`) ──────────────────────
module UserOrdersResponse = {
  @spice
  type t = {
    @spice.key("user_pubkey") userPubkey: Shared.pubkeyStr,
    orders: array<SnapshotOrder.t>,
    @spice.key("market_balances") marketBalances: array<UserMarketBalance.t>,
    @spice.key("next_cursor") nextCursor?: string,
    @spice.default(false) @spice.key("has_more") hasMore: bool,
  }
}

// ── User order fills (REST) ───────────────────────────────────────────────────
// Whether the user was the maker or taker on an order.
module Role = {
  @spice
  type t = | @as("maker") @spice.as("maker") Maker | @as("taker") @spice.as("taker") Taker
}

// Status of a filled order, derived from DB state after the fact (distinct from
// `Shared.OrderStatus`, the engine's real-time state).
module FillStatus = {
  @spice
  type t =
    | @as("filled") @spice.as("filled") Filled
    | @as("cancelled") @spice.as("cancelled") Cancelled
    | @as("partially_filled") @spice.as("partially_filled") PartiallyFilled
}

// A single fill event within an order.
module FillEvent = {
  @spice
  type t = {
    @spice.key("fill_amount") fillAmount: string,
    @spice.key("tx_signature") txSignature: string,
    @spice.key("filled_at") filledAt: float,
  }
}

// An order the user participated in, with nested fill events.
module UserFill = {
  @spice
  type t = {
    @spice.key("order_hash") orderHash: string,
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    side: Shared.Side.t,
    role: Role.t,
    price: string,
    size: string,
    @spice.key("filled_size") filledSize: string,
    @spice.key("remaining_size") remainingSize: string,
    @spice.key("base_mint") baseMint: Shared.pubkeyStr,
    @spice.key("quote_mint") quoteMint: Shared.pubkeyStr,
    @spice.key("outcome_index") outcomeIndex: float,
    status: FillStatus.t,
    @spice.key("created_at") createdAt: float,
    fills: array<FillEvent.t>,
  }
}

// Response from `GET /api/users/order-fills`.
module UserFillsResponse = {
  @spice
  type t = {
    orders: array<UserFill.t>,
    @spice.key("next_cursor") nextCursor?: string,
    @spice.key("has_more") hasMore: bool,
  }
}

// ── WS user order events ──────────────────────────────────────────────────────
// Payloads for the WS `user` channel's `order` / `snapshot` events; dispatched by
// `ws/Messages.res` and consumed by the stateful `Order__State` containers.

// Balance for a single conditional token, attached to WS order updates.
module ConditionalBalance = {
  @spice
  type t = {
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("conditional_token") conditionalToken: Shared.pubkeyStr,
    idle: string,
    @spice.key("on_book") onBook: string,
  }
}

module UpdateBalance = {
  @spice
  type t = {outcomes: array<ConditionalBalance.t>}
}

// Individual order within a WS update. `created_at` is epoch milliseconds.
module WsOrder = {
  @spice
  type t = {
    @spice.key("order_hash") orderHash: string,
    price: string,
    @spice.key("is_maker") isMaker: bool,
    remaining: string,
    filled: string,
    @spice.key("fill_amount") fillAmount: string,
    side: Shared.Side.t,
    @spice.key("created_at") createdAt: float,
    @spice.key("base_mint") baseMint: Shared.pubkeyStr,
    @spice.key("quote_mint") quoteMint: Shared.pubkeyStr,
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.default(Shared.OrderStatus.Open) status: Shared.OrderStatus.t,
    // Absent on cancellation events.
    balance?: UpdateBalance.t,
  }
}

// WS limit-order update event. `timestamp` is ISO-8601.
module Update = {
  @spice
  type t = {
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    timestamp: string,
    @spice.key("tx_signature") txSignature?: string,
    @spice.default(Shared.OrderUpdateType.Update) @spice.key("type") updateType: Shared.OrderUpdateType.t,
    order: WsOrder.t,
  }

  // WS limit-order update → domain order.
  let toLimit = (update: t): Order__Model.Limit.t => {
    marketPubkey: update.marketPubkey,
    orderbookId: update.orderbookId,
    txSignature: ?update.txSignature,
    baseMint: update.order.baseMint,
    quoteMint: update.order.quoteMint,
    orderHash: update.order.orderHash,
    side: update.order.side,
    size: sizeOf(~filled=update.order.filled, ~remaining=update.order.remaining),
    price: update.order.price,
    filledSize: update.order.filled,
    remainingSize: update.order.remaining,
    createdAt: update.order.createdAt,
    status: update.order.status,
    outcomeIndex: update.order.outcomeIndex,
  }
}

// Trigger-order WS update event on the user channel. Result fields are zero /
// absent until the order triggers; `tif` arrives numerically.
module TriggerUpdate = {
  type t = {
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

  // `type` field defaulting to Triggered when absent.
  let updateTypeDecode = (dict): result<Shared.TriggerUpdateType.t, Spice.decodeError> =>
    switch field(dict, "type") {
    | Some(json) => Shared.TriggerUpdateType.t_decode(json)
    | None => Ok(Shared.TriggerUpdateType.Triggered)
    }

  // `result_status`: absent or empty string → None.
  let resultStatusDecode = (dict): result<option<Shared.TriggerResultStatus.t>, Spice.decodeError> =>
    switch field(dict, "result_status") {
    | None | Some(JSON.String("")) => Ok(None)
    | Some(json) => Shared.TriggerResultStatus.t_decode(json)->Result.map(value => Some(value))
    }

  let t_decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch (
        optString(dict, "trigger_order_id"),
        optString(dict, "market_pubkey"),
        optString(dict, "orderbook_id"),
        optString(dict, "trigger_price"),
        field(dict, "trigger_above"),
        field(dict, "status"),
        optString(dict, "order_hash"),
        field(dict, "side"),
        optString(dict, "timestamp"),
      ) {
      | (
          Some(triggerOrderId),
          Some(marketPubkey),
          Some(orderbookId),
          Some(triggerPrice),
          Some(JSON.Boolean(triggerAbove)),
          Some(statusJson),
          Some(orderHash),
          Some(sideJson),
          Some(timestamp),
        ) =>
        switch (
          Shared.TriggerStatus.t_decode(statusJson),
          Shared.Side.t_decode(sideJson),
          updateTypeDecode(dict),
          resultStatusDecode(dict),
          tifNumericDecode(dict, "tif", json),
        ) {
        | (Ok(status), Ok(side), Ok(updateType), Ok(resultStatus), Ok(tif)) =>
          Ok({
            triggerOrderId,
            userPubkey: optString(dict, "user_pubkey")->Option.getOr(""),
            marketPubkey,
            orderbookId,
            triggerPrice,
            triggerAbove,
            status,
            updateType,
            orderHash,
            side,
            resultStatus: ?resultStatus,
            resultFilled: decimalOr0(dict, "result_filled"),
            resultRemaining: decimalOr0(dict, "result_remaining"),
            timestamp,
            makerAmount: decimalOr0(dict, "maker_amount"),
            takerAmount: decimalOr0(dict, "taker_amount"),
            tif,
          })
        | (Error(error), _, _, _, _)
        | (_, Error(error), _, _, _)
        | (_, _, Error(error), _, _)
        | (_, _, _, Error(error), _)
        | (_, _, _, _, Error(error)) =>
          Error(error)
        }
      | _ => Spice.error("trigger order update missing required fields", json)
      }
    | _ => Spice.error("trigger order update is not an object", json)
    }

  // WS trigger update → domain order: the trigger type is implied by the
  // trigger direction (`triggerAbove`).
  let toTrigger = (update: t): Order__Model.Trigger.t => {
    triggerOrderId: update.triggerOrderId,
    orderHash: update.orderHash,
    marketPubkey: update.marketPubkey,
    orderbookId: update.orderbookId,
    triggerPrice: update.triggerPrice,
    triggerType: update.triggerAbove ? Shared.TriggerType.TakeProfit : Shared.TriggerType.StopLoss,
    side: update.side,
    amountIn: update.makerAmount,
    amountOut: update.takerAmount,
    timeInForce: update.tif,
    createdAt: epochMs(update.timestamp),
  }
}

// WS order event — both limit and trigger updates arrive as `event_type: "order"`,
// discriminated by `order_type` at the same level (internally tagged).
module Event = {
  type t =
    | Limit(Update.t)
    | Trigger(TriggerUpdate.t)

  let t_decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch field(dict, "order_type") {
      | Some(JSON.String("limit")) => Update.t_decode(json)->Result.map(value => Limit(value))
      | Some(JSON.String("trigger")) => TriggerUpdate.t_decode(json)->Result.map(value => Trigger(value))
      | _ => Spice.error("unknown order event order_type", json)
      }
    | _ => Spice.error("order event is not an object", json)
    }
}

// WS user snapshot — the full authenticated-user state pushed on subscribe.
module UserSnapshot = {
  type t = {
    orders: array<SnapshotOrder.t>,
    marketBalances: array<UserMarketBalance.t>,
    globalDeposits: array<GlobalDepositBalance.t>,
    notifications: array<Notification__Model.t>,
    nonce: float,
  }

  let t_decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch (field(dict, "orders"), field(dict, "market_balances")) {
      | (Some(ordersJson), Some(balancesJson)) =>
        switch (
          Spice.arrayFromJson(SnapshotOrder.t_decode, ordersJson),
          Spice.arrayFromJson(UserMarketBalance.t_decode, balancesJson),
          field(dict, "global_deposits")->Option.mapOr(Ok([]), json =>
            Spice.arrayFromJson(GlobalDepositBalance.t_decode, json)
          ),
          field(dict, "notifications")->Option.mapOr(Ok([]), json =>
            Spice.arrayFromJson(Notification.Raw.decode, json)
          ),
        ) {
        | (Ok(orders), Ok(marketBalances), Ok(globalDeposits), Ok(notifications)) =>
          Ok({
            orders,
            marketBalances,
            globalDeposits,
            notifications,
            nonce: optFloat(dict, "nonce")->Option.getOr(0.0),
          })
        | (Error(error), _, _, _)
        | (_, Error(error), _, _)
        | (_, _, Error(error), _)
        | (_, _, _, Error(error)) =>
          Error(error)
        }
      | _ => Spice.error("user snapshot missing orders/market_balances", json)
      }
    | _ => Spice.error("user snapshot is not an object", json)
    }
}
