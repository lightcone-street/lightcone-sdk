/* TypeScript file generated from PriceHistory__State.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as PriceHistory__StateJS from './PriceHistory__State.res.mjs';

import type {LineData_t as PriceHistory__Model_LineData_t} from './PriceHistory__Model.gen.ts';

import type {OrderbookCandle_t as PriceHistory__Raw_OrderbookCandle_t} from './PriceHistory__Raw.gen.ts';

import type {Resolution_t as Shared_Resolution_t} from '../../../src/Shared.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../../src/Shared.gen.ts';

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export const make: () => t = PriceHistory__StateJS.make as any;

export const applySnapshot: (_1:t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t, candles:PriceHistory__Raw_OrderbookCandle_t[]) => void = PriceHistory__StateJS.applySnapshot as any;

export const applyUpdate: (_1:t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t, candle:PriceHistory__Raw_OrderbookCandle_t) => void = PriceHistory__StateJS.applyUpdate as any;

export const get: (_1:t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t) => (undefined | PriceHistory__Model_LineData_t[]) = PriceHistory__StateJS.get as any;

export const clear: (_1:t) => void = PriceHistory__StateJS.clear as any;
