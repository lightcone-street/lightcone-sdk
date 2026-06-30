/* TypeScript file generated from PriceHistory.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Resolution_t as Shared_Resolution_t} from '../../src/Shared.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

export type orderbookPriceCandle = {
  readonly t: number; 
  readonly m?: string; 
  readonly o?: string; 
  readonly h?: string; 
  readonly l?: string; 
  readonly c?: string; 
  readonly v?: string; 
  readonly bb?: string; 
  readonly ba?: string
};

export type priceHistoryDecimals = { readonly price: number; readonly volume: number };

export type orderbookPriceHistoryResponse = {
  readonly orderbookId: Shared_orderBookId; 
  readonly resolution: Shared_Resolution_t; 
  readonly includeOhlcv: boolean; 
  readonly prices: orderbookPriceCandle[]; 
  readonly nextCursor?: number; 
  readonly hasMore: boolean; 
  readonly decimals: priceHistoryDecimals
};

export type depositPriceCandle = {
  readonly t: number; 
  readonly tc: number; 
  readonly c: string
};

export type depositPriceHistoryResponse = {
  readonly depositAsset: Shared_pubkeyStr; 
  readonly binanceSymbol: string; 
  readonly resolution: Shared_Resolution_t; 
  readonly prices: depositPriceCandle[]; 
  readonly nextCursor?: number; 
  readonly hasMore: boolean
};

export type depositAssetPricesSnapshotResponse = { readonly prices: {[id: string]: string} };

export type lineData = { readonly time: number; readonly value: string };

export type orderbookPriceHistoryQuery = {
  readonly resolution: Shared_Resolution_t; 
  readonly fromMs?: number; 
  readonly toMs?: number; 
  readonly cursor?: number; 
  readonly limit?: number; 
  readonly includeOhlcv: boolean
};

export type depositPriceHistoryQuery = {
  readonly resolution: Shared_Resolution_t; 
  readonly fromMs?: number; 
  readonly toMs?: number; 
  readonly cursor?: number; 
  readonly limit?: number
};
