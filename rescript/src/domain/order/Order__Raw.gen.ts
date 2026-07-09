/* TypeScript file generated from Order__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Order__RawJS from './Order__Raw.res.mjs';

import type {DepositSource_t as Shared_DepositSource_t} from '../../../src/Shared.gen.ts';

import type {Limit_t as Order__Model_Limit_t} from './Order__Model.gen.ts';

import type {OrderStatus_t as Shared_OrderStatus_t} from '../../../src/Shared.gen.ts';

import type {OrderUpdateType_t as Shared_OrderUpdateType_t} from '../../../src/Shared.gen.ts';

import type {Side_t as Shared_Side_t} from '../../../src/Shared.gen.ts';

import type {TimeInForce_t as Shared_TimeInForce_t} from '../../../src/Shared.gen.ts';

import type {TriggerResultStatus_t as Shared_TriggerResultStatus_t} from '../../../src/Shared.gen.ts';

import type {TriggerStatus_t as Shared_TriggerStatus_t} from '../../../src/Shared.gen.ts';

import type {TriggerType_t as Shared_TriggerType_t} from '../../../src/Shared.gen.ts';

import type {TriggerUpdateType_t as Shared_TriggerUpdateType_t} from '../../../src/Shared.gen.ts';

import type {Trigger_t as Order__Model_Trigger_t} from './Order__Model.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../../src/Shared.gen.ts';

import type {t as Notification__Model_t} from '../../../src/domain/notification/Notification__Model.gen.ts';

export type FillInfo_t = {
  readonly counterparty: Shared_pubkeyStr; 
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
  readonly timeInForce?: Shared_TimeInForce_t; 
  readonly depositSource?: Shared_DepositSource_t; 
  readonly triggerPrice?: number; 
  readonly triggerType?: Shared_TriggerType_t
};

export type TriggerResponse_t = { readonly triggerOrderId: string; readonly orderHash: string };

export type CancelSuccess_t = { readonly orderHash: string; readonly remaining: string };

export type CancelAllSuccess_t = {
  readonly cancelledOrderHashes: string[]; 
  readonly count: number; 
  readonly userPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly message: string
};

export type CancelTriggerSuccess_t = { readonly triggerOrderId: string };

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

export type CancelTriggerBody_t = {
  readonly triggerOrderId: string; 
  readonly maker: string; 
  readonly signatureHex: string
};

export type SnapshotCommon_t = {
  readonly orderHash: string; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly side: Shared_Side_t; 
  readonly amountIn: string; 
  readonly amountOut: string; 
  readonly remaining: string; 
  readonly filled: string; 
  readonly price: string; 
  readonly createdAt: number; 
  readonly expiration: number; 
  readonly baseMint: Shared_pubkeyStr; 
  readonly quoteMint: Shared_pubkeyStr; 
  readonly outcomeIndex: number; 
  readonly status: Shared_OrderStatus_t
};

export type SnapshotLimit_t = { readonly common: SnapshotCommon_t; readonly txSignature?: string };

export type SnapshotTrigger_t = {
  readonly common: SnapshotCommon_t; 
  readonly triggerOrderId: string; 
  readonly triggerPrice: string; 
  readonly triggerType: Shared_TriggerType_t; 
  readonly timeInForce?: Shared_TimeInForce_t
};

export type SnapshotOrder_t = 
    { TAG: "Limit"; _0: SnapshotLimit_t }
  | { TAG: "Trigger"; _0: SnapshotTrigger_t };

export type UserOutcomeBalance_t = {
  readonly outcomeIndex: number; 
  readonly conditionalToken: Shared_pubkeyStr; 
  readonly balance: string; 
  readonly balanceIdle: string; 
  readonly balanceOnBook: string
};

export type UserDepositAssetBalance_t = { readonly depositAsset: Shared_pubkeyStr; readonly outcomes: UserOutcomeBalance_t[] };

export type UserMarketBalance_t = { readonly marketPubkey: Shared_pubkeyStr; readonly depositAssets: UserDepositAssetBalance_t[] };

export type GlobalDepositBalance_t = { readonly mint: Shared_pubkeyStr; readonly balance: string };

export type UserOrdersResponse_t = {
  readonly userPubkey: Shared_pubkeyStr; 
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
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly side: Shared_Side_t; 
  readonly role: Role_t; 
  readonly price: string; 
  readonly size: string; 
  readonly filledSize: string; 
  readonly remainingSize: string; 
  readonly baseMint: Shared_pubkeyStr; 
  readonly quoteMint: Shared_pubkeyStr; 
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
  readonly conditionalToken: Shared_pubkeyStr; 
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
  readonly side: Shared_Side_t; 
  readonly createdAt: number; 
  readonly baseMint: Shared_pubkeyStr; 
  readonly quoteMint: Shared_pubkeyStr; 
  readonly outcomeIndex: number; 
  readonly status: Shared_OrderStatus_t; 
  readonly balance?: UpdateBalance_t
};

export type Update_t = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly timestamp: string; 
  readonly txSignature?: string; 
  readonly updateType: Shared_OrderUpdateType_t; 
  readonly order: WsOrder_t
};

export type TriggerUpdate_t = {
  readonly triggerOrderId: string; 
  readonly userPubkey: Shared_pubkeyStr; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly triggerPrice: string; 
  readonly triggerAbove: boolean; 
  readonly status: Shared_TriggerStatus_t; 
  readonly updateType: Shared_TriggerUpdateType_t; 
  readonly orderHash: string; 
  readonly side: Shared_Side_t; 
  readonly resultStatus?: Shared_TriggerResultStatus_t; 
  readonly resultFilled: string; 
  readonly resultRemaining: string; 
  readonly timestamp: string; 
  readonly makerAmount: string; 
  readonly takerAmount: string; 
  readonly tif: Shared_TimeInForce_t
};

export type Event_t = 
    { TAG: "Limit"; _0: Update_t }
  | { TAG: "Trigger"; _0: TriggerUpdate_t };

export type UserSnapshot_t = {
  readonly orders: SnapshotOrder_t[]; 
  readonly marketBalances: UserMarketBalance_t[]; 
  readonly globalDeposits: GlobalDepositBalance_t[]; 
  readonly notifications: Notification__Model_t[]; 
  readonly nonce: number
};

export const SnapshotLimit_toLimit: (_1:SnapshotLimit_t) => Order__Model_Limit_t = Order__RawJS.SnapshotLimit.toLimit as any;

export const SnapshotTrigger_toTrigger: (_1:SnapshotTrigger_t) => Order__Model_Trigger_t = Order__RawJS.SnapshotTrigger.toTrigger as any;

export const Update_toLimit: (_1:Update_t) => Order__Model_Limit_t = Order__RawJS.Update.toLimit as any;

export const TriggerUpdate_toTrigger: (_1:TriggerUpdate_t) => Order__Model_Trigger_t = Order__RawJS.TriggerUpdate.toTrigger as any;

export const Update: { toLimit: (_1:Update_t) => Order__Model_Limit_t } = Order__RawJS.Update as any;

export const SnapshotTrigger: { toTrigger: (_1:SnapshotTrigger_t) => Order__Model_Trigger_t } = Order__RawJS.SnapshotTrigger as any;

export const TriggerUpdate: { toTrigger: (_1:TriggerUpdate_t) => Order__Model_Trigger_t } = Order__RawJS.TriggerUpdate as any;

export const SnapshotLimit: { toLimit: (_1:SnapshotLimit_t) => Order__Model_Limit_t } = Order__RawJS.SnapshotLimit as any;
