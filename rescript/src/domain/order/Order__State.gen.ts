/* TypeScript file generated from Order__State.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Order__StateJS from './Order__State.res.mjs';

import type {Limit_t as Order__Model_Limit_t} from './Order__Model.gen.ts';

import type {SnapshotOrder_t as Order__Raw_SnapshotOrder_t} from './Order__Raw.gen.ts';

import type {Trigger_t as Order__Model_Trigger_t} from './Order__Model.gen.ts';

import type {Update_t as Order__Raw_Update_t} from './Order__Raw.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../../src/Shared.gen.ts';

export abstract class Limits_t { protected opaque!: any }; /* simulate opaque types */

export abstract class Triggers_t { protected opaque!: any }; /* simulate opaque types */

export const Limits_make: () => Limits_t = Order__StateJS.Limits.make as any;

export const Limits_get: (_1:Limits_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | Order__Model_Limit_t[]) = Order__StateJS.Limits.get as any;

export const Limits_getByMarket: (_1:Limits_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: Order__Model_Limit_t[]}) = Order__StateJS.Limits.getByMarket as any;

export const Limits_insert: (_1:Limits_t, _2:Order__Model_Limit_t) => void = Order__StateJS.Limits.insert as any;

export const Limits_upsert: (_1:Limits_t, _2:Order__Raw_Update_t) => void = Order__StateJS.Limits.upsert as any;

export const Limits_remove: (_1:Limits_t, orderHash:string) => void = Order__StateJS.Limits.remove as any;

export const Limits_clear: (_1:Limits_t) => void = Order__StateJS.Limits.clear as any;

export const Limits_isEmpty: (_1:Limits_t) => boolean = Order__StateJS.Limits.isEmpty as any;

export const Triggers_make: () => Triggers_t = Order__StateJS.Triggers.make as any;

export const Triggers_get: (_1:Triggers_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | Order__Model_Trigger_t[]) = Order__StateJS.Triggers.get as any;

export const Triggers_getByMarket: (_1:Triggers_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: Order__Model_Trigger_t[]}) = Order__StateJS.Triggers.getByMarket as any;

export const Triggers_all: (_1:Triggers_t) => Order__Model_Trigger_t[] = Order__StateJS.Triggers.all as any;

export const Triggers_getById: (_1:Triggers_t, triggerOrderId:string) => (undefined | Order__Model_Trigger_t) = Order__StateJS.Triggers.getById as any;

export const Triggers_insert: (_1:Triggers_t, _2:Order__Model_Trigger_t) => void = Order__StateJS.Triggers.insert as any;

export const Triggers_remove: (_1:Triggers_t, triggerOrderId:string) => (undefined | Order__Model_Trigger_t) = Order__StateJS.Triggers.remove as any;

export const Triggers_clear: (_1:Triggers_t) => void = Order__StateJS.Triggers.clear as any;

export const Triggers_isEmpty: (_1:Triggers_t) => boolean = Order__StateJS.Triggers.isEmpty as any;

export const Triggers_size: (_1:Triggers_t) => number = Order__StateJS.Triggers.size as any;

export const fromSnapshotOrders: (_1:Order__Raw_SnapshotOrder_t[]) => [Limits_t, Triggers_t] = Order__StateJS.fromSnapshotOrders as any;

export const Limits: {
  upsert: (_1:Limits_t, _2:Order__Raw_Update_t) => void; 
  insert: (_1:Limits_t, _2:Order__Model_Limit_t) => void; 
  get: (_1:Limits_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | Order__Model_Limit_t[]); 
  remove: (_1:Limits_t, orderHash:string) => void; 
  make: () => Limits_t; 
  clear: (_1:Limits_t) => void; 
  getByMarket: (_1:Limits_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: Order__Model_Limit_t[]}); 
  isEmpty: (_1:Limits_t) => boolean
} = Order__StateJS.Limits as any;

export const Triggers: {
  insert: (_1:Triggers_t, _2:Order__Model_Trigger_t) => void; 
  size: (_1:Triggers_t) => number; 
  get: (_1:Triggers_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | Order__Model_Trigger_t[]); 
  remove: (_1:Triggers_t, triggerOrderId:string) => (undefined | Order__Model_Trigger_t); 
  getById: (_1:Triggers_t, triggerOrderId:string) => (undefined | Order__Model_Trigger_t); 
  make: () => Triggers_t; 
  clear: (_1:Triggers_t) => void; 
  getByMarket: (_1:Triggers_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: Order__Model_Trigger_t[]}); 
  all: (_1:Triggers_t) => Order__Model_Trigger_t[]; 
  isEmpty: (_1:Triggers_t) => boolean
} = Order__StateJS.Triggers as any;
