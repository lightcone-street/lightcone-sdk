/* TypeScript file generated from PriceHistoryState.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as PriceHistoryStateJS from './PriceHistoryState.res.mjs';

import type {Resolution_t as Shared_Resolution_t} from '../../src/Shared.gen.ts';

import type {lineData as PriceHistory_lineData} from './PriceHistory.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {orderbookPriceCandle as PriceHistory_orderbookPriceCandle} from './PriceHistory.gen.ts';

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export const make: () => t = PriceHistoryStateJS.make as any;

export const applySnapshot: (_1:t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t, candles:PriceHistory_orderbookPriceCandle[]) => void = PriceHistoryStateJS.applySnapshot as any;

export const applyUpdate: (_1:t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t, candle:PriceHistory_orderbookPriceCandle) => void = PriceHistoryStateJS.applyUpdate as any;

export const get: (_1:t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t) => (undefined | PriceHistory_lineData[]) = PriceHistoryStateJS.get as any;

export const clear: (_1:t) => void = PriceHistoryStateJS.clear as any;
