// Price/size → raw u64 maker/taker amounts.
// Public surface: the decimals config, the scaled-amounts result, the error
// enum, and the two pure transforms. The decimal.js %raw helpers are internal.

// The orderbook pair's decimals config.
module OrderbookDecimals: {
  @genType
  type t = {
    baseDecimals: int,
    quoteDecimals: int,
    priceDecimals: int,
    // Minimum price increment in quote lamports; 0 or 1 disables tick alignment.
    tickSize: float,
  }
}

// The scaled maker/taker amounts.
module Amounts: {
  @genType
  type t = {amountIn: bigint, amountOut: bigint}
}

module Error: {
  @genType
  type t =
    | NonPositivePrice(string)
    | NonPositiveSize(string)
    | Overflow(string)
    | ZeroAmount
}

// side: 0 = Bid, 1 = Ask.
let scalePriceSize: (
  ~price: string,
  ~size: string,
  ~side: int,
  ~decimals: OrderbookDecimals.t,
) => result<Amounts.t, Error.t>

// Snap a price to the nearest valid tick (quote-lamport multiple of tick_size).
let alignPriceToTick: (string, OrderbookDecimals.t) => string
