/* TypeScript file generated from TradeState.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as TradeStateJS from './TradeState.res.mjs';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {trade as Trade_trade} from './Trade.gen.ts';

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export const make: (orderbookId:Shared_orderBookId, maxSize:number) => t = TradeStateJS.make as any;

export const push: (_1:t, _2:Trade_trade) => void = TradeStateJS.push as any;

export const replace: (_1:t, _2:Trade_trade[]) => void = TradeStateJS.replace as any;

export const trades: (_1:t) => Trade_trade[] = TradeStateJS.trades as any;

export const latest: (_1:t) => (undefined | Trade_trade) = TradeStateJS.latest as any;

export const clear: (_1:t) => void = TradeStateJS.clear as any;

export const size: (_1:t) => number = TradeStateJS.size as any;

export const isEmpty: (_1:t) => boolean = TradeStateJS.isEmpty as any;
