/* TypeScript file generated from Orderbook.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Side_t as Shared_Side_t} from '../../src/Shared.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

export type BookAggregation_t = { readonly nSigFigs?: number; readonly mantissa?: number };

export type wsBookLevel = {
  readonly side: Shared_Side_t; 
  readonly price: string; 
  readonly size: string
};

export type restBookLevel = {
  readonly price: string; 
  readonly size: string; 
  readonly orders?: number
};

export type orderbookDepthDecimals = { readonly price: number; readonly size: number };

export type orderbookDepthResponse = {
  readonly orderbookId: Shared_orderBookId; 
  readonly marketPubkey?: string; 
  readonly bestBid?: string; 
  readonly bestAsk?: string; 
  readonly spread?: string; 
  readonly tickSize?: string; 
  readonly bids: restBookLevel[]; 
  readonly asks: restBookLevel[]; 
  readonly decimals?: orderbookDepthDecimals
};

export type orderBook = {
  readonly id: Shared_orderBookId; 
  readonly isSnapshot: boolean; 
  readonly seq: number; 
  readonly resync: boolean; 
  readonly bids: wsBookLevel[]; 
  readonly asks: wsBookLevel[]; 
  readonly nSigFigs?: number; 
  readonly mantissa?: number
};
