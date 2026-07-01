/* TypeScript file generated from DepositPriceState.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as DepositPriceStateJS from './DepositPriceState.res.mjs';

import type {Resolution_t as Shared_Resolution_t} from '../../src/Shared.gen.ts';

import type {depositPriceCandle as PriceHistory_depositPriceCandle} from './PriceHistory.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

export type latestDepositPrice = { readonly price: string; readonly eventTime: number };

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export const make: () => t = DepositPriceStateJS.make as any;

export const applySnapshot: (_1:t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t, candles:PriceHistory_depositPriceCandle[]) => void = DepositPriceStateJS.applySnapshot as any;

export const applyCandle: (_1:t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t, candle:PriceHistory_depositPriceCandle) => void = DepositPriceStateJS.applyCandle as any;

export const applyPriceTick: (_1:t, depositAsset:Shared_pubkeyStr, price:string, eventTime:number) => void = DepositPriceStateJS.applyPriceTick as any;

export const applyAssetSnapshot: (_1:t, depositAsset:Shared_pubkeyStr, price:string) => void = DepositPriceStateJS.applyAssetSnapshot as any;

export const getCandles: (_1:t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t) => (undefined | PriceHistory_depositPriceCandle[]) = DepositPriceStateJS.getCandles as any;

export const getLatestPrice: (_1:t, depositAsset:Shared_pubkeyStr) => (undefined | latestDepositPrice) = DepositPriceStateJS.getLatestPrice as any;

export const clear: (_1:t) => void = DepositPriceStateJS.clear as any;
