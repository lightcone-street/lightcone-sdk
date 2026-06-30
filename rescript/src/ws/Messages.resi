// Inbound WebSocket messages (counterpart of the Rust `ws::{MessageIn, Kind}`).
// Public surface: the gentype-exported per-channel payload types/sub-modules, the
// `kind`/`messageIn` envelope, and the single `decodeMessage` entry point. All the
// hand-decoders (`*Decode`, each sub-module's `decode`) and spice codecs are
// internal.

// ── Pong / Error ──────────────────────────────────────────────────────────────
@genType
type wsErrorFrame = {
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

// ── Trades ────────────────────────────────────────────────────────────────────
@genType
type wsTrade = {
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

// ── Ticker ────────────────────────────────────────────────────────────────────
@genType
type wsTicker = {
  orderbookId: Shared.orderBookId,
  bestBid?: string,
  bestAsk?: string,
  mid?: string,
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
    prices: array<PriceHistory.orderbookPriceCandle>,
    lastTimestamp?: float,
    serverTime?: float,
  }

  @genType
  type update = {
    orderbookId: Shared.orderBookId,
    resolution: Shared.Resolution.t,
    candle: PriceHistory.orderbookPriceCandle,
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
    prices: array<PriceHistory.depositPriceCandle>,
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

// ── User balances ─────────────────────────────────────────────────────────────
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

// ── User leaf events (timestamps are ISO-8601 strings) ────────────────────────
@genType
type userBalanceUpdate = {
  marketPubkey: Shared.pubkeyStr,
  marketBalance: userMarketBalance,
  timestamp: string,
}

@genType
type globalDepositUpdate = {
  mint: Shared.pubkeyStr,
  balance: string,
  timestamp: string,
}

@genType
type nonceUpdate = {
  userPubkey: Shared.pubkeyStr,
  newNonce: float,
  timestamp: string,
}

// ── User update ───────────────────────────────────────────────────────────────
// `Snapshot`/`Order` carry the raw (null-stripped) wire payload until the typed
// stateful containers land.
module UserUpdate: {
  @genType
  type t =
    | Snapshot(JSON.t)
    | Order(JSON.t)
    | BalanceUpdate(userBalanceUpdate)
    | GlobalDepositUpdate(globalDepositUpdate)
    | NonceUpdate(nonceUpdate)
    | NotificationPush(Notification.notification)
}

// ── Kind + MessageIn ──────────────────────────────────────────────────────────
@genType
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

@genType
type messageIn = {
  kind: kind,
  // Protocol version echoed by the backend (e.g. 0.1); informational.
  version: float,
}

// Decode a parsed inbound frame (strips nulls, dispatches on `type`). Failures
// surface as `SdkError.Ws(DeserializationError(_))`.
let decodeMessage: JSON.t => result<messageIn, SdkError.t>
