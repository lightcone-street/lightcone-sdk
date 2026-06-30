// Price-history domain — orderbook OHLCV / candle queries, deposit-token price
// candles, and the deposit-asset price snapshot. Public surface: the gentype
// candle / response / query types and the client query functions. The query-param
// validation helpers and the wire→domain conversion are internal.
//
// The two candle types KEEP their `@spice` codec because the WS message decoders
// in `ws/Messages.res` reuse `orderbookPriceCandle_decode` / `depositPriceCandle_decode`.
// Every other response codec stays internal (declared `@genType` only).

@genType @spice
type orderbookPriceCandle = {
  // Unix milliseconds (candle start).
  t: float,
  // Midpoint, open, high, low, close, volume, best bid, best ask. Every OHLCV
  // field is optional (candles with no trades carry only `t`/`m`).
  m?: string,
  o?: string,
  h?: string,
  l?: string,
  c?: string,
  v?: string,
  bb?: string,
  ba?: string,
}

@genType
type priceHistoryDecimals = {
  price: float,
  volume: float,
}

@genType
type orderbookPriceHistoryResponse = {
  orderbookId: Shared.orderBookId,
  resolution: Shared.Resolution.t,
  includeOhlcv: bool,
  prices: array<orderbookPriceCandle>,
  nextCursor?: float,
  hasMore: bool,
  decimals: priceHistoryDecimals,
}

@genType @spice
type depositPriceCandle = {
  // Unix milliseconds (candle open time).
  t: float,
  // Unix milliseconds (candle close time).
  tc: float,
  // Close price (raw Binance decimal string).
  c: string,
}

@genType
type depositPriceHistoryResponse = {
  depositAsset: Shared.pubkeyStr,
  binanceSymbol: string,
  resolution: Shared.Resolution.t,
  prices: array<depositPriceCandle>,
  nextCursor?: float,
  hasMore: bool,
}

@genType
type depositAssetPricesSnapshotResponse = {prices: dict<string>}

// A single data point on a price chart (simplified from the full candle).
@genType
type lineData = {
  // Unix milliseconds.
  time: float,
  // Midpoint value as a decimal string.
  value: string,
}

// Query options for orderbook price-history requests. `fromMs` / `toMs` / `cursor`
// are unix milliseconds; `limit` is 1..=1000.
@genType
type orderbookPriceHistoryQuery = {
  resolution: Shared.Resolution.t,
  fromMs?: float,
  toMs?: float,
  cursor?: float,
  limit?: float,
  includeOhlcv: bool,
}

// Query options for deposit-token price-history requests.
@genType
type depositPriceHistoryQuery = {
  resolution: Shared.Resolution.t,
  fromMs?: float,
  toMs?: float,
  cursor?: float,
  limit?: float,
}

// ── Client functions ──────────────────────────────────────────────────────────

// Orderbook price history. `fromMs` / `toMs` are unix milliseconds.
let get: (
  Client.t,
  ~orderbookId: string,
  ~resolution: Shared.Resolution.t,
  ~fromMs: float=?,
  ~toMs: float=?,
) => promise<result<orderbookPriceHistoryResponse, SdkError.t>>

// Deposit-token price history from the same REST endpoint.
let getDepositAsset: (
  Client.t,
  ~depositAsset: string,
  ~query: depositPriceHistoryQuery,
) => promise<result<depositPriceHistoryResponse, SdkError.t>>

// Snapshot of current prices for every active mint in `global_deposit_tokens`.
let getDepositAssetPricesSnapshot: Client.t => promise<
  result<depositAssetPricesSnapshotResponse, SdkError.t>,
>

// Simplified midpoint line data for charting (orderbook candles → `lineData`).
let getLineData: (
  Client.t,
  ~orderbookId: string,
  ~resolution: Shared.Resolution.t,
  ~fromMs: float=?,
  ~toMs: float=?,
  ~cursor: float=?,
  ~limit: float=?,
) => promise<result<array<lineData>, SdkError.t>>
