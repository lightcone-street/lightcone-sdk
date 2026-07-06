// Price-history wire types — the exact JSON the backend sends for orderbook
// OHLCV candles, deposit-token candles, and the deposit-asset price snapshot.
// Prices stay as wire strings; timestamps and cursors are floats (JS numbers,
// unix milliseconds).

// ── Candles ───────────────────────────────────────────────────────────────────
// Orderbook price candle. With `include_ohlcv=false` (default) candles with no
// trades carry only `t` (+ `m`); every OHLCV field is optional.
module OrderbookCandle = {
  @spice
  type t = {
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

  // Candle → line-data point (midpoint, else close, else empty).
  let toLineData = (candle: t): PriceHistory__Model.LineData.t => {
    let value = switch candle.m {
    | Some(midpoint) => midpoint
    | None => candle.c->Option.getOr("")
    }
    {time: candle.t, value}
  }
}

// Deposit-token price candle.
module DepositCandle = {
  @spice
  type t = {
    // Unix milliseconds (candle open time).
    t: float,
    // Unix milliseconds (candle close time).
    tc: float,
    // Close price (raw Binance decimal string).
    c: string,
  }
}

// ── Responses ─────────────────────────────────────────────────────────────────
// Orderbook price-history decimals metadata.
module Decimals = {
  @spice
  type t = {
    price: float,
    volume: float,
  }
}

// Response for `GET /api/price-history?orderbook_id=…`.
module OrderbookResponse = {
  @spice
  type t = {
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    resolution: Shared.Resolution.t,
    @spice.key("include_ohlcv") includeOhlcv: bool,
    prices: array<OrderbookCandle.t>,
    @spice.key("next_cursor") nextCursor?: float,
    @spice.key("has_more") hasMore: bool,
    decimals: Decimals.t,
  }
}

// Response for `GET /api/price-history?deposit_asset=…`.
module DepositResponse = {
  @spice
  type t = {
    @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
    @spice.key("binance_symbol") binanceSymbol: string,
    resolution: Shared.Resolution.t,
    prices: array<DepositCandle.t>,
    @spice.key("next_cursor") nextCursor?: float,
    @spice.key("has_more") hasMore: bool,
  }
}

// Response for `GET /api/deposit-asset-prices-snapshot` — mint → latest price.
module DepositPricesSnapshotResponse = {
  @spice
  type t = {
    prices: dict<string>,
  }
}
