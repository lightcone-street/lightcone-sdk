// Order wire types — submit/cancel (REST), the user order/fill read paths, and
// the WS `user` channel's `snapshot` / `order` payloads. `@spice` is kept where
// the codecs cross a file boundary (the REST client, `ws/Messages.res`, tests);
// hand-decoded tagged/aliased payloads expose explicit `t_decode` vals instead.
// `@genType` exports the TS shapes. The decode helpers, `SnapshotCommon`'s
// codec/encode helpers, and the JSON body builders they feed are internal.

// ── Submit / cancel responses ─────────────────────────────────────────────────
// A single fill reported on a submit response.
module FillInfo: {
  @genType @spice
  type t = {
    counterparty: Shared.pubkeyStr,
    @spice.key("counterparty_order_hash") counterpartyOrderHash: string,
    @spice.key("fill_amount") fillAmount: string,
    price: string,
    @spice.key("is_maker") isMaker: bool,
  }
}

module SubmitStatus: {
  @genType @spice
  type t =
    | @as("accepted") @spice.as("accepted") Accepted
    | @as("partial_fill") @spice.as("partial_fill") PartialFill
    | @as("filled") @spice.as("filled") Filled
}

module SubmitResponse: {
  @genType @spice
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
module SubmitRequest: {
  @genType
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
  let toJson: t => JSON.t
}

// ── Trigger / cancel responses ────────────────────────────────────────────────
// Trigger orders use the same submit endpoint; the trigger fields on the request
// select the trigger path and the backend answers with the trigger order's ids.
module TriggerResponse: {
  @genType @spice
  type t = {
    @spice.key("trigger_order_id") triggerOrderId: string,
    @spice.key("order_hash") orderHash: string,
  }
}

module CancelSuccess: {
  @genType @spice
  type t = {
    @spice.key("order_hash") orderHash: string,
    remaining: string,
  }
}

module CancelAllSuccess: {
  @genType @spice
  type t = {
    @spice.key("cancelled_order_hashes") cancelledOrderHashes: array<string>,
    count: float,
    @spice.key("user_pubkey") userPubkey: Shared.pubkeyStr,
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    message: string,
  }
}

module CancelTriggerSuccess: {
  @genType @spice
  type t = {
    @spice.key("trigger_order_id") triggerOrderId: string,
  }
}

// ── Cancel bodies (signed by `Order__Client`) ─────────────────────────────────
module CancelBody: {
  @genType
  type t = {orderHash: string, maker: string, signatureHex: string}
}

module CancelAllBody: {
  @genType
  type t = {
    userPubkey: string,
    orderbookId: string,
    signatureHex: string,
    // Unix seconds.
    timestamp: float,
    salt: string,
  }
}

module CancelTriggerBody: {
  @genType
  type t = {triggerOrderId: string, maker: string, signatureHex: string}
}

// ── User orders snapshot ──────────────────────────────────────────────────────
// The backend returns limit + trigger orders in one array, discriminated by
// `order_type`, with the common fields flattened into the same object. Used by
// both REST `GET /api/users/orders` and the WS user snapshot.

// Fields shared by limit and trigger order snapshots.
module SnapshotCommon: {
  @genType
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
}

// The limit arm of a snapshot order.
module SnapshotLimit: {
  @genType
  type t = {
    common: SnapshotCommon.t,
    txSignature?: string,
  }

  // Snapshot limit arm → domain order.
  @genType
  let toLimit: t => Order__Model.Limit.t
}

// The trigger arm of a snapshot order.
module SnapshotTrigger: {
  @genType
  type t = {
    common: SnapshotCommon.t,
    triggerOrderId: string,
    triggerPrice: string,
    triggerType: Shared.TriggerType.t,
    timeInForce?: Shared.TimeInForce.t,
  }

  // Snapshot trigger arm → domain order; an absent TIF defaults to GTC.
  @genType
  let toTrigger: t => Order__Model.Trigger.t
}

// Order snapshot — tagged on `order_type` ("limit" | "trigger").
module SnapshotOrder: {
  @genType
  type t =
    | Limit(SnapshotLimit.t)
    | Trigger(SnapshotTrigger.t)

  // The fields shared by both variants.
  let common: t => SnapshotCommon.t

  // Spice-convention codecs (also embeddable in spice records; `t_encodeJson`
  // is spice ≥0.4's fixed-JSON-context alias of `t_encode`).
  let t_decode: JSON.t => result<t, Spice.decodeError>
  let t_encode: t => JSON.t
  let t_encodeJson: t => JSON.t
}

// ── User balances ─────────────────────────────────────────────────────────────
module UserOutcomeBalance: {
  @genType @spice
  type t = {
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("conditional_token") conditionalToken: Shared.pubkeyStr,
    balance: string,
    @spice.key("balance_idle") balanceIdle: string,
    @spice.key("balance_on_book") balanceOnBook: string,
  }

  // Neither idle nor on-book balance is positive.
  let isZero: t => bool
}

module UserDepositAssetBalance: {
  @genType @spice
  type t = {
    @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
    outcomes: array<UserOutcomeBalance.t>,
  }
}

// The WS balance update carries the same tree (`ws/Messages.res` decodes it).
module UserMarketBalance: {
  @genType @spice
  type t = {
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("deposit_assets") depositAssets: array<UserDepositAssetBalance.t>,
  }
}

// Global deposit balance for a single mint (WS snapshots).
module GlobalDepositBalance: {
  @genType @spice
  type t = {
    mint: Shared.pubkeyStr,
    balance: string,
  }
}

// ── User orders response (REST `GET /api/users/orders`) ──────────────────────
module UserOrdersResponse: {
  @genType @spice
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
module FillEvent: {
  @genType @spice
  type t = {
    @spice.key("fill_amount") fillAmount: string,
    @spice.key("tx_signature") txSignature: string,
    // Unix milliseconds.
    @spice.key("filled_at") filledAt: float,
  }
}

// An order the user participated in, with nested fill events.
module UserFill: {
  @genType @spice
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
    // Unix milliseconds.
    @spice.key("created_at") createdAt: float,
    fills: array<FillEvent.t>,
  }
}

// Response from `GET /api/users/order-fills`.
module UserFillsResponse: {
  @genType @spice
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
module ConditionalBalance: {
  @genType @spice
  type t = {
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("conditional_token") conditionalToken: Shared.pubkeyStr,
    idle: string,
    @spice.key("on_book") onBook: string,
  }
}

module UpdateBalance: {
  @genType @spice
  type t = {outcomes: array<ConditionalBalance.t>}
}

// Individual order within a WS update. `createdAt` is epoch milliseconds.
module WsOrder: {
  @genType @spice
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
module Update: {
  @genType @spice
  type t = {
    @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    timestamp: string,
    @spice.key("tx_signature") txSignature?: string,
    @spice.default(Shared.OrderUpdateType.Update) @spice.key("type") updateType: Shared.OrderUpdateType.t,
    order: WsOrder.t,
  }

  // WS limit-order update → domain order.
  @genType
  let toLimit: t => Order__Model.Limit.t
}

// Trigger-order WS update event on the user channel. Result fields are zero /
// absent until the order triggers; `tif` arrives numerically.
module TriggerUpdate: {
  @genType
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

  // Exposed for tests (the WS trigger update is hand-decoded).
  let t_decode: JSON.t => result<t, Spice.decodeError>

  // WS trigger update → domain order: the trigger type is implied by the
  // trigger direction (`triggerAbove`).
  @genType
  let toTrigger: t => Order__Model.Trigger.t
}

// WS order event — both limit and trigger updates arrive as `event_type: "order"`,
// discriminated by `order_type` at the same level.
module Event: {
  @genType
  type t =
    | Limit(Update.t)
    | Trigger(TriggerUpdate.t)

  let t_decode: JSON.t => result<t, Spice.decodeError>
}

// WS user snapshot — the full authenticated-user state pushed on subscribe.
module UserSnapshot: {
  @genType
  type t = {
    orders: array<SnapshotOrder.t>,
    marketBalances: array<UserMarketBalance.t>,
    globalDeposits: array<GlobalDepositBalance.t>,
    notifications: array<Notification__Model.t>,
    nonce: float,
  }

  let t_decode: JSON.t => result<t, Spice.decodeError>
}
