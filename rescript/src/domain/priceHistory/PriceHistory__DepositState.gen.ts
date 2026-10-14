/* TypeScript file generated from PriceHistory__DepositState.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as PriceHistory__DepositStateJS from './PriceHistory__DepositState.res.mjs';

import type {PriceHistory__Raw_DepositCandle_t as LightconeSdk_PriceHistory__Raw_DepositCandle_t} from './LightconeSdk.gen.ts';

import type {Shared_Resolution_t as LightconeSdk_Shared_Resolution_t} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

export type latestPrice = { readonly price: string; readonly eventTime: number };

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export const make: () => t = PriceHistory__DepositStateJS.make as any;

export const applySnapshot: (_1:t, depositAsset:LightconeSdk_Shared_pubkeyStr, resolution:LightconeSdk_Shared_Resolution_t, candles:LightconeSdk_PriceHistory__Raw_DepositCandle_t[]) => void = PriceHistory__DepositStateJS.applySnapshot as any;

export const applyCandle: (_1:t, depositAsset:LightconeSdk_Shared_pubkeyStr, resolution:LightconeSdk_Shared_Resolution_t, candle:LightconeSdk_PriceHistory__Raw_DepositCandle_t) => void = PriceHistory__DepositStateJS.applyCandle as any;

export const applyPriceTick: (_1:t, depositAsset:LightconeSdk_Shared_pubkeyStr, price:string, eventTime:number) => void = PriceHistory__DepositStateJS.applyPriceTick as any;

export const applyAssetSnapshot: (_1:t, depositAsset:LightconeSdk_Shared_pubkeyStr, price:string) => void = PriceHistory__DepositStateJS.applyAssetSnapshot as any;

export const getCandles: (_1:t, depositAsset:LightconeSdk_Shared_pubkeyStr, resolution:LightconeSdk_Shared_Resolution_t) => (undefined | LightconeSdk_PriceHistory__Raw_DepositCandle_t[]) = PriceHistory__DepositStateJS.getCandles as any;

export const getLatestPrice: (_1:t, depositAsset:LightconeSdk_Shared_pubkeyStr) => (undefined | latestPrice) = PriceHistory__DepositStateJS.getLatestPrice as any;

export const clear: (_1:t) => void = PriceHistory__DepositStateJS.clear as any;
