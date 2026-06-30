/* TypeScript file generated from Accounts.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

export type MarketStatus_t = 
    "Pending"
  | "Active"
  | "Resolved"
  | "Cancelled";

export type PendingRoleKind_t = "None" | "Authority" | "Manager" | "Operator";

export type exchange = {
  readonly authority: Shared_pubkeyStr; 
  readonly operator: Shared_pubkeyStr; 
  readonly manager: Shared_pubkeyStr; 
  readonly marketCount: bigint; 
  readonly paused: boolean; 
  readonly bump: number; 
  readonly depositTokenCount: number; 
  readonly feeReceiver: Shared_pubkeyStr; 
  readonly pendingRole: Shared_pubkeyStr; 
  readonly pendingRoleKind: PendingRoleKind_t
};

export type market = {
  readonly marketId: bigint; 
  readonly numOutcomes: number; 
  readonly status: MarketStatus_t; 
  readonly bump: number; 
  readonly makerFeeBps: number; 
  readonly takerFeeBps: number; 
  readonly oracle: Shared_pubkeyStr; 
  readonly questionId: string; 
  readonly conditionId: string; 
  readonly payoutNumerators: number[]; 
  readonly payoutDenominator: number
};

export type orderbook = {
  readonly market: Shared_pubkeyStr; 
  readonly mintA: Shared_pubkeyStr; 
  readonly mintB: Shared_pubkeyStr; 
  readonly lookupTable: Shared_pubkeyStr; 
  readonly baseIndex: number; 
  readonly bump: number
};

export type position = {
  readonly owner: Shared_pubkeyStr; 
  readonly market: Shared_pubkeyStr; 
  readonly bump: number
};

export type userNonce = { readonly nonce: bigint };
