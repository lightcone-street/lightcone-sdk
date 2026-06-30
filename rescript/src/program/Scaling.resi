// Price/size → raw u64 maker/taker amounts (mirrors rust/src/shared/scaling.rs).
// Public surface: the decimals config, the scaled-amounts result, the error
// enum, and the two pure transforms. The decimal.js %raw helpers are internal.

@genType
type orderbookDecimals = {
  baseDecimals: int,
  quoteDecimals: int,
  priceDecimals: int,
  // Minimum price increment in quote lamports; 0 or 1 disables tick alignment.
  tickSize: float,
}

@genType
type scaledAmounts = {amountIn: bigint, amountOut: bigint}

@genType
type scalingError =
  | NonPositivePrice(string)
  | NonPositiveSize(string)
  | Overflow(string)
  | ZeroAmount

// side: 0 = Bid, 1 = Ask.
let scalePriceSize: (
  ~price: string,
  ~size: string,
  ~side: int,
  ~decimals: orderbookDecimals,
) => result<scaledAmounts, scalingError>

// Snap a price to the nearest valid tick (quote-lamport multiple of tick_size).
let alignPriceToTick: (string, orderbookDecimals) => string
