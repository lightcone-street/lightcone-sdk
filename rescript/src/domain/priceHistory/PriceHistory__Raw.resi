// Price-history wire types — orderbook OHLCV candles, deposit-token candles,
// and the deposit-asset price snapshot. `@spice` is kept so the codecs are
// public (the REST client and the WS message layer decode across the file
// boundary); `@genType` exports the TS shapes.

// ── Candles ───────────────────────────────────────────────────────────────────
// Orderbook price candle. With `include_ohlcv=false` (default) candles with no
// trades carry only `t` (+ `m`); every OHLCV field is optional.
module OrderbookCandle: {
  @genType @spice
  type t = {
    // Unix milliseconds (candle start).
    t: float,
    // Midpoint, open, high, low, close, volume, best bid, best ask.
    m?: string,
    o?: string,
    h?: string,
    l?: string,
    c?: string,
    v?: string,
    bb?: string,
    ba?: string,
  }

  // Candle → line-data point (midpoint, else close, else empty). Reused by the
  // WS `PriceHistory__State` container.
  let toLineData: t => PriceHistory__Model.LineData.t
}

// Deposit-token price candle.
module DepositCandle: {
  @genType @spice
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
module Decimals: {
  @genType @spice
  type t = {
    price: float,
    volume: float,
  }
}

// Response for `GET /api/price-history?orderbook_id=…`.
module OrderbookResponse: {
  @genType @spice
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
module DepositResponse: {
  @genType @spice
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
module DepositPricesSnapshotResponse: {
  @genType @spice
  type t = {
    prices: dict<string>,
  }
}
