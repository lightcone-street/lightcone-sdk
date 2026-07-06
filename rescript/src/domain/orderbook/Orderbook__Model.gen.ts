/* TypeScript file generated from Orderbook__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {orderBookId as Shared_orderBookId} from '../../../src/Shared.gen.ts';

export type Aggregation_t = { readonly nSigFigs?: number; readonly mantissa?: number };

export type Ticker_t = {
  readonly orderbookId: Shared_orderBookId; 
  readonly bestBid?: string; 
  readonly bestAsk?: string; 
  readonly midPrice?: string
};
