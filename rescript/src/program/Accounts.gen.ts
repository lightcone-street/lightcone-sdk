/* TypeScript file generated from Accounts.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

export type MarketStatus_t =
    "Pending"
  | "Active"
  | "Resolved"
  | "Cancelled";

export type PendingRoleKind_t = "None" | "Authority" | "Manager" | "Operator";

export type Exchange_t = {
  readonly authority: LightconeSdk_Shared_pubkeyStr;
  readonly operator: LightconeSdk_Shared_pubkeyStr;
  readonly manager: LightconeSdk_Shared_pubkeyStr;
  readonly marketCount: bigint;
  readonly paused: boolean;
  readonly bump: number;
  readonly depositTokenCount: number;
  readonly feeReceiver: LightconeSdk_Shared_pubkeyStr;
  readonly pendingRole: LightconeSdk_Shared_pubkeyStr;
  readonly pendingRoleKind: PendingRoleKind_t
};

export type Market_t = {
  readonly marketId: bigint;
  readonly numOutcomes: number;
  readonly status: MarketStatus_t;
  readonly bump: number;
  readonly makerFeeBps: number;
  readonly takerFeeBps: number;
  readonly oracle: LightconeSdk_Shared_pubkeyStr;
  readonly questionId: string;
  readonly conditionId: string;
  readonly payoutNumerators: number[];
  readonly payoutDenominator: number;
  readonly depositMintCount: number
};

export type Orderbook_t = {
  readonly market: LightconeSdk_Shared_pubkeyStr;
  readonly mintA: LightconeSdk_Shared_pubkeyStr;
  readonly mintB: LightconeSdk_Shared_pubkeyStr;
  readonly depositMintA: LightconeSdk_Shared_pubkeyStr;
  readonly depositMintB: LightconeSdk_Shared_pubkeyStr;
  readonly outcomeIndex: number;
  readonly baseIndex: number;
  readonly bump: number
};

export type Position_t = {
  readonly owner: LightconeSdk_Shared_pubkeyStr;
  readonly market: LightconeSdk_Shared_pubkeyStr;
  readonly bump: number
};

export type UserNonce_t = { readonly nonce: bigint };

export type OrderStatus_t = {
  readonly remaining: bigint;
  readonly baseRemaining: bigint;
  readonly isCancelled: boolean
};

export type GlobalDepositToken_t = {
  readonly mint: LightconeSdk_Shared_pubkeyStr;
  readonly bump: number;
  readonly index: number;
  readonly active: boolean
};
