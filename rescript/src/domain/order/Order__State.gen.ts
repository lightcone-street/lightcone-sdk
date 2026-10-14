/* TypeScript file generated from Order__State.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Order__StateJS from './Order__State.res.mjs';

import type {Order__Model_Limit_t as LightconeSdk_Order__Model_Limit_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_SnapshotOrder_t as LightconeSdk_Order__Raw_SnapshotOrder_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_Update_t as LightconeSdk_Order__Raw_Update_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

export abstract class Limits_t { protected opaque!: any }; /* simulate opaque types */

export const Limits_make: () => Limits_t = Order__StateJS.Limits.make as any;

export const Limits_get: (_1:Limits_t, marketPubkey:LightconeSdk_Shared_pubkeyStr, orderbookId:LightconeSdk_Shared_orderBookId) => (undefined | LightconeSdk_Order__Model_Limit_t[]) = Order__StateJS.Limits.get as any;

export const Limits_getByMarket: (_1:Limits_t, marketPubkey:LightconeSdk_Shared_pubkeyStr) => (undefined | {[id: string]: LightconeSdk_Order__Model_Limit_t[]}) = Order__StateJS.Limits.getByMarket as any;

export const Limits_insert: (_1:Limits_t, _2:LightconeSdk_Order__Model_Limit_t) => void = Order__StateJS.Limits.insert as any;

export const Limits_upsert: (_1:Limits_t, _2:LightconeSdk_Order__Raw_Update_t) => void = Order__StateJS.Limits.upsert as any;

export const Limits_remove: (_1:Limits_t, orderHash:string) => void = Order__StateJS.Limits.remove as any;

export const Limits_clear: (_1:Limits_t) => void = Order__StateJS.Limits.clear as any;

export const Limits_isEmpty: (_1:Limits_t) => boolean = Order__StateJS.Limits.isEmpty as any;

export const fromSnapshotOrders: (_1:LightconeSdk_Order__Raw_SnapshotOrder_t[]) => Limits_t = Order__StateJS.fromSnapshotOrders as any;

export const Limits: {
  upsert: (_1:Limits_t, _2:LightconeSdk_Order__Raw_Update_t) => void;
  insert: (_1:Limits_t, _2:LightconeSdk_Order__Model_Limit_t) => void;
  get: (_1:Limits_t, marketPubkey:LightconeSdk_Shared_pubkeyStr, orderbookId:LightconeSdk_Shared_orderBookId) => (undefined | LightconeSdk_Order__Model_Limit_t[]);
  remove: (_1:Limits_t, orderHash:string) => void;
  make: () => Limits_t;
  clear: (_1:Limits_t) => void;
  getByMarket: (_1:Limits_t, marketPubkey:LightconeSdk_Shared_pubkeyStr) => (undefined | {[id: string]: LightconeSdk_Order__Model_Limit_t[]});
  isEmpty: (_1:Limits_t) => boolean
} = Order__StateJS.Limits as any;
