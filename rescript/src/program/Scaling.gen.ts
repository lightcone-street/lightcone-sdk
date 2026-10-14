/* TypeScript file generated from Scaling.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type TradingRules_t = {
  readonly baseSizeDecimals: number;
  readonly maxPriceDecimals: number;
  readonly maxPriceSignificantFigures: number;
  readonly integerPricesAlwaysAllowed: boolean;
  readonly priceQuantum: string;
  readonly priceQuantumRaw: string;
  readonly baseSizeQuantum: string;
  readonly baseSizeQuantumRaw: string
};

export type OrderbookRules_t = {
  readonly orderbookId: string;
  readonly baseDecimals: number;
  readonly quoteDecimals: number;
  readonly priceDecimals: number;
  readonly tradingRules: TradingRules_t
};

export type Amounts_t = {
  readonly amountIn: bigint;
  readonly amountOut: bigint;
  readonly priceRaw: bigint;
  readonly baseAtoms: bigint;
  readonly quoteAtoms: bigint
};

export type Error_t =
    "InvalidSizeDecimals"
  | "InvalidPriceDecimals"
  | "InvalidPriceSignificantFigures"
  | "PriceNotExactlyRepresentable"
  | "PriceOutOfRange"
  | { TAG: "InvalidDecimal"; readonly input: string; readonly reason: string }
  | { TAG: "NonPositivePrice"; _0: string }
  | { TAG: "NonPositiveSize"; _0: string }
  | { TAG: "OrderFieldOutOfRange"; _0: string }
  | { TAG: "OrderbookMismatch"; readonly expected: string; readonly actual: string };
