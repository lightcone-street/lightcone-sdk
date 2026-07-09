/* TypeScript file generated from Scaling.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type OrderbookDecimals_t = {
  readonly baseDecimals: number; 
  readonly quoteDecimals: number; 
  readonly priceDecimals: number; 
  readonly tickSize: number
};

export type Amounts_t = { readonly amountIn: bigint; readonly amountOut: bigint };

export type Error_t = 
    "ZeroAmount"
  | { TAG: "NonPositivePrice"; _0: string }
  | { TAG: "NonPositiveSize"; _0: string }
  | { TAG: "Overflow"; _0: string };
