/* TypeScript file generated from OrderbookState.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as OrderbookStateJS from './OrderbookState.res.mjs';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {orderBook as Orderbook_orderBook} from './Orderbook.gen.ts';

export type refreshReason = "ServerResync";

export type applyResult = 
    "Applied"
  | { TAG: "RefreshRequired"; _0: refreshReason };

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export const make: (_1:Shared_orderBookId) => t = OrderbookStateJS.make as any;

export const apply: (_1:t, _2:Orderbook_orderBook) => applyResult = OrderbookStateJS.apply as any;

export const bestBid: (_1:t) => (undefined | string) = OrderbookStateJS.bestBid as any;

export const bestAsk: (_1:t) => (undefined | string) = OrderbookStateJS.bestAsk as any;

export const midPrice: (_1:t) => (undefined | string) = OrderbookStateJS.midPrice as any;

export const spread: (_1:t) => (undefined | string) = OrderbookStateJS.spread as any;

export const bids: (_1:t) => Array<[string, string]> = OrderbookStateJS.bids as any;

export const asks: (_1:t) => Array<[string, string]> = OrderbookStateJS.asks as any;

export const isEmpty: (_1:t) => boolean = OrderbookStateJS.isEmpty as any;

export const seq: (_1:t) => number = OrderbookStateJS.seq as any;

export const orderbookId: (_1:t) => Shared_orderBookId = OrderbookStateJS.orderbookId as any;

export const clear: (_1:t) => void = OrderbookStateJS.clear as any;
