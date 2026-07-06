/* TypeScript file generated from Referral__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type Code_t = {
  readonly code: string; 
  readonly maxUses: number; 
  readonly useCount: number
};

export type StatusResponse_t = {
  readonly isBeta: boolean; 
  readonly source: (undefined | string); 
  readonly referralCodes: Code_t[]
};

export type RedeemResponse_t = { readonly success: boolean; readonly isBeta: boolean };
