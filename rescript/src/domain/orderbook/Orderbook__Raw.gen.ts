/* TypeScript file generated from Orderbook__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Side_t as Shared_Side_t} from '../../../src/Shared.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../../src/Shared.gen.ts';

export type WsLevel_t = {
  readonly side: Shared_Side_t; 
  readonly price: string; 
  readonly size: string
};

export type RestLevel_t = {
  readonly price: string; 
  readonly size: string; 
  readonly orders?: number
};

export type DepthDecimals_t = { readonly price: number; readonly size: number };

export type DepthResponse_t = {
  readonly orderbookId: Shared_orderBookId; 
  readonly marketPubkey?: string; 
  readonly bestBid?: string; 
  readonly bestAsk?: string; 
  readonly spread?: string; 
  readonly tickSize?: string; 
  readonly bids: RestLevel_t[]; 
  readonly asks: RestLevel_t[]; 
  readonly decimals?: DepthDecimals_t
};

export type Book_t = {
  readonly id: Shared_orderBookId; 
  readonly isSnapshot: boolean; 
  readonly seq: number; 
  readonly resync: boolean; 
  readonly bids: WsLevel_t[]; 
  readonly asks: WsLevel_t[]; 
  readonly nSigFigs?: number; 
  readonly mantissa?: number
};
