/* TypeScript file generated from Subscriptions.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Resolution_t as Shared_Resolution_t} from '../../src/Shared.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

export type SubscribeParams_booksParams = {
  readonly orderbookIds: Shared_orderBookId[]; 
  readonly nSigFigs?: number; 
  readonly mantissa?: number
};

export type SubscribeParams_priceHistoryParams = {
  readonly orderbookId: Shared_orderBookId; 
  readonly resolution: Shared_Resolution_t; 
  readonly includeOhlcv: boolean
};

export type SubscribeParams_depositPriceParams = { readonly depositAsset: Shared_pubkeyStr; readonly resolution: Shared_Resolution_t };

export type SubscribeParams_t = 
    { TAG: "Books"; _0: SubscribeParams_booksParams }
  | { TAG: "Trades"; _0: Shared_orderBookId[] }
  | { TAG: "User"; _0: Shared_pubkeyStr }
  | { TAG: "PriceHistory"; _0: SubscribeParams_priceHistoryParams }
  | { TAG: "Ticker"; _0: Shared_orderBookId[] }
  | { TAG: "Market"; _0: Shared_pubkeyStr }
  | { TAG: "DepositPrice"; _0: SubscribeParams_depositPriceParams }
  | { TAG: "DepositAssetPrice"; _0: Shared_pubkeyStr };

export type UnsubscribeParams_booksParams = {
  readonly orderbookIds: Shared_orderBookId[]; 
  readonly nSigFigs?: number; 
  readonly mantissa?: number
};

export type UnsubscribeParams_priceHistoryParams = { readonly orderbookId: Shared_orderBookId; readonly resolution: Shared_Resolution_t };

export type UnsubscribeParams_depositPriceParams = { readonly depositAsset: Shared_pubkeyStr; readonly resolution: Shared_Resolution_t };

export type UnsubscribeParams_t = 
    { TAG: "Books"; _0: UnsubscribeParams_booksParams }
  | { TAG: "Trades"; _0: Shared_orderBookId[] }
  | { TAG: "User"; _0: Shared_pubkeyStr }
  | { TAG: "PriceHistory"; _0: UnsubscribeParams_priceHistoryParams }
  | { TAG: "Ticker"; _0: Shared_orderBookId[] }
  | { TAG: "Market"; _0: Shared_pubkeyStr }
  | { TAG: "DepositPrice"; _0: UnsubscribeParams_depositPriceParams }
  | { TAG: "DepositAssetPrice"; _0: Shared_pubkeyStr };
