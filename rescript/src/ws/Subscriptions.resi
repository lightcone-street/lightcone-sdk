// WebSocket subscription params + outbound message serialization. Public surface:
// the `SubscribeParams`/`UnsubscribeParams` sub-modules, the `messageOut` enum,
// and the helpers the connection layer needs (`toJson`, `subscriptionKey`,
// `toUnsubscribeParams`, `matchesUnsubscribe`). The per-channel JSON encoders and
// id-sorting/aggregation helpers are internal.

module SubscribeParams: {
  @genType
  type booksParams = {orderbookIds: array<Shared.orderBookId>, nSigFigs?: int, mantissa?: int}

  @genType
  type priceHistoryParams = {
    orderbookId: Shared.orderBookId,
    resolution: Shared.Resolution.t,
    includeOhlcv: bool,
  }

  @genType
  type depositPriceParams = {depositAsset: Shared.pubkeyStr, resolution: Shared.Resolution.t}

  @genType
  type t =
    | Books(booksParams)
    | Trades(array<Shared.orderBookId>)
    | User(Shared.pubkeyStr)
    | PriceHistory(priceHistoryParams)
    | Ticker(array<Shared.orderBookId>)
    | Market(Shared.pubkeyStr)
    | DepositPrice(depositPriceParams)
    | DepositAssetPrice(Shared.pubkeyStr)
}

module UnsubscribeParams: {
  @genType
  type booksParams = {orderbookIds: array<Shared.orderBookId>, nSigFigs?: int, mantissa?: int}

  @genType
  type priceHistoryParams = {orderbookId: Shared.orderBookId, resolution: Shared.Resolution.t}

  @genType
  type depositPriceParams = {depositAsset: Shared.pubkeyStr, resolution: Shared.Resolution.t}

  @genType
  type t =
    | Books(booksParams)
    | Trades(array<Shared.orderBookId>)
    | User(Shared.pubkeyStr)
    | PriceHistory(priceHistoryParams)
    | Ticker(array<Shared.orderBookId>)
    | Market(Shared.pubkeyStr)
    | DepositPrice(depositPriceParams)
    | DepositAssetPrice(Shared.pubkeyStr)
}

// Outbound message. `Ping` is an application-level keepalive.
type messageOut =
  | Subscribe(SubscribeParams.t)
  | Unsubscribe(UnsubscribeParams.t)
  | Ping

// Outbound message → wire JSON: `{"method": …, "params": …}` (Ping omits params).
let toJson: messageOut => JSON.t

// Stable per-subscription tracking key (order-independent on ids; book
// aggregation normalized into the suffix).
let subscriptionKey: SubscribeParams.t => string

// Derive the matching unsubscribe for a tracked subscription.
let toUnsubscribeParams: SubscribeParams.t => UnsubscribeParams.t

// Whether a tracked subscription is removed by this unsubscribe.
let matchesUnsubscribe: (SubscribeParams.t, UnsubscribeParams.t) => bool
