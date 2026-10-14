/* TypeScript file generated from PriceHistory__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Shared_Resolution_t as LightconeSdk_Shared_Resolution_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

export type OrderbookCandle_t = {
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

export type DepositCandle_t = {
  readonly t: number;
  readonly tc: number;
  readonly c: string
};

export type Decimals_t = { readonly price: number; readonly volume: number };

export type OrderbookResponse_t = {
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly resolution: LightconeSdk_Shared_Resolution_t;
  readonly includeOhlcv: boolean;
  readonly prices: OrderbookCandle_t[];
  readonly nextCursor?: number;
  readonly hasMore: boolean;
  readonly decimals: Decimals_t
};

export type DepositResponse_t = {
  readonly depositAsset: LightconeSdk_Shared_pubkeyStr;
  readonly binanceSymbol: string;
  readonly resolution: LightconeSdk_Shared_Resolution_t;
  readonly prices: DepositCandle_t[];
  readonly nextCursor?: number;
  readonly hasMore: boolean
};

export type DepositPricesSnapshotResponse_t = { readonly prices: {[id: string]: string} };
