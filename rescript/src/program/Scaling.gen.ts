/* TypeScript file generated from Scaling.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type orderbookDecimals = {
  readonly baseDecimals: number; 
  readonly quoteDecimals: number; 
  readonly priceDecimals: number; 
  readonly tickSize: number
};

export type scaledAmounts = { readonly amountIn: bigint; readonly amountOut: bigint };

export type scalingError = 
    "ZeroAmount"
  | { TAG: "NonPositivePrice"; _0: string }
  | { TAG: "NonPositiveSize"; _0: string }
  | { TAG: "Overflow"; _0: string };
