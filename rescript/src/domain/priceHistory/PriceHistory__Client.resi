// Price-history REST client — orderbook OHLCV / candle queries, deposit-token
// price candles, and the deposit-asset price snapshot. The query-param
// validation helpers and the full-query orderbook request are internal.

// Orderbook price history. `fromMs` / `toMs` are unix milliseconds.
let get: (
  Client.t,
  ~orderbookId: string,
  ~resolution: Shared.Resolution.t,
  ~fromMs: float=?,
  ~toMs: float=?,
) => promise<result<PriceHistory__Raw.OrderbookResponse.t, SdkError.t>>

// Deposit-token price history from the same REST endpoint.
let getDepositAsset: (
  Client.t,
  ~depositAsset: string,
  ~query: PriceHistory__Model.DepositQuery.t,
) => promise<result<PriceHistory__Raw.DepositResponse.t, SdkError.t>>

// Snapshot of current prices for every active mint in `global_deposit_tokens`.
let getDepositAssetPricesSnapshot: Client.t => promise<
  result<PriceHistory__Raw.DepositPricesSnapshotResponse.t, SdkError.t>,
>

// Simplified midpoint line data for charting (orderbook candles → line data).
let getLineData: (
  Client.t,
  ~orderbookId: string,
  ~resolution: Shared.Resolution.t,
  ~fromMs: float=?,
  ~toMs: float=?,
  ~cursor: float=?,
  ~limit: float=?,
) => promise<result<array<PriceHistory__Model.LineData.t>, SdkError.t>>
