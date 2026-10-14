// Price-history domain types — the chart line-data point and the REST query
// option records. Prices stay as wire strings (no precision loss,
// gentype-clean); timestamps and cursors are floats (JS numbers, unix
// milliseconds). NOTE: the wire query params `from` / `to` are spelled
// `fromMs` / `toMs` here — `to` is a reserved ReScript keyword, and the
// backend requires these to be unix milliseconds.

// A single data point on a price chart (simplified from the full candle).
module LineData = {
  type t = {
    // Unix milliseconds.
    time: float,
    // Midpoint value as a decimal string.
    value: string,
  }
}

// Query options for orderbook price-history requests. `fromMs` / `toMs` / `cursor`
// are unix milliseconds; `limit` is 1..=1000.
module OrderbookQuery = {
  type t = {
    resolution: Shared.Resolution.t,
    fromMs?: float,
    toMs?: float,
    cursor?: float,
    limit?: float,
    includeOhlcv: bool,
  }
}

// Query options for deposit-token price-history requests.
module DepositQuery = {
  type t = {
    resolution: Shared.Resolution.t,
    fromMs?: float,
    toMs?: float,
    cursor?: float,
    limit?: float,
  }
}
