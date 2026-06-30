// Price-history domain — OHLCV / candle queries (mirrors the Rust
// `domain/price_history`).
//
// Reference shape (see `Trade.res`):
//   1. wire types (`@spice`)       — exact JSON the backend sends
//   2. domain / query types (`@genType`)
//   3. conversions (mirror the Rust `From<Wire> for LineData`)
//   4. client functions taking a `Client.t`, returning `promise<result<_, sdkError>>`
//
// Prices stay as wire strings (no precision loss, gentype-clean). Timestamps and
// cursors are floats (JS numbers, unix milliseconds). NOTE: the Rust query field
// names `from` / `to` are spelled `fromMs` / `toMs` here — `to` is a reserved
// ReScript keyword, and the backend requires these to be unix milliseconds.

// ── Wire types ────────────────────────────────────────────────────────────────
// Orderbook price candle. With `include_ohlcv=false` (default) candles with no
// trades carry only `t` (+ `m`); every OHLCV field is optional.
@spice
type orderbookPriceCandle = {
  // Unix milliseconds (candle start).
  t: float,
  // Midpoint: (best_bid + best_ask) / 2.
  m?: string,
  // Open.
  o?: string,
  // High.
  h?: string,
  // Low.
  l?: string,
  // Close.
  c?: string,
  // Volume.
  v?: string,
  // Best bid at this candle's time.
  bb?: string,
  // Best ask at this candle's time.
  ba?: string,
}

// Orderbook price-history decimals metadata.
@spice
type priceHistoryDecimals = {
  price: float,
  volume: float,
}

// Response for `GET /api/price-history?orderbook_id=…`.
@spice
type orderbookPriceHistoryResponse = {
  @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
  resolution: Shared.Resolution.t,
  @spice.key("include_ohlcv") includeOhlcv: bool,
  prices: array<orderbookPriceCandle>,
  @spice.key("next_cursor") nextCursor?: float,
  @spice.key("has_more") hasMore: bool,
  decimals: priceHistoryDecimals,
}

// Deposit-token price candle.
@spice
type depositPriceCandle = {
  // Unix milliseconds (candle open time).
  t: float,
  // Unix milliseconds (candle close time).
  tc: float,
  // Close price (raw Binance decimal string).
  c: string,
}

// Response for `GET /api/price-history?deposit_asset=…`.
@spice
type depositPriceHistoryResponse = {
  @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
  @spice.key("binance_symbol") binanceSymbol: string,
  resolution: Shared.Resolution.t,
  prices: array<depositPriceCandle>,
  @spice.key("next_cursor") nextCursor?: float,
  @spice.key("has_more") hasMore: bool,
}

// Response for `GET /api/deposit-asset-prices-snapshot` — mint → latest price.
@spice
type depositAssetPricesSnapshotResponse = {
  prices: dict<string>,
}

// ── Domain / query types ──────────────────────────────────────────────────────
// A single data point on a price chart (simplified from the full candle).
type lineData = {
  // Unix milliseconds.
  time: float,
  // Midpoint value as a decimal string.
  value: string,
}

// Query options for orderbook price-history requests. `fromMs` / `toMs` / `cursor`
// are unix milliseconds; `limit` is 1..=1000.
type orderbookPriceHistoryQuery = {
  resolution: Shared.Resolution.t,
  fromMs?: float,
  toMs?: float,
  cursor?: float,
  limit?: float,
  includeOhlcv: bool,
}

// Query options for deposit-token price-history requests.
type depositPriceHistoryQuery = {
  resolution: Shared.Resolution.t,
  fromMs?: float,
  toMs?: float,
  cursor?: float,
  limit?: float,
}

// ── Conversions ───────────────────────────────────────────────────────────────
// Mirrors Rust `impl From<OrderbookPriceCandle> for LineData` (midpoint, else
// close, else empty).
let lineDataOfOrderbookCandle = (candle: orderbookPriceCandle): lineData => {
  let value = switch candle.m {
  | Some(midpoint) => midpoint
  | None => candle.c->Option.getOr("")
  }
  {time: candle.t, value}
}

// ── Query-param validation (mirrors the Rust client guards) ───────────────────
// `from` / `to` / `cursor` must be unix milliseconds, not seconds.
let ensureUnixMilliseconds = (name: string, value: float): result<string, SdkError.t> =>
  value < 1.0e10
    ? Error(SdkError.Validation(`${name} must be a Unix timestamp in milliseconds, not seconds`))
    : Ok(Float.toString(value))

// `limit` must be an integer between 1 and 1000.
let ensurePageLimit = (value: float): result<string, SdkError.t> =>
  value < 1.0 || value > 1000.0
    ? Error(SdkError.Validation("limit must be an integer between 1 and 1000"))
    : Ok(Float.toString(value))

// Turn an optional numeric query value into a validated `(key, value)` pair.
let optionalParam = (
  key: string,
  value: option<float>,
  validate: float => result<string, SdkError.t>,
): result<option<(string, string)>, SdkError.t> =>
  switch value {
  | None => Ok(None)
  | Some(raw) => validate(raw)->Result.map(encoded => Some((key, encoded)))
  }

// Fold validated optional params onto a base param list, short-circuiting on the
// first validation error.
let collectParams = (
  base: array<(string, string)>,
  optionals: array<result<option<(string, string)>, SdkError.t>>,
): result<array<(string, string)>, SdkError.t> =>
  optionals->Array.reduce(Ok(base), (acc, item) =>
    switch (acc, item) {
    | (Error(_), _) => acc
    | (_, Error(error)) => Error(error)
    | (Ok(params), Ok(None)) => Ok(params)
    | (Ok(params), Ok(Some(pair))) => {
        params->Array.push(pair)
        Ok(params)
      }
    }
  )

// ── Client functions ──────────────────────────────────────────────────────────

// Orderbook price history with full pagination / OHLCV options.
let getWithQuery = async (
  client: Client.t,
  ~orderbookId: string,
  ~query: orderbookPriceHistoryQuery,
): result<orderbookPriceHistoryResponse, SdkError.t> => {
  let base = [("orderbook_id", orderbookId), ("resolution", Shared.Resolution.toString(query.resolution))]
  switch collectParams(
    base,
    [
      optionalParam("from", query.fromMs, value => ensureUnixMilliseconds("from", value)),
      optionalParam("to", query.toMs, value => ensureUnixMilliseconds("to", value)),
      optionalParam("cursor", query.cursor, value => ensureUnixMilliseconds("cursor", value)),
      optionalParam("limit", query.limit, ensurePageLimit),
    ],
  ) {
  | Error(error) => Error(error)
  | Ok(params) =>
    if query.includeOhlcv {
      params->Array.push(("include_ohlcv", "true"))
    }
    await Http.get(
      client.http,
      ~path="/api/price-history",
      ~query=params,
      ~decode=orderbookPriceHistoryResponse_decode,
    )
  }
}

// Orderbook price history. `fromMs` / `toMs` are unix milliseconds.
let get = async (
  client: Client.t,
  ~orderbookId: string,
  ~resolution: Shared.Resolution.t,
  ~fromMs: option<float>=?,
  ~toMs: option<float>=?,
): result<orderbookPriceHistoryResponse, SdkError.t> =>
  await getWithQuery(client, ~orderbookId, ~query={resolution, fromMs: ?fromMs, toMs: ?toMs, includeOhlcv: false})

// Deposit-token price history from the same REST endpoint.
let getDepositAsset = async (
  client: Client.t,
  ~depositAsset: string,
  ~query: depositPriceHistoryQuery,
): result<depositPriceHistoryResponse, SdkError.t> => {
  let base = [("deposit_asset", depositAsset), ("resolution", Shared.Resolution.toString(query.resolution))]
  switch collectParams(
    base,
    [
      optionalParam("from", query.fromMs, value => ensureUnixMilliseconds("from", value)),
      optionalParam("to", query.toMs, value => ensureUnixMilliseconds("to", value)),
      optionalParam("cursor", query.cursor, value => ensureUnixMilliseconds("cursor", value)),
      optionalParam("limit", query.limit, ensurePageLimit),
    ],
  ) {
  | Error(error) => Error(error)
  | Ok(params) =>
    await Http.get(
      client.http,
      ~path="/api/price-history",
      ~query=params,
      ~decode=depositPriceHistoryResponse_decode,
    )
  }
}

// Snapshot of current prices for every active mint in `global_deposit_tokens`.
let getDepositAssetPricesSnapshot = async (
  client: Client.t,
): result<depositAssetPricesSnapshotResponse, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/deposit-asset-prices-snapshot",
    ~decode=depositAssetPricesSnapshotResponse_decode,
  )

// Simplified midpoint line data for charting (orderbook candles → `lineData`).
let getLineData = async (
  client: Client.t,
  ~orderbookId: string,
  ~resolution: Shared.Resolution.t,
  ~fromMs: option<float>=?,
  ~toMs: option<float>=?,
  ~cursor: option<float>=?,
  ~limit: option<float>=?,
): result<array<lineData>, SdkError.t> => {
  let query = {resolution, fromMs: ?fromMs, toMs: ?toMs, cursor: ?cursor, limit: ?limit, includeOhlcv: false}
  (await getWithQuery(client, ~orderbookId, ~query))->Result.map(response =>
    response.prices->Array.map(lineDataOfOrderbookCandle)
  )
}

// TODO(state): the stateful WS-driven containers `PriceHistoryState` and
// `DepositPriceState` (plus `LatestDepositPrice` and their apply/snapshot/update
// helpers, and the WS wire/tagged-enum types used to feed them) are handled
// separately.
