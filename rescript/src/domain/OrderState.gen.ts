/* TypeScript file generated from OrderState.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as OrderStateJS from './OrderState.res.mjs';

import type {OrderStatus_t as Shared_OrderStatus_t} from '../../src/Shared.gen.ts';

import type {Side_t as Shared_Side_t} from '../../src/Shared.gen.ts';

import type {TimeInForce_t as Shared_TimeInForce_t} from '../../src/Shared.gen.ts';

import type {TriggerType_t as Shared_TriggerType_t} from '../../src/Shared.gen.ts';

import type {UserSnapshotOrder_limit as Order_UserSnapshotOrder_limit} from './Order.gen.ts';

import type {UserSnapshotOrder_t as Order_UserSnapshotOrder_t} from './Order.gen.ts';

import type {UserSnapshotOrder_trigger as Order_UserSnapshotOrder_trigger} from './Order.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {orderUpdate as Order_orderUpdate} from './Order.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

import type {triggerOrderUpdate as Order_triggerOrderUpdate} from './Order.gen.ts';

export type limitOrder = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly txSignature?: string; 
  readonly baseMint: Shared_pubkeyStr; 
  readonly quoteMint: Shared_pubkeyStr; 
  readonly orderHash: string; 
  readonly side: Shared_Side_t; 
  readonly size: string; 
  readonly price: string; 
  readonly filledSize: string; 
  readonly remainingSize: string; 
  readonly createdAt: number; 
  readonly status: Shared_OrderStatus_t; 
  readonly outcomeIndex: number
};

export type triggerOrder = {
  readonly triggerOrderId: string; 
  readonly orderHash: string; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly triggerPrice: string; 
  readonly triggerType: Shared_TriggerType_t; 
  readonly side: Shared_Side_t; 
  readonly amountIn: string; 
  readonly amountOut: string; 
  readonly timeInForce: Shared_TimeInForce_t; 
  readonly createdAt: number
};

export abstract class UserOpenLimitOrders_t { protected opaque!: any }; /* simulate opaque types */

export abstract class UserTriggerOrders_t { protected opaque!: any }; /* simulate opaque types */

export const limitPrice: (_1:triggerOrder) => (undefined | string) = OrderStateJS.limitPrice as any;

export const limitOrderOfUpdate: (_1:Order_orderUpdate) => limitOrder = OrderStateJS.limitOrderOfUpdate as any;

export const limitOrderOfSnapshot: (_1:Order_UserSnapshotOrder_limit) => limitOrder = OrderStateJS.limitOrderOfSnapshot as any;

export const triggerOrderOfSnapshot: (_1:Order_UserSnapshotOrder_trigger) => triggerOrder = OrderStateJS.triggerOrderOfSnapshot as any;

export const triggerOrderOfUpdate: (_1:Order_triggerOrderUpdate) => triggerOrder = OrderStateJS.triggerOrderOfUpdate as any;

export const UserOpenLimitOrders_make: () => UserOpenLimitOrders_t = OrderStateJS.UserOpenLimitOrders.make as any;

export const UserOpenLimitOrders_get: (_1:UserOpenLimitOrders_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | limitOrder[]) = OrderStateJS.UserOpenLimitOrders.get as any;

export const UserOpenLimitOrders_getByMarket: (_1:UserOpenLimitOrders_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: limitOrder[]}) = OrderStateJS.UserOpenLimitOrders.getByMarket as any;

export const UserOpenLimitOrders_insert: (_1:UserOpenLimitOrders_t, _2:limitOrder) => void = OrderStateJS.UserOpenLimitOrders.insert as any;

export const UserOpenLimitOrders_upsert: (_1:UserOpenLimitOrders_t, _2:Order_orderUpdate) => void = OrderStateJS.UserOpenLimitOrders.upsert as any;

export const UserOpenLimitOrders_remove: (_1:UserOpenLimitOrders_t, orderHash:string) => void = OrderStateJS.UserOpenLimitOrders.remove as any;

export const UserOpenLimitOrders_clear: (_1:UserOpenLimitOrders_t) => void = OrderStateJS.UserOpenLimitOrders.clear as any;

export const UserOpenLimitOrders_isEmpty: (_1:UserOpenLimitOrders_t) => boolean = OrderStateJS.UserOpenLimitOrders.isEmpty as any;

export const UserTriggerOrders_make: () => UserTriggerOrders_t = OrderStateJS.UserTriggerOrders.make as any;

export const UserTriggerOrders_get: (_1:UserTriggerOrders_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | triggerOrder[]) = OrderStateJS.UserTriggerOrders.get as any;

export const UserTriggerOrders_getByMarket: (_1:UserTriggerOrders_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: triggerOrder[]}) = OrderStateJS.UserTriggerOrders.getByMarket as any;

export const UserTriggerOrders_all: (_1:UserTriggerOrders_t) => triggerOrder[] = OrderStateJS.UserTriggerOrders.all as any;

export const UserTriggerOrders_getById: (_1:UserTriggerOrders_t, triggerOrderId:string) => (undefined | triggerOrder) = OrderStateJS.UserTriggerOrders.getById as any;

export const UserTriggerOrders_insert: (_1:UserTriggerOrders_t, _2:triggerOrder) => void = OrderStateJS.UserTriggerOrders.insert as any;

export const UserTriggerOrders_remove: (_1:UserTriggerOrders_t, triggerOrderId:string) => (undefined | triggerOrder) = OrderStateJS.UserTriggerOrders.remove as any;

export const UserTriggerOrders_clear: (_1:UserTriggerOrders_t) => void = OrderStateJS.UserTriggerOrders.clear as any;

export const UserTriggerOrders_isEmpty: (_1:UserTriggerOrders_t) => boolean = OrderStateJS.UserTriggerOrders.isEmpty as any;

export const UserTriggerOrders_size: (_1:UserTriggerOrders_t) => number = OrderStateJS.UserTriggerOrders.size as any;

export const ofSnapshotOrders: (_1:Order_UserSnapshotOrder_t[]) => [UserOpenLimitOrders_t, UserTriggerOrders_t] = OrderStateJS.ofSnapshotOrders as any;

export const UserOpenLimitOrders: {
  upsert: (_1:UserOpenLimitOrders_t, _2:Order_orderUpdate) => void; 
  insert: (_1:UserOpenLimitOrders_t, _2:limitOrder) => void; 
  get: (_1:UserOpenLimitOrders_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | limitOrder[]); 
  remove: (_1:UserOpenLimitOrders_t, orderHash:string) => void; 
  make: () => UserOpenLimitOrders_t; 
  clear: (_1:UserOpenLimitOrders_t) => void; 
  getByMarket: (_1:UserOpenLimitOrders_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: limitOrder[]}); 
  isEmpty: (_1:UserOpenLimitOrders_t) => boolean
} = OrderStateJS.UserOpenLimitOrders as any;

export const UserTriggerOrders: {
  insert: (_1:UserTriggerOrders_t, _2:triggerOrder) => void; 
  size: (_1:UserTriggerOrders_t) => number; 
  get: (_1:UserTriggerOrders_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | triggerOrder[]); 
  remove: (_1:UserTriggerOrders_t, triggerOrderId:string) => (undefined | triggerOrder); 
  getById: (_1:UserTriggerOrders_t, triggerOrderId:string) => (undefined | triggerOrder); 
  make: () => UserTriggerOrders_t; 
  clear: (_1:UserTriggerOrders_t) => void; 
  getByMarket: (_1:UserTriggerOrders_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: triggerOrder[]}); 
  all: (_1:UserTriggerOrders_t) => triggerOrder[]; 
  isEmpty: (_1:UserTriggerOrders_t) => boolean
} = OrderStateJS.UserTriggerOrders as any;
