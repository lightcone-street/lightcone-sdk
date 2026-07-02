/* TypeScript file generated from Order.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {DepositSource_t as Shared_DepositSource_t} from '../../src/Shared.gen.ts';

import type {OrderStatus_t as Shared_OrderStatus_t} from '../../src/Shared.gen.ts';

import type {OrderUpdateType_t as Shared_OrderUpdateType_t} from '../../src/Shared.gen.ts';

import type {Side_t as Shared_Side_t} from '../../src/Shared.gen.ts';

import type {TimeInForce_t as Shared_TimeInForce_t} from '../../src/Shared.gen.ts';

import type {TriggerResultStatus_t as Shared_TriggerResultStatus_t} from '../../src/Shared.gen.ts';

import type {TriggerStatus_t as Shared_TriggerStatus_t} from '../../src/Shared.gen.ts';

import type {TriggerType_t as Shared_TriggerType_t} from '../../src/Shared.gen.ts';

import type {TriggerUpdateType_t as Shared_TriggerUpdateType_t} from '../../src/Shared.gen.ts';

import type {notification as Notification_notification} from './Notification.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

export type OrderType_t = 
    "limit"
  | "market"
  | "deposit"
  | "merge"
  | "withdraw"
  | "stop_limit"
  | "take_profit_limit";

export type fillInfo = {
  readonly counterparty: Shared_pubkeyStr; 
  readonly counterpartyOrderHash: string; 
  readonly fillAmount: string; 
  readonly price: string; 
  readonly isMaker: boolean
};

export type SubmitOrderStatus_t = "accepted" | "partial_fill" | "filled";

export type submitOrderResponse = {
  readonly orderHash: string; 
  readonly status: SubmitOrderStatus_t; 
  readonly remaining: string; 
  readonly filled: string; 
  readonly fills: fillInfo[]
};

export type cancelSuccess = { readonly orderHash: string; readonly remaining: string };

export type cancelAllSuccess = {
  readonly cancelledOrderHashes: string[]; 
  readonly count: number; 
  readonly userPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly message: string
};

export type submitOrderRequest = {
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

export type triggerOrderResponse = { readonly triggerOrderId: string; readonly orderHash: string };

export type cancelTriggerSuccess = { readonly triggerOrderId: string };

export type cancelTriggerBody = {
  readonly triggerOrderId: string; 
  readonly maker: string; 
  readonly signatureHex: string
};

export type cancelBody = {
  readonly orderHash: string; 
  readonly maker: string; 
  readonly signatureHex: string
};

export type cancelAllBody = {
  readonly userPubkey: string; 
  readonly orderbookId: string; 
  readonly signatureHex: string; 
  readonly timestamp: number; 
  readonly salt: string
};

export type userSnapshotOrderCommon = {
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

export type UserSnapshotOrder_limit = { readonly common: userSnapshotOrderCommon; readonly txSignature?: string };

export type UserSnapshotOrder_trigger = {
  readonly common: userSnapshotOrderCommon; 
  readonly triggerOrderId: string; 
  readonly triggerPrice: string; 
  readonly triggerType: Shared_TriggerType_t; 
  readonly timeInForce?: Shared_TimeInForce_t
};

export type UserSnapshotOrder_t = 
    { TAG: "Limit"; _0: UserSnapshotOrder_limit }
  | { TAG: "Trigger"; _0: UserSnapshotOrder_trigger };

export type userOutcomeBalance = {
  readonly outcomeIndex: number; 
  readonly conditionalToken: Shared_pubkeyStr; 
  readonly balance: string; 
  readonly balanceIdle: string; 
  readonly balanceOnBook: string
};

export type userDepositAssetBalance = { readonly depositAsset: Shared_pubkeyStr; readonly outcomes: userOutcomeBalance[] };

export type userMarketBalance = { readonly marketPubkey: Shared_pubkeyStr; readonly depositAssets: userDepositAssetBalance[] };

export type userOrdersResponse = {
  readonly userPubkey: Shared_pubkeyStr; 
  readonly orders: UserSnapshotOrder_t[]; 
  readonly marketBalances: userMarketBalance[]; 
  readonly nextCursor?: string; 
  readonly hasMore: boolean
};

export type Role_t = "maker" | "taker";

export type FillStatus_t = "filled" | "cancelled" | "partially_filled";

export type orderFillEvent = {
  readonly fillAmount: string; 
  readonly txSignature: string; 
  readonly filledAt: number
};

export type userOrderFill = {
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
  readonly fills: orderFillEvent[]
};

export type userOrderFillsResponse = {
  readonly orders: userOrderFill[]; 
  readonly nextCursor?: string; 
  readonly hasMore: boolean
};

export type conditionalBalance = {
  readonly outcomeIndex: number; 
  readonly conditionalToken: Shared_pubkeyStr; 
  readonly idle: string; 
  readonly onBook: string
};

export type userOrderUpdateBalance = { readonly outcomes: conditionalBalance[] };

export type wsOrder = {
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
  readonly balance?: userOrderUpdateBalance
};

export type orderUpdate = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly timestamp: string; 
  readonly txSignature?: string; 
  readonly updateType: Shared_OrderUpdateType_t; 
  readonly order: wsOrder
};

export type triggerOrderUpdate = {
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

export type OrderEvent_t = 
    { TAG: "Limit"; _0: orderUpdate }
  | { TAG: "Trigger"; _0: triggerOrderUpdate };

export type globalDepositBalance = { readonly mint: Shared_pubkeyStr; readonly balance: string };

export type userSnapshot = {
  readonly orders: UserSnapshotOrder_t[]; 
  readonly marketBalances: userMarketBalance[]; 
  readonly globalDeposits: globalDepositBalance[]; 
  readonly notifications: Notification_notification[]; 
  readonly nonce: number
};
