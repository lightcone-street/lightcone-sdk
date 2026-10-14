/* TypeScript file generated from Order__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Order__RawJS from './Order__Raw.res.mjs';

import type {Notification__Model_t as LightconeSdk_Notification__Model_t} from './LightconeSdk.gen.ts';

import type {Order__Model_Limit_t as LightconeSdk_Order__Model_Limit_t} from './LightconeSdk.gen.ts';

import type {Shared_DepositSource_t as LightconeSdk_Shared_DepositSource_t} from './LightconeSdk.gen.ts';

import type {Shared_OrderStatus_t as LightconeSdk_Shared_OrderStatus_t} from './LightconeSdk.gen.ts';

import type {Shared_OrderUpdateType_t as LightconeSdk_Shared_OrderUpdateType_t} from './LightconeSdk.gen.ts';

import type {Shared_Side_t as LightconeSdk_Shared_Side_t} from './LightconeSdk.gen.ts';

import type {Shared_TimeInForce_t as LightconeSdk_Shared_TimeInForce_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

export type FillInfo_t = {
  readonly counterparty: LightconeSdk_Shared_pubkeyStr;
  readonly counterpartyOrderHash: string;
  readonly fillAmount: string;
  readonly price: string;
  readonly isMaker: boolean
};

export type SubmitStatus_t = "accepted" | "partial_fill" | "filled";

export type SubmitResponse_t = {
  readonly orderHash: string;
  readonly status: SubmitStatus_t;
  readonly remaining: string;
  readonly filled: string;
  readonly fills: FillInfo_t[]
};

export type SubmitRequest_t = {
  readonly maker: string;
  readonly nonce: bigint;
  readonly salt: bigint;
  readonly marketPubkey: string;
  readonly baseToken: string;
  readonly quoteToken: string;
  readonly side: number;
  readonly amountIn: bigint;
  readonly amountOut: bigint;
  readonly expiration: bigint;
  readonly signatureHex: string;
  readonly orderbookId: string;
  readonly timeInForce?: LightconeSdk_Shared_TimeInForce_t;
  readonly depositSource?: LightconeSdk_Shared_DepositSource_t
};

export type CancelSuccess_t = { readonly orderHash: string; readonly remaining: string };

export type CancelAllSuccess_t = {
  readonly cancelledOrderHashes: string[];
  readonly count: number;
  readonly userPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly message: string
};

export type CancelBody_t = {
  readonly orderHash: string;
  readonly maker: string;
  readonly signatureHex: string
};

export type CancelAllBody_t = {
  readonly userPubkey: string;
  readonly orderbookId: string;
  readonly signatureHex: string;
  readonly timestamp: number;
  readonly salt: string
};

export type SnapshotCommon_t = {
  readonly orderHash: string;
  readonly marketPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly side: LightconeSdk_Shared_Side_t;
  readonly amountIn: string;
  readonly amountOut: string;
  readonly remaining: string;
  readonly filled: string;
  readonly price: string;
  readonly createdAt: number;
  readonly expiration: number;
  readonly baseMint: LightconeSdk_Shared_pubkeyStr;
  readonly quoteMint: LightconeSdk_Shared_pubkeyStr;
  readonly outcomeIndex: number;
  readonly status: LightconeSdk_Shared_OrderStatus_t
};

export type SnapshotLimit_t = { readonly common: SnapshotCommon_t; readonly txSignature?: string };

export type SnapshotOrder_t = { TAG: "Limit"; _0: SnapshotLimit_t };

export type UserOutcomeBalance_t = {
  readonly outcomeIndex: number;
  readonly conditionalToken: LightconeSdk_Shared_pubkeyStr;
  readonly balance: string;
  readonly balanceIdle: string;
  readonly balanceOnBook: string
};

export type UserDepositAssetBalance_t = { readonly depositAsset: LightconeSdk_Shared_pubkeyStr; readonly outcomes: UserOutcomeBalance_t[] };

export type UserMarketBalance_t = { readonly marketPubkey: LightconeSdk_Shared_pubkeyStr; readonly depositAssets: UserDepositAssetBalance_t[] };

export type GlobalDepositBalance_t = { readonly mint: LightconeSdk_Shared_pubkeyStr; readonly balance: string };

export type UserOrdersResponse_t = {
  readonly userPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly orders: SnapshotOrder_t[];
  readonly marketBalances: UserMarketBalance_t[];
  readonly nextCursor?: string;
  readonly hasMore: boolean
};

export type Role_t = "maker" | "taker";

export type FillStatus_t = "filled" | "cancelled" | "partially_filled";

export type FillEvent_t = {
  readonly fillAmount: string;
  readonly txSignature: string;
  readonly filledAt: number
};

export type UserFill_t = {
  readonly orderHash: string;
  readonly marketPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly side: LightconeSdk_Shared_Side_t;
  readonly role: Role_t;
  readonly price: string;
  readonly size: string;
  readonly filledSize: string;
  readonly remainingSize: string;
  readonly baseMint: LightconeSdk_Shared_pubkeyStr;
  readonly quoteMint: LightconeSdk_Shared_pubkeyStr;
  readonly outcomeIndex: number;
  readonly status: FillStatus_t;
  readonly createdAt: number;
  readonly fills: FillEvent_t[]
};

export type UserFillsResponse_t = {
  readonly orders: UserFill_t[];
  readonly nextCursor?: string;
  readonly hasMore: boolean
};

export type ConditionalBalance_t = {
  readonly outcomeIndex: number;
  readonly conditionalToken: LightconeSdk_Shared_pubkeyStr;
  readonly idle: string;
  readonly onBook: string
};

export type UpdateBalance_t = { readonly outcomes: ConditionalBalance_t[] };

export type WsOrder_t = {
  readonly orderHash: string;
  readonly price: string;
  readonly isMaker: boolean;
  readonly remaining: string;
  readonly filled: string;
  readonly fillAmount: string;
  readonly side: LightconeSdk_Shared_Side_t;
  readonly createdAt: number;
  readonly baseMint: LightconeSdk_Shared_pubkeyStr;
  readonly quoteMint: LightconeSdk_Shared_pubkeyStr;
  readonly outcomeIndex: number;
  readonly status: LightconeSdk_Shared_OrderStatus_t;
  readonly balance?: UpdateBalance_t
};

export type Update_t = {
  readonly marketPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly timestamp: string;
  readonly txSignature?: string;
  readonly updateType: LightconeSdk_Shared_OrderUpdateType_t;
  readonly order: WsOrder_t
};

export type Event_t = { TAG: "Limit"; _0: Update_t };

export type UserSnapshot_t = {
  readonly orders: SnapshotOrder_t[];
  readonly marketBalances: UserMarketBalance_t[];
  readonly globalDeposits: GlobalDepositBalance_t[];
  readonly notifications: LightconeSdk_Notification__Model_t[];
  readonly nonce: number
};

export const SnapshotLimit_toLimit: (_1:SnapshotLimit_t) => LightconeSdk_Order__Model_Limit_t = Order__RawJS.SnapshotLimit.toLimit as any;

export const Update_toLimit: (_1:Update_t) => LightconeSdk_Order__Model_Limit_t = Order__RawJS.Update.toLimit as any;

export const Update: { toLimit: (_1:Update_t) => LightconeSdk_Order__Model_Limit_t } = Order__RawJS.Update as any;

export const SnapshotLimit: { toLimit: (_1:SnapshotLimit_t) => LightconeSdk_Order__Model_Limit_t } = Order__RawJS.SnapshotLimit as any;
