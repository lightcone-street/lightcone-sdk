/* TypeScript file generated from Orderbook__State.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Orderbook__StateJS from './Orderbook__State.res.mjs';

import type {Orderbook__Model_Aggregation_t as LightconeSdk_Orderbook__Model_Aggregation_t} from './LightconeSdk.gen.ts';

import type {Orderbook__Raw_Book_t as LightconeSdk_Orderbook__Raw_Book_t} from './LightconeSdk.gen.ts';

import type {Orderbook__Raw_WsLevel_t as LightconeSdk_Orderbook__Raw_WsLevel_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

export type refreshReason = "ServerResync";

export type applyResult =
    "Applied"
  | "DiscardedStale"
  | "SubscriptionMismatch"
  | { TAG: "RefreshRequired"; _0: refreshReason };

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export const make: (_1:LightconeSdk_Shared_orderBookId) => t = Orderbook__StateJS.make as any;

export const apply: (_1:t, _2:LightconeSdk_Orderbook__Raw_Book_t) => applyResult = Orderbook__StateJS.apply as any;

export const bestBid: (_1:t) => (undefined | string) = Orderbook__StateJS.bestBid as any;

export const bestAsk: (_1:t) => (undefined | string) = Orderbook__StateJS.bestAsk as any;

export const midPrice: (_1:t) => (undefined | string) = Orderbook__StateJS.midPrice as any;

export const spread: (_1:t) => (undefined | string) = Orderbook__StateJS.spread as any;

export const bids: (_1:t) => Array<[string, string]> = Orderbook__StateJS.bids as any;

export const asks: (_1:t) => Array<[string, string]> = Orderbook__StateJS.asks as any;

export const isEmpty: (_1:t) => boolean = Orderbook__StateJS.isEmpty as any;

export const seq: (_1:t) => bigint = Orderbook__StateJS.seq as any;

export const orderbookId: (_1:t) => LightconeSdk_Shared_orderBookId = Orderbook__StateJS.orderbookId as any;

export const clear: (_1:t) => void = Orderbook__StateJS.clear as any;

export const withAggregation: (_1:LightconeSdk_Shared_orderBookId, _2:LightconeSdk_Orderbook__Model_Aggregation_t) => t = Orderbook__StateJS.withAggregation as any;

export const beginGeneration: (_1:t) => void = Orderbook__StateJS.beginGeneration as any;

export const bidsTruncated: (_1:t) => boolean = Orderbook__StateJS.bidsTruncated as any;

export const asksTruncated: (_1:t) => boolean = Orderbook__StateJS.asksTruncated as any;

export const bidLevels: (_1:t) => LightconeSdk_Orderbook__Raw_WsLevel_t[] = Orderbook__StateJS.bidLevels as any;

export const askLevels: (_1:t) => LightconeSdk_Orderbook__Raw_WsLevel_t[] = Orderbook__StateJS.askLevels as any;
