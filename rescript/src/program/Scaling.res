// Price/size → raw u64 maker/taker amounts. Pure decimal math (decimal.js):
//   base_lamports  = trunc(size, base_decimals) * 10^base_decimals
//   quote_lamports = trunc(price * size * 10^quote_decimals)
//   BID: amount_in = quote, amount_out = base ; ASK: swapped.

// The orderbook pair's decimals config.
module OrderbookDecimals = {
  type t = {
    baseDecimals: int,
    quoteDecimals: int,
    priceDecimals: int,
    // Minimum price increment in quote lamports; 0 or 1 disables tick alignment.
    tickSize: float,
  }
}

// The scaled maker/taker amounts.
module Amounts = {
  type t = {amountIn: bigint, amountOut: bigint}
}

module Error = {
  type t =
    | NonPositivePrice(string)
    | NonPositiveSize(string)
    | Overflow(string)
    | ZeroAmount
}

// A whole-valued decimal → bigint (via its integer string).
let decimalToBigInt: Decimal.t => bigint = %raw(`(decimal) => BigInt(decimal.toFixed(0))`)
let isBigIntZero: bigint => bool = %raw(`(value) => value === 0n`)
let fitsU64: bigint => bool = %raw(`(value) => value >= 0n && value <= 18446744073709551615n`)

let trunc = (decimal, places) => Decimal.toDecimalPlaces(decimal, places, Decimal.roundDown)

// side: 0 = Bid, 1 = Ask.
let scalePriceSize = (
  ~price: string,
  ~size: string,
  ~side: int,
  ~decimals: OrderbookDecimals.t,
): result<Amounts.t, Error.t> => {
  let zero = Decimal.fromInt(0)
  let priceDecimal = Decimal.fromString(price)
  let sizeDecimal = Decimal.fromString(size)

  if !Decimal.gt(priceDecimal, zero) {
    Error(Error.NonPositivePrice(price))
  } else if !Decimal.gt(sizeDecimal, zero) {
    Error(Error.NonPositiveSize(size))
  } else {
    let ten = Decimal.fromInt(10)
    let baseMultiplier = Decimal.powInt(ten, decimals.baseDecimals)
    let quoteMultiplier = Decimal.powInt(ten, decimals.quoteDecimals)

    // Truncate size to the base token's precision (drops f64 noise / sub-lamport dust).
    let sizeTruncated = trunc(sizeDecimal, decimals.baseDecimals)
    let baseLamports = trunc(Decimal.times(sizeTruncated, baseMultiplier), 0)
    let quoteLamports = trunc(
      Decimal.times(Decimal.times(priceDecimal, sizeTruncated), quoteMultiplier),
      0,
    )

    let baseAmount = decimalToBigInt(baseLamports)
    let quoteAmount = decimalToBigInt(quoteLamports)

    if !fitsU64(baseAmount) {
      Error(Error.Overflow("base_lamports does not fit in u64"))
    } else if !fitsU64(quoteAmount) {
      Error(Error.Overflow("quote_lamports does not fit in u64"))
    } else if isBigIntZero(baseAmount) || isBigIntZero(quoteAmount) {
      Error(Error.ZeroAmount)
    } else {
      switch side {
      | 0 => Ok({amountIn: quoteAmount, amountOut: baseAmount})
      | _ => Ok({amountIn: baseAmount, amountOut: quoteAmount})
      }
    }
  }
}

// Snap a price to the nearest valid tick (quote-lamport multiple of tick_size).
let alignPriceToTick = (price: string, decimals: OrderbookDecimals.t): string => {
  if decimals.tickSize <= 1.0 {
    price
  } else {
    let priceDecimal = Decimal.fromString(price)
    let quoteMultiplier = Decimal.powInt(Decimal.fromInt(10), decimals.quoteDecimals)
    let tick = Decimal.fromFloat(decimals.tickSize)
    let lamports = trunc(Decimal.times(priceDecimal, quoteMultiplier), 0)
    let alignedLamports = Decimal.times(trunc(Decimal.div(lamports, tick), 0), tick)
    Decimal.toString(Decimal.div(alignedLamports, quoteMultiplier))
  }
}
