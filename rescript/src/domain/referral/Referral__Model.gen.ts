/* TypeScript file generated from Referral__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Referral__ModelJS from './Referral__Model.res.mjs';

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

export type RedeemErrorCode_t =
    "InvalidCode"
  | "CodeFullyRedeemed"
  | "AlreadyBeta"
  | "SelfReferral"
  | "AlreadyRedeemedByUser"
  | "RateLimited"
  | "InternalError"
  | { TAG: "Unknown"; _0: string };

export const RedeemErrorCode_fromWire: (_1:string) => RedeemErrorCode_t = Referral__ModelJS.RedeemErrorCode.fromWire as any;

export const RedeemErrorCode_toWire: (_1:RedeemErrorCode_t) => string = Referral__ModelJS.RedeemErrorCode.toWire as any;

export const RedeemErrorCode: { fromWire: (_1:string) => RedeemErrorCode_t; toWire: (_1:RedeemErrorCode_t) => string } = Referral__ModelJS.RedeemErrorCode as any;
