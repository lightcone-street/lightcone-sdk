// Deterministic full-precision rules for offline signing and codec tests.
let make = (
  ~orderbookId="ob_1",
  ~baseDecimals=6,
  ~quoteDecimals=6,
  (),
): Scaling.OrderbookRules.t => {
  orderbookId,
  baseDecimals,
  quoteDecimals,
  priceDecimals: 6 + quoteDecimals - baseDecimals,
  tradingRules: {
    baseSizeDecimals: baseDecimals,
    maxPriceDecimals: 6 + quoteDecimals - baseDecimals,
    maxPriceSignificantFigures: 9,
    integerPricesAlwaysAllowed: true,
    priceQuantum: "0.000001",
    priceQuantumRaw: "1",
    baseSizeQuantum: "0.000001",
    baseSizeQuantumRaw: "1",
  },
}
