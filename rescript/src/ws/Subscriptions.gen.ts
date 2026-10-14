/* TypeScript file generated from Subscriptions.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Shared_Resolution_t as LightconeSdk_Shared_Resolution_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

export type SubscribeParams_booksParams = {
  readonly orderbookIds: LightconeSdk_Shared_orderBookId[];
  readonly nSigFigs?: number;
  readonly mantissa?: number
};

export type SubscribeParams_priceHistoryParams = {
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly resolution: LightconeSdk_Shared_Resolution_t;
  readonly includeOhlcv: boolean
};

export type SubscribeParams_depositPriceParams = { readonly depositAsset: LightconeSdk_Shared_pubkeyStr; readonly resolution: LightconeSdk_Shared_Resolution_t };

export type SubscribeParams_t =
    { TAG: "Books"; _0: SubscribeParams_booksParams }
  | { TAG: "Trades"; _0: LightconeSdk_Shared_orderBookId[] }
  | { TAG: "User"; _0: LightconeSdk_Shared_pubkeyStr }
  | { TAG: "WalletDepositBalances"; _0: LightconeSdk_Shared_pubkeyStr }
  | { TAG: "PriceHistory"; _0: SubscribeParams_priceHistoryParams }
  | { TAG: "Ticker"; _0: LightconeSdk_Shared_orderBookId[] }
  | { TAG: "Market"; _0: LightconeSdk_Shared_pubkeyStr }
  | { TAG: "DepositPrice"; _0: SubscribeParams_depositPriceParams }
  | { TAG: "DepositAssetPrice"; _0: LightconeSdk_Shared_pubkeyStr };

export type UnsubscribeParams_booksParams = {
  readonly orderbookIds: LightconeSdk_Shared_orderBookId[];
  readonly nSigFigs?: number;
  readonly mantissa?: number
};

export type UnsubscribeParams_priceHistoryParams = { readonly orderbookId: LightconeSdk_Shared_orderBookId; readonly resolution: LightconeSdk_Shared_Resolution_t };

export type UnsubscribeParams_depositPriceParams = { readonly depositAsset: LightconeSdk_Shared_pubkeyStr; readonly resolution: LightconeSdk_Shared_Resolution_t };

export type UnsubscribeParams_t =
    { TAG: "Books"; _0: UnsubscribeParams_booksParams }
  | { TAG: "Trades"; _0: LightconeSdk_Shared_orderBookId[] }
  | { TAG: "User"; _0: LightconeSdk_Shared_pubkeyStr }
  | { TAG: "WalletDepositBalances"; _0: LightconeSdk_Shared_pubkeyStr }
  | { TAG: "PriceHistory"; _0: UnsubscribeParams_priceHistoryParams }
  | { TAG: "Ticker"; _0: LightconeSdk_Shared_orderBookId[] }
  | { TAG: "Market"; _0: LightconeSdk_Shared_pubkeyStr }
  | { TAG: "DepositPrice"; _0: UnsubscribeParams_depositPriceParams }
  | { TAG: "DepositAssetPrice"; _0: LightconeSdk_Shared_pubkeyStr };
