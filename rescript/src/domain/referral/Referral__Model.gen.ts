/* TypeScript file generated from Referral__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type Code_t = {
  readonly code: string; 
  readonly maxUses: number; 
  readonly useCount: number
};

export type Status_t = {
  readonly isBeta: boolean; 
  readonly source?: string; 
  readonly referralCodes: Code_t[]
};

export type RedeemResult_t = { readonly success: boolean; readonly isBeta: boolean };
