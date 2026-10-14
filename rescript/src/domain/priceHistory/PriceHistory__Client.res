// Price-history REST client — orderbook OHLCV / candle queries, deposit-token
// price candles, and the deposit-asset price snapshot. Functions take a
// `Client.t` and return `promise<result<_, SdkError.t>>`; responses decode
// straight into the `PriceHistory__Raw` wire types.

// ── Query-param validation ────────────────────────────────────────────────────
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
  ~query: PriceHistory__Model.OrderbookQuery.t,
): result<PriceHistory__Raw.OrderbookResponse.t, SdkError.t> => {
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
      ~decode=PriceHistory__Raw.OrderbookResponse.t_decode,
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
): result<PriceHistory__Raw.OrderbookResponse.t, SdkError.t> =>
  await getWithQuery(client, ~orderbookId, ~query={resolution, fromMs: ?fromMs, toMs: ?toMs, includeOhlcv: false})

// Deposit-token price history from the same REST endpoint.
let getDepositAsset = async (
  client: Client.t,
  ~depositAsset: string,
  ~query: PriceHistory__Model.DepositQuery.t,
): result<PriceHistory__Raw.DepositResponse.t, SdkError.t> => {
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
      ~decode=PriceHistory__Raw.DepositResponse.t_decode,
    )
  }
}

// Snapshot of current prices for every active mint in `global_deposit_tokens`.
let getDepositAssetPricesSnapshot = async (
  client: Client.t,
): result<PriceHistory__Raw.DepositPricesSnapshotResponse.t, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/deposit-asset-prices-snapshot",
    ~decode=PriceHistory__Raw.DepositPricesSnapshotResponse.t_decode,
  )

// Simplified midpoint line data for charting (orderbook candles → line data).
let getLineData = async (
  client: Client.t,
  ~orderbookId: string,
  ~resolution: Shared.Resolution.t,
  ~fromMs: option<float>=?,
  ~toMs: option<float>=?,
  ~cursor: option<float>=?,
  ~limit: option<float>=?,
): result<array<PriceHistory__Model.LineData.t>, SdkError.t> => {
  let query: PriceHistory__Model.OrderbookQuery.t = {
    resolution,
    fromMs: ?fromMs,
    toMs: ?toMs,
    cursor: ?cursor,
    limit: ?limit,
    includeOhlcv: false,
  }
  (await getWithQuery(client, ~orderbookId, ~query))->Result.map(response =>
    response.prices->Array.map(PriceHistory__Raw.OrderbookCandle.toLineData)
  )
}
