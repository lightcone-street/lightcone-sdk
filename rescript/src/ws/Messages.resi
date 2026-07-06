// Inbound WebSocket messages — the envelope plus per-channel payload wire types.
// Public surface: the gentype-exported per-channel payload modules, the `Kind.t` /
// `t` envelope, and the single `decodeMessage` entry point. All the hand-decoders
// (each module's `decode`) and spice codecs are internal.

// ── Pong / Error ──────────────────────────────────────────────────────────────
module ErrorFrame: {
  @genType
  type t = {
    error: string,
    code?: string,
    orderbookId?: string,
    nSigFigs?: int,
    mantissa?: int,
    walletAddress?: string,
    depositAsset?: string,
    hint?: string,
    details?: string,
  }
}

// ── Trades ────────────────────────────────────────────────────────────────────
module Trade: {
  @genType
  type t = {
    orderbookId: Shared.orderBookId,
    tradeId: string,
    // ISO-8601 string (RFC 3339 `DateTime<Utc>`).
    timestamp: string,
    price: string,
    size: string,
    side: Shared.Side.t,
    // Monotonic per-orderbook counter.
    sequence: float,
  }
}

// ── Ticker ────────────────────────────────────────────────────────────────────
module Ticker: {
  @genType
  type t = {
    orderbookId: Shared.orderBookId,
    bestBid?: string,
    bestAsk?: string,
    mid?: string,
  }
}

// ── Market lifecycle events ───────────────────────────────────────────────────
module MarketEvent: {
  @genType
  type t =
    | Settled(Shared.pubkeyStr)
    | Created(Shared.pubkeyStr)
    | Opened(Shared.pubkeyStr)
    | Paused(Shared.pubkeyStr)
    | OrderbookCreated(Shared.pubkeyStr, Shared.orderBookId)
}

// ── Auth ──────────────────────────────────────────────────────────────────────
module AuthUpdate: {
  @genType
  type t =
    | Authenticated(Shared.pubkeyStr)
    | Anonymous(option<string>)
}

// ── Price history ─────────────────────────────────────────────────────────────
module WsPriceHistory: {
  @genType
  type snapshot = {
    orderbookId: Shared.orderBookId,
    resolution: Shared.Resolution.t,
    prices: array<PriceHistory__Raw.OrderbookCandle.t>,
    lastTimestamp?: float,
    serverTime?: float,
  }

  @genType
  type update = {
    orderbookId: Shared.orderBookId,
    resolution: Shared.Resolution.t,
    candle: PriceHistory__Raw.OrderbookCandle.t,
  }

  @genType
  type heartbeat = {
    serverTime: float,
    lastProcessed?: float,
  }

  @genType
  type t =
    | Snapshot(snapshot)
    | Update(update)
    | Heartbeat(heartbeat)
}

// ── Deposit price (OHLCV per resolution) ──────────────────────────────────────
module WsDepositPrice: {
  @genType
  type snapshot = {
    depositAsset: Shared.pubkeyStr,
    resolution: Shared.Resolution.t,
    prices: array<PriceHistory__Raw.DepositCandle.t>,
  }

  @genType
  type candle = {
    depositAsset: Shared.pubkeyStr,
    resolution: Shared.Resolution.t,
    // Candle open / close time (unix ms).
    t: float,
    tc: float,
    // Close price (raw Binance decimal string).
    c: string,
  }

  @genType
  type tick = {
    depositAsset: Shared.pubkeyStr,
    price: string,
    eventTime: float,
  }

  @genType
  type t =
    | Snapshot(snapshot)
    | Candle(candle)
    | Price(tick)
}

// ── Deposit asset price (live spot) ───────────────────────────────────────────
module WsDepositAssetPrice: {
  @genType
  type snapshot = {
    depositAsset: Shared.pubkeyStr,
    price: string,
  }

  @genType
  type tick = {
    depositAsset: Shared.pubkeyStr,
    price: string,
    eventTime: float,
  }

  @genType
  type t =
    | Snapshot(snapshot)
    | Price(tick)
}

// ── User leaf events (timestamps are ISO-8601 strings) ────────────────────────
// The balance tree is `Order__Raw.UserMarketBalance.t` — the same type the user
// snapshot carries (and `Position.State` consumes).
module UserBalanceUpdate: {
  @genType
  type t = {
    marketPubkey: Shared.pubkeyStr,
    marketBalance: Order__Raw.UserMarketBalance.t,
    timestamp: string,
  }
}

module GlobalDepositUpdate: {
  @genType
  type t = {
    mint: Shared.pubkeyStr,
    balance: string,
    timestamp: string,
  }
}

module NonceUpdate: {
  @genType
  type t = {
    userPubkey: Shared.pubkeyStr,
    newNonce: float,
    timestamp: string,
  }
}

// ── User update ───────────────────────────────────────────────────────────────
// The `snapshot` / `order` payload types live in `Order__Raw`; the stateful
// `Order.State` containers consume them.
module UserUpdate: {
  @genType
  type t =
    | Snapshot(Order__Raw.UserSnapshot.t)
    | Order(Order__Raw.Event.t)
    | BalanceUpdate(UserBalanceUpdate.t)
    | GlobalDepositUpdate(GlobalDepositUpdate.t)
    | NonceUpdate(NonceUpdate.t)
    | NotificationPush(Notification__Model.t)
}

// ── Kind + the message envelope ───────────────────────────────────────────────
module Kind: {
  @genType
  type t =
    | BookUpdate(Orderbook__Raw.Book.t)
    | Trades(Trade.t)
    | User(UserUpdate.t)
    | Ticker(Ticker.t)
    | PriceHistory(WsPriceHistory.t)
    | Market(MarketEvent.t)
    | DepositPrice(WsDepositPrice.t)
    | DepositAssetPrice(WsDepositAssetPrice.t)
    | Auth(AuthUpdate.t)
    | Pong
    | ErrorFrame(ErrorFrame.t)
}

// The file's primary type: one decoded inbound message.
@genType
type t = {
  kind: Kind.t,
  // Protocol version echoed by the backend (e.g. 0.1); informational.
  version: float,
}

// Decode a parsed inbound frame (strips nulls, dispatches on `type`). Failures
// surface as `SdkError.Ws(DeserializationError(_))`.
let decodeMessage: JSON.t => result<t, SdkError.t>
