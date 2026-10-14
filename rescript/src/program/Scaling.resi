// Exact decimal and signed-ratio admission against server-owned rules.
module TradingRules: {
  @genType @spice @genType
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
module OrderbookRules: {
  @genType @spice @genType
  type t = {
    @spice.key("orderbook_id") orderbookId: string,
    @spice.key("base_decimals") baseDecimals: int,
    @spice.key("quote_decimals") quoteDecimals: int,
    @spice.key("price_decimals") priceDecimals: int,
    @spice.key("trading_rules") tradingRules: TradingRules.t,
  }
}
module Amounts: {
  @genType
  type t = {
    amountIn: bigint,
    amountOut: bigint,
    priceRaw: bigint,
    baseAtoms: bigint,
    quoteAtoms: bigint,
  }
}
module Error: {
  @genType
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
let priceScale: bigint
let i64Max: bigint
let nonceMax: bigint
let toMessage: Error.t => string
let exactScaledInteger: (string, int) => result<bigint, Error.t>
let validateForOrderbook: (OrderbookRules.t, string) => result<unit, Error.t>
let validateRules: OrderbookRules.t => result<unit, Error.t>
let scalePriceSize: (
  ~price: string,
  ~size: string,
  ~side: int,
  ~rules: OrderbookRules.t,
) => result<Amounts.t, Error.t>
let validateRawAmounts: (
  ~amountIn: bigint,
  ~amountOut: bigint,
  ~side: int,
  ~rules: OrderbookRules.t,
) => result<Amounts.t, Error.t>
let validateSignedFields: (
  ~amountIn: bigint,
  ~amountOut: bigint,
  ~salt: bigint,
  ~nonce: bigint,
) => result<unit, Error.t>
