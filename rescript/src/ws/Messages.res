// Inbound WebSocket messages — the ReScript counterpart of the Rust SDK's
// `ws::{MessageIn, Kind}` plus the per-channel payload wire types.
//
// Wire envelope: `{"type": "<channel>", "version": 0.1, "data": <payload>}`.
// `decodeMessage` strips JSON `null`s (so spice's `field?:T` tolerates both
// absent and null), reads the `type` discriminator, and decodes `data` into the
// matching `Kind` variant. Internally-tagged payloads (`event_type` / `status` /
// market `event_type`) are hand-decoded, mirroring `Auth.res`/`Notification.res`;
// plain records lean on spice.
//
// Conventions: prices/sizes/Decimals → wire strings (no precision loss; the Rust
// uses `rust_decimal` with `serde-str`). Sequences and millisecond timestamps →
// `float`. `DateTime<Utc>` fields the backend serializes as ISO-8601 strings stay
// strings (e.g. WS trade `timestamp`, the user-event `timestamp`s). The user
// `snapshot` / `order` payload trees live in `Order.res` (like `Orderbook.orderBook`
// does for `book_update`) and are dispatched into from `UserUpdate` below.
//
// Variant payloads use value or named-record arms (never ReScript inline records,
// which fail to compile): single-field channels carry a bare value, multi-field
// channels carry a named record declared just above the tagged variant.

// ── Decode helpers ────────────────────────────────────────────────────────────
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

// ── Pong / Error ──────────────────────────────────────────────────────────────
// Server-side WS error with full diagnostics. `error` is the message; book-scoped
// errors carry `orderbook_id` + the aggregation tags (absent = full precision).
@spice
type wsErrorFrame = {
  error: string,
  code?: string,
  @spice.key("orderbook_id") orderbookId?: string,
  @spice.key("n_sig_figs") nSigFigs?: int,
  mantissa?: int,
  @spice.key("wallet_address") walletAddress?: string,
  @spice.key("deposit_asset") depositAsset?: string,
  hint?: string,
  details?: string,
}

// ── Trades ────────────────────────────────────────────────────────────────────
// WS trade event. `timestamp` is an ISO-8601 string (the backend serializes the
// `DateTime<Utc>` as RFC 3339); `sequence` is a monotonic per-orderbook counter.
@spice
type wsTrade = {
  @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
  @spice.key("trade_id") tradeId: string,
  timestamp: string,
  price: string,
  size: string,
  side: Shared.Side.t,
  @spice.default(0.0) sequence: float,
}

// ── Ticker ────────────────────────────────────────────────────────────────────
// `mid` accepts the `mid_price` alias (hand-decoded — spice can't alias keys).
type wsTicker = {
  orderbookId: Shared.orderBookId,
  bestBid?: string,
  bestAsk?: string,
  mid?: string,
}

let wsTickerDecode = (json: JSON.t): result<wsTicker, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch field(dict, "orderbook_id") {
    | Some(JSON.String(orderbookId)) =>
      let mid = switch optString(dict, "mid") {
      | Some(value) => Some(value)
      | None => optString(dict, "mid_price")
      }
      Ok({
        orderbookId,
        bestBid: ?optString(dict, "best_bid"),
        bestAsk: ?optString(dict, "best_ask"),
        mid: ?mid,
      })
    | _ => Spice.error("ticker payload missing orderbook_id", json)
    }
  | _ => Spice.error("ticker payload is not an object", json)
  }

// ── Market lifecycle events (internally tagged on `event_type`) ───────────────
module MarketEvent = {
  type t =
    | Settled(Shared.pubkeyStr)
    | Created(Shared.pubkeyStr)
    | Opened(Shared.pubkeyStr)
    | Paused(Shared.pubkeyStr)
    | OrderbookCreated(Shared.pubkeyStr, Shared.orderBookId)

  let decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch (field(dict, "event_type"), optString(dict, "market_pubkey")) {
      | (Some(JSON.String("settled")), Some(marketPubkey)) => Ok(Settled(marketPubkey))
      | (Some(JSON.String("created")), Some(marketPubkey)) => Ok(Created(marketPubkey))
      | (Some(JSON.String("opened")), Some(marketPubkey)) => Ok(Opened(marketPubkey))
      | (Some(JSON.String("paused")), Some(marketPubkey)) => Ok(Paused(marketPubkey))
      | (Some(JSON.String("orderbook_created")), Some(marketPubkey)) =>
        switch optString(dict, "orderbook_id") {
        | Some(orderbookId) => Ok(OrderbookCreated(marketPubkey, orderbookId))
        | None => Spice.error("orderbook_created market event missing orderbook_id", json)
        }
      | (Some(JSON.String(other)), _) => Spice.error(`unknown market event_type: ${other}`, json)
      | _ => Spice.error("market event missing event_type/market_pubkey", json)
      }
    | _ => Spice.error("market event is not an object", json)
    }
}

// ── Auth (internally tagged on `status`) ──────────────────────────────────────
module AuthUpdate = {
  type t =
    | Authenticated(Shared.pubkeyStr)
    | Anonymous(option<string>)

  let decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch field(dict, "status") {
      | Some(JSON.String("authenticated")) =>
        switch optString(dict, "wallet") {
        | Some(wallet) => Ok(Authenticated(wallet))
        | None => Spice.error("authenticated auth update missing wallet", json)
        }
      | Some(JSON.String("anonymous")) => Ok(Anonymous(optString(dict, "reason")))
      | _ => Spice.error("unknown auth status", json)
      }
    | _ => Spice.error("auth update is not an object", json)
    }
}

// ── Price history (internally tagged on `event_type`) ─────────────────────────
// Reuses `PriceHistory.orderbookPriceCandle` (t/m/o/h/l/c/v/bb/ba) for both the
// snapshot candles and the single-candle update (decoded off the same object).
module WsPriceHistory = {
  type snapshot = {
    orderbookId: Shared.orderBookId,
    resolution: Shared.Resolution.t,
    prices: array<PriceHistory.orderbookPriceCandle>,
    lastTimestamp?: float,
    serverTime?: float,
  }

  type update = {
    orderbookId: Shared.orderBookId,
    resolution: Shared.Resolution.t,
    candle: PriceHistory.orderbookPriceCandle,
  }

  type heartbeat = {
    serverTime: float,
    lastProcessed?: float,
  }

  type t =
    | Snapshot(snapshot)
    | Update(update)
    | Heartbeat(heartbeat)

  let decodeSnapshot = (json: JSON.t): result<snapshot, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch (field(dict, "orderbook_id"), field(dict, "resolution"), field(dict, "prices")) {
      | (Some(JSON.String(orderbookId)), Some(resolutionJson), Some(pricesJson)) =>
        switch (
          Shared.Resolution.t_decode(resolutionJson),
          Spice.arrayFromJson(PriceHistory.orderbookPriceCandle_decode, pricesJson),
        ) {
        | (Ok(resolution), Ok(prices)) =>
          Ok({
            orderbookId,
            resolution,
            prices,
            lastTimestamp: ?optFloat(dict, "last_timestamp"),
            serverTime: ?optFloat(dict, "server_time"),
          })
        | (Error(error), _) | (_, Error(error)) => Error(error)
        }
      | _ => Spice.error("price_history snapshot missing fields", json)
      }
    | _ => Spice.error("price_history snapshot is not an object", json)
    }

  let decodeUpdate = (json: JSON.t): result<update, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch (field(dict, "orderbook_id"), field(dict, "resolution")) {
      | (Some(JSON.String(orderbookId)), Some(resolutionJson)) =>
        switch (Shared.Resolution.t_decode(resolutionJson), PriceHistory.orderbookPriceCandle_decode(json)) {
        | (Ok(resolution), Ok(candle)) => Ok({orderbookId, resolution, candle})
        | (Error(error), _) | (_, Error(error)) => Error(error)
        }
      | _ => Spice.error("price_history update missing fields", json)
      }
    | _ => Spice.error("price_history update is not an object", json)
    }

  let decodeHeartbeat = (json: JSON.t): result<heartbeat, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch optFloat(dict, "server_time") {
      | Some(serverTime) => Ok({serverTime, lastProcessed: ?optFloat(dict, "last_processed")})
      | None => Spice.error("price_history heartbeat missing server_time", json)
      }
    | _ => Spice.error("price_history heartbeat is not an object", json)
    }

  let decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch field(dict, "event_type") {
      | Some(JSON.String("snapshot")) => decodeSnapshot(json)->Result.map(value => Snapshot(value))
      | Some(JSON.String("update")) => decodeUpdate(json)->Result.map(value => Update(value))
      | Some(JSON.String("heartbeat")) => decodeHeartbeat(json)->Result.map(value => Heartbeat(value))
      | _ => Spice.error("unknown price_history event_type", json)
      }
    | _ => Spice.error("price_history payload is not an object", json)
    }
}

// ── Deposit price (OHLCV per resolution; internally tagged on `event_type`) ───
module WsDepositPrice = {
  type snapshot = {
    depositAsset: Shared.pubkeyStr,
    resolution: Shared.Resolution.t,
    prices: array<PriceHistory.depositPriceCandle>,
  }

  type candle = {
    depositAsset: Shared.pubkeyStr,
    resolution: Shared.Resolution.t,
    // Candle open / close time (unix ms).
    t: float,
    tc: float,
    // Close price (raw Binance decimal string).
    c: string,
  }

  type tick = {
    depositAsset: Shared.pubkeyStr,
    price: string,
    eventTime: float,
  }

  type t =
    | Snapshot(snapshot)
    | Candle(candle)
    | Price(tick)

  let decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch field(dict, "event_type") {
      | Some(JSON.String("snapshot")) =>
        switch (field(dict, "deposit_asset"), field(dict, "resolution"), field(dict, "prices")) {
        | (Some(JSON.String(depositAsset)), Some(resolutionJson), Some(pricesJson)) =>
          switch (
            Shared.Resolution.t_decode(resolutionJson),
            Spice.arrayFromJson(PriceHistory.depositPriceCandle_decode, pricesJson),
          ) {
          | (Ok(resolution), Ok(prices)) => Ok(Snapshot({depositAsset, resolution, prices}))
          | (Error(error), _) | (_, Error(error)) => Error(error)
          }
        | _ => Spice.error("deposit_price snapshot missing fields", json)
        }
      | Some(JSON.String("candle")) =>
        switch (
          field(dict, "deposit_asset"),
          field(dict, "resolution"),
          field(dict, "t"),
          field(dict, "tc"),
          field(dict, "c"),
        ) {
        | (
            Some(JSON.String(depositAsset)),
            Some(resolutionJson),
            Some(JSON.Number(t)),
            Some(JSON.Number(tc)),
            Some(JSON.String(c)),
          ) =>
          switch Shared.Resolution.t_decode(resolutionJson) {
          | Ok(resolution) => Ok(Candle({depositAsset, resolution, t, tc, c}))
          | Error(error) => Error(error)
          }
        | _ => Spice.error("deposit_price candle missing fields", json)
        }
      | Some(JSON.String("price")) =>
        switch (field(dict, "deposit_asset"), field(dict, "price"), field(dict, "event_time")) {
        | (Some(JSON.String(depositAsset)), Some(JSON.String(price)), Some(JSON.Number(eventTime))) =>
          Ok(Price({depositAsset, price, eventTime}))
        | _ => Spice.error("deposit_price tick missing fields", json)
        }
      | _ => Spice.error("unknown deposit_price event_type", json)
      }
    | _ => Spice.error("deposit_price payload is not an object", json)
    }
}

// ── Deposit asset price (live spot; internally tagged on `event_type`) ────────
module WsDepositAssetPrice = {
  type snapshot = {
    depositAsset: Shared.pubkeyStr,
    price: string,
  }

  type tick = {
    depositAsset: Shared.pubkeyStr,
    price: string,
    eventTime: float,
  }

  type t =
    | Snapshot(snapshot)
    | Price(tick)

  let decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch field(dict, "event_type") {
      | Some(JSON.String("snapshot")) =>
        switch (field(dict, "deposit_asset"), field(dict, "price")) {
        | (Some(JSON.String(depositAsset)), Some(JSON.String(price))) =>
          Ok(Snapshot({depositAsset, price}))
        | _ => Spice.error("deposit_asset_price snapshot missing fields", json)
        }
      | Some(JSON.String("price")) =>
        switch (field(dict, "deposit_asset"), field(dict, "price"), field(dict, "event_time")) {
        | (Some(JSON.String(depositAsset)), Some(JSON.String(price)), Some(JSON.Number(eventTime))) =>
          Ok(Price({depositAsset, price, eventTime}))
        | _ => Spice.error("deposit_asset_price tick missing fields", json)
        }
      | _ => Spice.error("unknown deposit_asset_price event_type", json)
      }
    | _ => Spice.error("deposit_asset_price payload is not an object", json)
    }
}

// ── User leaf events (timestamps are ISO-8601 strings) ────────────────────────
// The balance tree is `Order.userMarketBalance` — the same type the user
// snapshot carries (and `Position.UserMarketBalanceIndex` consumes), matching
// the Rust wire layout where both events share `UserMarketBalance`.
type userBalanceUpdate = {
  marketPubkey: Shared.pubkeyStr,
  marketBalance: Order.userMarketBalance,
  timestamp: string,
}

type globalDepositUpdate = {
  mint: Shared.pubkeyStr,
  balance: string,
  timestamp: string,
}

type nonceUpdate = {
  userPubkey: Shared.pubkeyStr,
  newNonce: float,
  timestamp: string,
}

let userBalanceUpdateDecode = (json: JSON.t): result<userBalanceUpdate, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch (field(dict, "market_pubkey"), field(dict, "market_balance"), field(dict, "timestamp")) {
    | (Some(JSON.String(marketPubkey)), Some(balanceJson), Some(JSON.String(timestamp))) =>
      switch Order.userMarketBalance_decode(balanceJson) {
      | Ok(marketBalance) => Ok({marketPubkey, marketBalance, timestamp})
      | Error(error) => Error(error)
      }
    | _ => Spice.error("user balance update missing fields", json)
    }
  | _ => Spice.error("user balance update is not an object", json)
  }

let globalDepositUpdateDecode = (json: JSON.t): result<globalDepositUpdate, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch (field(dict, "mint"), field(dict, "balance"), field(dict, "timestamp")) {
    | (Some(JSON.String(mint)), Some(JSON.String(balance)), Some(JSON.String(timestamp))) =>
      Ok({mint, balance, timestamp})
    | _ => Spice.error("global deposit update missing fields", json)
    }
  | _ => Spice.error("global deposit update is not an object", json)
  }

let nonceUpdateDecode = (json: JSON.t): result<nonceUpdate, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch (field(dict, "user_pubkey"), field(dict, "new_nonce"), field(dict, "timestamp")) {
    | (Some(JSON.String(userPubkey)), Some(JSON.Number(newNonce)), Some(JSON.String(timestamp))) =>
      Ok({userPubkey, newNonce, timestamp})
    | _ => Spice.error("nonce update missing fields", json)
    }
  | _ => Spice.error("nonce update is not an object", json)
  }

// ── User update (internally tagged on `event_type`) ───────────────────────────
// The `snapshot` and `order` events carry the largest nested wire trees (orders,
// trigger orders, balances); their payload types + decoders live in `Order.res`
// and feed the stateful `OrderState` containers.
module UserUpdate = {
  type t =
    | Snapshot(Order.userSnapshot)
    | Order(Order.OrderEvent.t)
    | BalanceUpdate(userBalanceUpdate)
    | GlobalDepositUpdate(globalDepositUpdate)
    | NonceUpdate(nonceUpdate)
    | NotificationPush(Notification.notification)

  let decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch field(dict, "event_type") {
      | Some(JSON.String("snapshot")) =>
        Order.userSnapshotDecode(json)->Result.map(value => Snapshot(value))
      | Some(JSON.String("order")) =>
        Order.OrderEvent.decode(json)->Result.map(value => Order(value))
      | Some(JSON.String("market_balance_update")) =>
        userBalanceUpdateDecode(json)->Result.map(value => BalanceUpdate(value))
      | Some(JSON.String("global_deposit_update")) =>
        globalDepositUpdateDecode(json)->Result.map(value => GlobalDepositUpdate(value))
      | Some(JSON.String("nonce")) => nonceUpdateDecode(json)->Result.map(value => NonceUpdate(value))
      | Some(JSON.String("notification")) =>
        switch field(dict, "notification") {
        | Some(notificationJson) =>
          Notification.notificationDecode(notificationJson)->Result.map(value => NotificationPush(value))
        | None => Spice.error("user notification event missing 'notification'", json)
        }
      | _ => Spice.error("unknown user event_type", json)
      }
    | _ => Spice.error("user payload is not an object", json)
    }
}

// ── Kind + MessageIn ──────────────────────────────────────────────────────────
// `ErrorFrame` mirrors the Rust `Kind::Error` (renamed so it doesn't shadow the
// `result` `Error` constructor used throughout the decoders).
type kind =
  | BookUpdate(Orderbook.orderBook)
  | Trades(wsTrade)
  | User(UserUpdate.t)
  | Ticker(wsTicker)
  | PriceHistory(WsPriceHistory.t)
  | Market(MarketEvent.t)
  | DepositPrice(WsDepositPrice.t)
  | DepositAssetPrice(WsDepositAssetPrice.t)
  | Auth(AuthUpdate.t)
  | Pong
  | ErrorFrame(wsErrorFrame)

type messageIn = {
  kind: kind,
  // Protocol version echoed by the backend (e.g. 0.1); informational.
  version: float,
}

let kindDecode = (json: JSON.t): result<kind, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    let data = field(dict, "data")->Option.getOr(JSON.Null)
    switch field(dict, "type") {
    | Some(JSON.String("book_update")) => Orderbook.orderBook_decode(data)->Result.map(value => BookUpdate(value))
    | Some(JSON.String("trades")) => wsTrade_decode(data)->Result.map(value => Trades(value))
    | Some(JSON.String("ticker")) => wsTickerDecode(data)->Result.map(value => Ticker(value))
    | Some(JSON.String("pong")) => Ok(Pong)
    | Some(JSON.String("error")) => wsErrorFrame_decode(data)->Result.map(value => ErrorFrame(value))
    | Some(JSON.String("user")) => UserUpdate.decode(data)->Result.map(value => User(value))
    | Some(JSON.String("price_history")) => WsPriceHistory.decode(data)->Result.map(value => PriceHistory(value))
    | Some(JSON.String("auth")) => AuthUpdate.decode(data)->Result.map(value => Auth(value))
    | Some(JSON.String("market")) => MarketEvent.decode(data)->Result.map(value => Market(value))
    | Some(JSON.String("deposit_price")) => WsDepositPrice.decode(data)->Result.map(value => DepositPrice(value))
    | Some(JSON.String("deposit_asset_price")) =>
      WsDepositAssetPrice.decode(data)->Result.map(value => DepositAssetPrice(value))
    | Some(JSON.String(other)) => Spice.error(`unknown ws message type: ${other}`, json)
    | _ => Spice.error("ws message missing 'type'", json)
    }
  | _ => Spice.error("ws message is not an object", json)
  }

// Decode a parsed inbound frame. Strips `null`s first (so optional fields tolerate
// both absent and null), then dispatches on the `type` discriminator. Decode
// failures surface as `SdkError.Ws(DeserializationError(_))`.
let decodeMessage = (json: JSON.t): result<messageIn, SdkError.t> => {
  let stripped = SdkError.stripNulls(json)
  switch kindDecode(stripped) {
  | Ok(kind) =>
    let version = switch stripped {
    | JSON.Object(dict) => optFloat(dict, "version")->Option.getOr(0.0)
    | _ => 0.0
    }
    Ok({kind, version})
  | Error(error) => Error(SdkError.Ws(SdkError.DeserializationError(error.message)))
  }
}
