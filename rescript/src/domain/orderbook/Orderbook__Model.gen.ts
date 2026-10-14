/* TypeScript file generated from Orderbook__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

export type Aggregation_t = { readonly nSigFigs?: number; readonly mantissa?: number };

export type Ticker_t = {
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly bestBid?: string;
  readonly bestAsk?: string;
  readonly midPrice?: string
};
