/* TypeScript file generated from Referral.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type referralCodeInfo = {
  readonly code: string; 
  readonly maxUses: number; 
  readonly useCount: number
};

export type referralStatus = {
  readonly isBeta: boolean; 
  readonly source?: string; 
  readonly referralCodes: referralCodeInfo[]
};

export type redeemResult = { readonly success: boolean; readonly isBeta: boolean };
