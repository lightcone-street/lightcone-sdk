/* TypeScript file generated from PriceHistory__DepositState.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as PriceHistory__DepositStateJS from './PriceHistory__DepositState.res.mjs';

import type {DepositCandle_t as PriceHistory__Raw_DepositCandle_t} from './PriceHistory__Raw.gen.ts';

import type {Resolution_t as Shared_Resolution_t} from '../../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../../src/Shared.gen.ts';

export type latestPrice = { readonly price: string; readonly eventTime: number };

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export const make: () => t = PriceHistory__DepositStateJS.make as any;

export const applySnapshot: (_1:t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t, candles:PriceHistory__Raw_DepositCandle_t[]) => void = PriceHistory__DepositStateJS.applySnapshot as any;

export const applyCandle: (_1:t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t, candle:PriceHistory__Raw_DepositCandle_t) => void = PriceHistory__DepositStateJS.applyCandle as any;

export const applyPriceTick: (_1:t, depositAsset:Shared_pubkeyStr, price:string, eventTime:number) => void = PriceHistory__DepositStateJS.applyPriceTick as any;

export const applyAssetSnapshot: (_1:t, depositAsset:Shared_pubkeyStr, price:string) => void = PriceHistory__DepositStateJS.applyAssetSnapshot as any;

export const getCandles: (_1:t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t) => (undefined | PriceHistory__Raw_DepositCandle_t[]) = PriceHistory__DepositStateJS.getCandles as any;

export const getLatestPrice: (_1:t, depositAsset:Shared_pubkeyStr) => (undefined | latestPrice) = PriceHistory__DepositStateJS.getLatestPrice as any;

export const clear: (_1:t) => void = PriceHistory__DepositStateJS.clear as any;
