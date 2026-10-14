// Exact order admission uses the signed ratio and the orderbook's server rules.
// Prices and sizes enter as decimal text. Every arithmetic operation is integral.
module TradingRules = {
  @spice
  type t = {
    @spice.key("base_size_decimals") baseSizeDecimals: int,
    @spice.key("max_price_decimals") maxPriceDecimals: int,
    @spice.key("max_price_significant_figures") maxPriceSignificantFigures: int,
    @spice.key("integer_prices_always_allowed") integerPricesAlwaysAllowed: bool,
    @spice.key("price_quantum") priceQuantum: string,
    @spice.key("price_quantum_raw") priceQuantumRaw: string,
    @spice.key("base_size_quantum") baseSizeQuantum: string,
    @spice.key("base_size_quantum_raw") baseSizeQuantumRaw: string,
  }
}
module OrderbookRules = {
  @spice
  type t = {
    @spice.key("orderbook_id") orderbookId: string,
    @spice.key("base_decimals") baseDecimals: int,
    @spice.key("quote_decimals") quoteDecimals: int,
    @spice.key("price_decimals") priceDecimals: int,
    @spice.key("trading_rules") tradingRules: TradingRules.t,
  }
}
module Amounts = {
  type t = {
    amountIn: bigint,
    amountOut: bigint,
    priceRaw: bigint,
    baseAtoms: bigint,
    quoteAtoms: bigint,
  }
}
module Error = {
  type t =
    | InvalidDecimal({input: string, reason: string})
    | NonPositivePrice(string)
    | NonPositiveSize(string)
    | InvalidSizeDecimals
    | InvalidPriceDecimals
    | InvalidPriceSignificantFigures
    | PriceNotExactlyRepresentable
    | PriceOutOfRange
    | OrderFieldOutOfRange(string)
    | OrderbookMismatch({expected: string, actual: string})
}
let priceScale = 1000000n
let i64Max = 9223372036854775807n
let nonceMax = 4294967295n
let toMessage = (error: Error.t): string =>
  switch error {
  | InvalidDecimal({input, reason}) => `invalid decimal '${input}': ${reason}`
  | NonPositivePrice(value) => `price must be positive, got ${value}`
  | NonPositiveSize(value) => `size must be positive, got ${value}`
  | InvalidSizeDecimals => "INVALID_SIZE_DECIMALS"
  | InvalidPriceDecimals => "INVALID_PRICE_DECIMALS"
  | InvalidPriceSignificantFigures => "INVALID_PRICE_SIGNIFICANT_FIGURES"
  | PriceNotExactlyRepresentable => "PRICE_NOT_EXACTLY_REPRESENTABLE"
  | PriceOutOfRange => "PRICE_OUT_OF_RANGE"
  | OrderFieldOutOfRange(field) => `ORDER_FIELD_OUT_OF_RANGE: ${field}`
  | OrderbookMismatch({expected, actual}) =>
    `trading rules for '${actual}' cannot be used for '${expected}'`
  }
let exactScaledInteger = (input, decimals) =>
  switch ExactDecimal.scaledInteger(input, decimals) {
  | Ok(value) => Ok(value)
  | Error(reason) => Error(Error.InvalidDecimal({input, reason}))
  }
let validateForOrderbook = (rules: OrderbookRules.t, expected): result<unit, Error.t> =>
  rules.orderbookId == expected
    ? Ok()
    : Error(OrderbookMismatch({expected, actual: rules.orderbookId}))

// Reject malformed rules before caching them. Quantums are arbitrary-width text.
let isInteger: int => bool = %raw(`Number.isInteger`)
let validateRules = (rules: OrderbookRules.t): result<unit, Error.t> => {
  let counts = [
    rules.baseDecimals,
    rules.quoteDecimals,
    rules.priceDecimals,
    rules.tradingRules.baseSizeDecimals,
    rules.tradingRules.maxPriceDecimals,
    rules.tradingRules.maxPriceSignificantFigures,
  ]
  if counts->Array.some(value => !isInteger(value) || value < 0 || value > 255) {
    Error(OrderFieldOutOfRange("trading rule decimals"))
  } else if (
    !RegExp.test(/^[0-9]+$/, rules.tradingRules.priceQuantumRaw) ||
    BigInt.fromString(rules.tradingRules.priceQuantumRaw)->Option.mapOr(true, value => value <= 0n)
  ) {
    Error(InvalidPriceDecimals)
  } else if (
    !RegExp.test(/^[0-9]+$/, rules.tradingRules.baseSizeQuantumRaw) ||
    BigInt.fromString(rules.tradingRules.baseSizeQuantumRaw)->Option.mapOr(true, value =>
      value <= 0n
    )
  ) {
    Error(InvalidSizeDecimals)
  } else {
    Ok()
  }
}
let checkedAmount = (value, field): result<bigint, Error.t> =>
  value <= 0n || value > i64Max ? Error(OrderFieldOutOfRange(field)) : Ok(value)
let divisibleBy = (value, quantum) =>
  switch BigInt.fromString(quantum) {
  | Some(divisor) if divisor > 0n => BigInt.mod(value, divisor) == 0n
  | _ => false
  }
let rec powerTen = n => n == 0 ? 1n : BigInt.mul(10n, powerTen(n - 1))
let validatePrice = (price: bigint, rules: OrderbookRules.t): result<bigint, Error.t> => {
  if price <= 0n || price > i64Max {
    Error(PriceOutOfRange)
  } else {
    let scale = powerTen(rules.priceDecimals)
    if BigInt.mod(price, scale) == 0n {
      Ok(price)
    } else if !divisibleBy(price, rules.tradingRules.priceQuantumRaw) {
      Error(InvalidPriceDecimals)
    } else if (
      BigInt.toString(price)->String.replaceRegExp(/0+$/, "")->String.length >
        rules.tradingRules.maxPriceSignificantFigures
    ) {
      Error(InvalidPriceSignificantFigures)
    } else {
      Ok(price)
    }
  }
}
let validateBase = (base: bigint, rules: OrderbookRules.t): result<bigint, Error.t> =>
  if base == 0n {
    Error(NonPositiveSize("0"))
  } else if !divisibleBy(base, rules.tradingRules.baseSizeQuantumRaw) {
    Error(InvalidSizeDecimals)
  } else {
    checkedAmount(base, "base amount")
  }
let amounts = (~baseAtoms, ~quoteAtoms, ~priceRaw, ~side): result<Amounts.t, Error.t> =>
  switch side {
  | 0 => Ok({amountIn: quoteAtoms, amountOut: baseAtoms, priceRaw, baseAtoms, quoteAtoms})
  | 1 => Ok({amountIn: baseAtoms, amountOut: quoteAtoms, priceRaw, baseAtoms, quoteAtoms})
  | _ => Error(OrderFieldOutOfRange("side"))
  }
let scalePriceSize = (~price, ~size, ~side, ~rules: OrderbookRules.t): result<
  Amounts.t,
  Error.t,
> => {
  validateRules(rules)->Result.flatMap(() =>
    exactScaledInteger(price, rules.priceDecimals)->Result.flatMap(priceRaw =>
      exactScaledInteger(size, rules.baseDecimals)->Result.flatMap(
        baseAtoms =>
          if priceRaw == 0n {
            Error(Error.NonPositivePrice(price))
          } else if baseAtoms == 0n {
            Error(Error.NonPositiveSize(size))
          } else {
            validatePrice(priceRaw, rules)->Result.flatMap(
              priceRaw =>
                validateBase(baseAtoms, rules)->Result.flatMap(
                  baseAtoms => {
                    let numerator = BigInt.mul(priceRaw, baseAtoms)
                    if BigInt.mod(numerator, priceScale) != 0n {
                      Error(Error.PriceNotExactlyRepresentable)
                    } else {
                      checkedAmount(
                        BigInt.div(numerator, priceScale),
                        "quote amount",
                      )->Result.flatMap(
                        quoteAtoms => amounts(~baseAtoms, ~quoteAtoms, ~priceRaw, ~side),
                      )
                    }
                  },
                ),
            )
          },
      )
    )
  )
}
// Direct signed requests undergo the same admission as human price/size inputs.
let validateRawAmounts = (~amountIn, ~amountOut, ~side, ~rules: OrderbookRules.t): result<
  Amounts.t,
  Error.t,
> => {
  validateRules(rules)->Result.flatMap(() =>
    checkedAmount(amountIn, "amount_in")->Result.flatMap(_ =>
      checkedAmount(amountOut, "amount_out")->Result.flatMap(
        _ => {
          let (baseAtoms, quoteAtoms) = side == 0 ? (amountOut, amountIn) : (amountIn, amountOut)
          validateBase(baseAtoms, rules)->Result.flatMap(
            _ => {
              let numerator = BigInt.mul(quoteAtoms, priceScale)
              if BigInt.mod(numerator, baseAtoms) != 0n {
                Error(Error.PriceNotExactlyRepresentable)
              } else {
                validatePrice(BigInt.div(numerator, baseAtoms), rules)->Result.flatMap(
                  priceRaw => amounts(~baseAtoms, ~quoteAtoms, ~priceRaw, ~side),
                )
              }
            },
          )
        },
      )
    )
  )
}
let validateSignedFields = (~amountIn, ~amountOut, ~salt, ~nonce): result<unit, Error.t> => {
  checkedAmount(amountIn, "amount_in")->Result.flatMap(_ =>
    checkedAmount(amountOut, "amount_out")->Result.flatMap(_ =>
      if salt < 0n || salt > i64Max {
        Error(Error.OrderFieldOutOfRange("salt"))
      } else if nonce < 0n || nonce > nonceMax {
        Error(Error.OrderFieldOutOfRange("nonce"))
      } else {
        Ok()
      }
    )
  )
}
