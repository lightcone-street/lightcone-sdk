module SubscribeParams = {
  type booksParams = {orderbookIds: array<Shared.orderBookId>, nSigFigs?: int, mantissa?: int}

  type priceHistoryParams = {
    orderbookId: Shared.orderBookId,
    resolution: Shared.Resolution.t,
    includeOhlcv: bool,
  }

  type depositPriceParams = {depositAsset: Shared.pubkeyStr, resolution: Shared.Resolution.t}

  type t =
    | Books(booksParams)
    | Trades(array<Shared.orderBookId>)
    | User(Shared.pubkeyStr)
    | WalletDepositBalances(Shared.pubkeyStr)
    | PriceHistory(priceHistoryParams)
    | Ticker(array<Shared.orderBookId>)
    | Market(Shared.pubkeyStr)
    | DepositPrice(depositPriceParams)
    | DepositAssetPrice(Shared.pubkeyStr)
}

module UnsubscribeParams = {
  type booksParams = {orderbookIds: array<Shared.orderBookId>, nSigFigs?: int, mantissa?: int}

  // No `include_ohlcv` on unsubscribe — matched server-side on (orderbook, resolution).
  type priceHistoryParams = {orderbookId: Shared.orderBookId, resolution: Shared.Resolution.t}

  type depositPriceParams = {depositAsset: Shared.pubkeyStr, resolution: Shared.Resolution.t}

  type t =
    | Books(booksParams)
    | Trades(array<Shared.orderBookId>)
    | User(Shared.pubkeyStr)
    | WalletDepositBalances(Shared.pubkeyStr)
    | PriceHistory(priceHistoryParams)
    | Ticker(array<Shared.orderBookId>)
    | Market(Shared.pubkeyStr)
    | DepositPrice(depositPriceParams)
    | DepositAssetPrice(Shared.pubkeyStr)
}

// The file's primary type: one outbound message. `Ping` is an application-level
// keepalive (distinct from the WS protocol ping frame).
type t =
  | Subscribe(SubscribeParams.t)
  | Unsubscribe(UnsubscribeParams.t)
  | Ping

// ── Serialization helpers ─────────────────────────────────────────────────────
// Orderbook ids are sorted before sending so a subscription's wire form (and
// its tracking key) is independent of argument order.
let sortedIds = (ids: array<Shared.orderBookId>): array<Shared.orderBookId> =>
  ids->Array.toSorted((left, right) => String.compare(left, right))

let idsJson = (ids: array<Shared.orderBookId>): JSON.t => JSON.Array(
  sortedIds(ids)->Array.map(id => JSON.String(id)),
)

// Aggregation params, normalized and with absent keys omitted: full precision →
// no keys; (2..4) → `nSigFigs` only; (5, m) → both. Mirrors the backend contract
// (unknown/snake_case/null aggregation params are rejected server-side).
let aggregationFields = (nSigFigs: option<int>, mantissa: option<int>): array<(string, JSON.t)> => {
  let normalized = Orderbook.Aggregation.normalized({?nSigFigs, ?mantissa})
  let fields = []
  normalized.nSigFigs->Option.forEach(value =>
    fields->Array.push(("nSigFigs", JSON.Number(Int.toFloat(value))))
  )
  normalized.mantissa->Option.forEach(value =>
    fields->Array.push(("mantissa", JSON.Number(Int.toFloat(value))))
  )
  fields
}

let jsonObject = (fields: array<(string, JSON.t)>): JSON.t => JSON.Object(Dict.fromArray(fields))

let subscribeParamsToJson = (params: SubscribeParams.t): JSON.t =>
  switch params {
  | Books({orderbookIds, ?nSigFigs, ?mantissa}) => {
      let fields = [("type", JSON.String("book_update")), ("orderbook_ids", idsJson(orderbookIds))]
      aggregationFields(nSigFigs, mantissa)->Array.forEach(field => fields->Array.push(field))
      jsonObject(fields)
    }
  | Trades(orderbookIds) =>
    jsonObject([("type", JSON.String("trades")), ("orderbook_ids", idsJson(orderbookIds))])
  | WalletDepositBalances(walletAddress) =>
    jsonObject([
      ("type", JSON.String("wallet_deposit_balances")),
      ("wallet_address", JSON.String(walletAddress)),
    ])
  | User(walletAddress) =>
    jsonObject([("type", JSON.String("user")), ("wallet_address", JSON.String(walletAddress))])
  | PriceHistory({orderbookId, resolution, includeOhlcv}) =>
    jsonObject([
      ("type", JSON.String("price_history")),
      ("orderbook_id", JSON.String(orderbookId)),
      ("resolution", Shared.Resolution.t_encode(resolution)),
      ("include_ohlcv", JSON.Boolean(includeOhlcv)),
    ])
  | Ticker(orderbookIds) =>
    jsonObject([("type", JSON.String("ticker")), ("orderbook_ids", idsJson(orderbookIds))])
  | Market(marketPubkey) =>
    jsonObject([("type", JSON.String("market")), ("market_pubkey", JSON.String(marketPubkey))])
  | DepositPrice({depositAsset, resolution}) =>
    jsonObject([
      ("type", JSON.String("deposit_price")),
      ("deposit_asset", JSON.String(depositAsset)),
      ("resolution", Shared.Resolution.t_encode(resolution)),
    ])
  | DepositAssetPrice(depositAsset) =>
    jsonObject([
      ("type", JSON.String("deposit_asset_price")),
      ("deposit_asset", JSON.String(depositAsset)),
    ])
  }

let unsubscribeParamsToJson = (params: UnsubscribeParams.t): JSON.t =>
  switch params {
  | Books({orderbookIds, ?nSigFigs, ?mantissa}) => {
      let fields = [("type", JSON.String("book_update")), ("orderbook_ids", idsJson(orderbookIds))]
      aggregationFields(nSigFigs, mantissa)->Array.forEach(field => fields->Array.push(field))
      jsonObject(fields)
    }
  | Trades(orderbookIds) =>
    jsonObject([("type", JSON.String("trades")), ("orderbook_ids", idsJson(orderbookIds))])
  | WalletDepositBalances(walletAddress) =>
    jsonObject([
      ("type", JSON.String("wallet_deposit_balances")),
      ("wallet_address", JSON.String(walletAddress)),
    ])
  | User(walletAddress) =>
    jsonObject([("type", JSON.String("user")), ("wallet_address", JSON.String(walletAddress))])
  | PriceHistory({orderbookId, resolution}) =>
    jsonObject([
      ("type", JSON.String("price_history")),
      ("orderbook_id", JSON.String(orderbookId)),
      ("resolution", Shared.Resolution.t_encode(resolution)),
    ])
  | Ticker(orderbookIds) =>
    jsonObject([("type", JSON.String("ticker")), ("orderbook_ids", idsJson(orderbookIds))])
  | Market(marketPubkey) =>
    jsonObject([("type", JSON.String("market")), ("market_pubkey", JSON.String(marketPubkey))])
  | DepositPrice({depositAsset, resolution}) =>
    jsonObject([
      ("type", JSON.String("deposit_price")),
      ("deposit_asset", JSON.String(depositAsset)),
      ("resolution", Shared.Resolution.t_encode(resolution)),
    ])
  | DepositAssetPrice(depositAsset) =>
    jsonObject([
      ("type", JSON.String("deposit_asset_price")),
      ("deposit_asset", JSON.String(depositAsset)),
    ])
  }

// Outbound message → wire JSON: `{"method": …, "params": …}` (Ping omits params).
let toJson = (message: t): JSON.t =>
  switch message {
  | Subscribe(params) =>
    jsonObject([("method", JSON.String("subscribe")), ("params", subscribeParamsToJson(params))])
  | Unsubscribe(params) =>
    jsonObject([
      ("method", JSON.String("unsubscribe")),
      ("params", unsubscribeParamsToJson(params)),
    ])
  | Ping => jsonObject([("method", JSON.String("ping"))])
  }

// ── Subscription tracking (for the connection's resubscribe-on-reconnect) ──────
// Sorted, comma-joined ids — order-independent.
let idsKey = (ids: array<Shared.orderBookId>): string => sortedIds(ids)->Array.join(",")

// Stable per-subscription key. Full-precision books keep the pre-aggregation
// shape (`book:a,b`); grouped books append the aggregation suffix.
let subscriptionKey = (params: SubscribeParams.t): string =>
  switch params {
  | Books({orderbookIds, ?nSigFigs, ?mantissa}) => {
      let aggregation = Orderbook.Aggregation.fromFrame(nSigFigs, mantissa)
      Orderbook.Aggregation.isFull(aggregation)
        ? `book:${idsKey(orderbookIds)}`
        : `book:${idsKey(orderbookIds)}:${Orderbook.Aggregation.keySuffix(aggregation)}`
    }
  | Trades(orderbookIds) => `trades:${idsKey(orderbookIds)}`
  | User(walletAddress) => `user:${walletAddress}`
  | WalletDepositBalances(walletAddress) => `wallet_deposit_balances:${walletAddress}`
  | PriceHistory({orderbookId, resolution}) =>
    `price_history:${orderbookId}:${Shared.Resolution.toString(resolution)}`
  | Ticker(orderbookIds) => `ticker:${idsKey(orderbookIds)}`
  | Market(marketPubkey) => `market:${marketPubkey}`
  | DepositPrice({depositAsset, resolution}) =>
    `deposit_price:${depositAsset}:${Shared.Resolution.toString(resolution)}`
  | DepositAssetPrice(depositAsset) => `deposit_asset_price:${depositAsset}`
  }

// Same key shape, computed from an unsubscribe — used to match a tracked
// subscription for removal (set-equality on ids + normalized aggregation).
let unsubscribeKey = (params: UnsubscribeParams.t): string =>
  switch params {
  | Books({orderbookIds, ?nSigFigs, ?mantissa}) => {
      let aggregation = Orderbook.Aggregation.fromFrame(nSigFigs, mantissa)
      Orderbook.Aggregation.isFull(aggregation)
        ? `book:${idsKey(orderbookIds)}`
        : `book:${idsKey(orderbookIds)}:${Orderbook.Aggregation.keySuffix(aggregation)}`
    }
  | Trades(orderbookIds) => `trades:${idsKey(orderbookIds)}`
  | User(walletAddress) => `user:${walletAddress}`
  | WalletDepositBalances(walletAddress) => `wallet_deposit_balances:${walletAddress}`
  | PriceHistory({orderbookId, resolution}) =>
    `price_history:${orderbookId}:${Shared.Resolution.toString(resolution)}`
  | Ticker(orderbookIds) => `ticker:${idsKey(orderbookIds)}`
  | Market(marketPubkey) => `market:${marketPubkey}`
  | DepositPrice({depositAsset, resolution}) =>
    `deposit_price:${depositAsset}:${Shared.Resolution.toString(resolution)}`
  | DepositAssetPrice(depositAsset) => `deposit_asset_price:${depositAsset}`
  }

// Derive the matching unsubscribe for a tracked subscription.
let toUnsubscribeParams = (params: SubscribeParams.t): UnsubscribeParams.t =>
  switch params {
  | Books({orderbookIds, ?nSigFigs, ?mantissa}) => Books({orderbookIds, ?nSigFigs, ?mantissa})
  | Trades(orderbookIds) => Trades(orderbookIds)
  | User(walletAddress) => User(walletAddress)
  | WalletDepositBalances(walletAddress) => WalletDepositBalances(walletAddress)
  | PriceHistory({orderbookId, resolution, includeOhlcv: _}) =>
    PriceHistory({orderbookId, resolution})
  | Ticker(orderbookIds) => Ticker(orderbookIds)
  | Market(marketPubkey) => Market(marketPubkey)
  | DepositPrice({depositAsset, resolution}) => DepositPrice({depositAsset, resolution})
  | DepositAssetPrice(depositAsset) => DepositAssetPrice(depositAsset)
  }

// Whether a tracked subscription is removed by this unsubscribe (cross-channel
// never matches; book aggregation must match normalized).
let matchesUnsubscribe = (sub: SubscribeParams.t, unsub: UnsubscribeParams.t): bool =>
  subscriptionKey(sub) == unsubscribeKey(unsub)
