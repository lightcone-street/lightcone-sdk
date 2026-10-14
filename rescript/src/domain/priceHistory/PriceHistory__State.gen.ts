/* TypeScript file generated from PriceHistory__State.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as PriceHistory__StateJS from './PriceHistory__State.res.mjs';

import type {PriceHistory__Model_LineData_t as LightconeSdk_PriceHistory__Model_LineData_t} from './LightconeSdk.gen.ts';

import type {PriceHistory__Raw_OrderbookCandle_t as LightconeSdk_PriceHistory__Raw_OrderbookCandle_t} from './LightconeSdk.gen.ts';

import type {Shared_Resolution_t as LightconeSdk_Shared_Resolution_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export const make: () => t = PriceHistory__StateJS.make as any;

export const applySnapshot: (_1:t, orderbookId:LightconeSdk_Shared_orderBookId, resolution:LightconeSdk_Shared_Resolution_t, candles:LightconeSdk_PriceHistory__Raw_OrderbookCandle_t[]) => void = PriceHistory__StateJS.applySnapshot as any;

export const applyUpdate: (_1:t, orderbookId:LightconeSdk_Shared_orderBookId, resolution:LightconeSdk_Shared_Resolution_t, candle:LightconeSdk_PriceHistory__Raw_OrderbookCandle_t) => void = PriceHistory__StateJS.applyUpdate as any;

export const get: (_1:t, orderbookId:LightconeSdk_Shared_orderBookId, resolution:LightconeSdk_Shared_Resolution_t) => (undefined | LightconeSdk_PriceHistory__Model_LineData_t[]) = PriceHistory__StateJS.get as any;

export const clear: (_1:t) => void = PriceHistory__StateJS.clear as any;
