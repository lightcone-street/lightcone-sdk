/* TypeScript file generated from Trade__State.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Trade__StateJS from './Trade__State.res.mjs';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

import type {Trade__Model_t as LightconeSdk_Trade__Model_t} from './LightconeSdk.gen.ts';

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export const make: (orderbookId:LightconeSdk_Shared_orderBookId, maxSize:number) => t = Trade__StateJS.make as any;

export const push: (_1:t, _2:LightconeSdk_Trade__Model_t) => void = Trade__StateJS.push as any;

export const replace: (_1:t, _2:LightconeSdk_Trade__Model_t[]) => void = Trade__StateJS.replace as any;

export const trades: (_1:t) => LightconeSdk_Trade__Model_t[] = Trade__StateJS.trades as any;

export const latest: (_1:t) => (undefined | LightconeSdk_Trade__Model_t) = Trade__StateJS.latest as any;

export const clear: (_1:t) => void = Trade__StateJS.clear as any;

export const size: (_1:t) => number = Trade__StateJS.size as any;

export const isEmpty: (_1:t) => boolean = Trade__StateJS.isEmpty as any;
