/* TypeScript file generated from Orderbook__State.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Orderbook__StateJS from './Orderbook__State.res.mjs';

import type {Book_t as Orderbook__Raw_Book_t} from './Orderbook__Raw.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../../src/Shared.gen.ts';

export type refreshReason = "ServerResync";

export type applyResult = 
    "Applied"
  | { TAG: "RefreshRequired"; _0: refreshReason };

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export const make: (_1:Shared_orderBookId) => t = Orderbook__StateJS.make as any;

export const apply: (_1:t, _2:Orderbook__Raw_Book_t) => applyResult = Orderbook__StateJS.apply as any;

export const bestBid: (_1:t) => (undefined | string) = Orderbook__StateJS.bestBid as any;

export const bestAsk: (_1:t) => (undefined | string) = Orderbook__StateJS.bestAsk as any;

export const midPrice: (_1:t) => (undefined | string) = Orderbook__StateJS.midPrice as any;

export const spread: (_1:t) => (undefined | string) = Orderbook__StateJS.spread as any;

export const bids: (_1:t) => Array<[string, string]> = Orderbook__StateJS.bids as any;

export const asks: (_1:t) => Array<[string, string]> = Orderbook__StateJS.asks as any;

export const isEmpty: (_1:t) => boolean = Orderbook__StateJS.isEmpty as any;

export const seq: (_1:t) => number = Orderbook__StateJS.seq as any;

export const orderbookId: (_1:t) => Shared_orderBookId = Orderbook__StateJS.orderbookId as any;

export const clear: (_1:t) => void = Orderbook__StateJS.clear as any;
