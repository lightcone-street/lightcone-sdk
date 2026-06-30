/* TypeScript file generated from Notification.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

export type MarketResolution_Kind_t = "single_winner" | "scalar";

export type MarketResolution_payout = { readonly outcomeIndex: number; readonly payoutNumerator: number };

export type MarketResolution_t = {
  readonly kind: MarketResolution_Kind_t; 
  readonly payoutDenominator: number; 
  readonly payouts: MarketResolution_payout[]; 
  readonly singleWinningOutcome?: number
};

export type marketResolvedData = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly marketSlug?: string; 
  readonly marketName?: string; 
  readonly resolution?: MarketResolution_t
};

export type orderFilledData = {
  readonly orderHash: string; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly side: string; 
  readonly price: string; 
  readonly filled: string; 
  readonly remaining: string; 
  readonly marketSlug?: string; 
  readonly marketName?: string; 
  readonly outcomeName?: string; 
  readonly outcomeNameLong?: string; 
  readonly outcomeIconUrlLow?: string; 
  readonly outcomeIconUrlMedium?: string; 
  readonly outcomeIconUrlHigh?: string
};

export type marketData = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly marketSlug?: string; 
  readonly marketName?: string
};

export type notificationKind = 
    "Global"
  | { TAG: "MarketResolved"; _0: marketResolvedData }
  | { TAG: "OrderFilled"; _0: orderFilledData }
  | { TAG: "NewMarket"; _0: marketData }
  | { TAG: "RulesClarified"; _0: marketData };

export type notification = {
  readonly id: string; 
  readonly kind: notificationKind; 
  readonly title: string; 
  readonly message: string; 
  readonly expiresAt?: string; 
  readonly createdAt: string
};
