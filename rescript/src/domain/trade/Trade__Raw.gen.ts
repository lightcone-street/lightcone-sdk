/* TypeScript file generated from Trade__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Side_t as Shared_Side_t} from '../../../src/Shared.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../../src/Shared.gen.ts';

export type TradeResponse_t = {
  readonly id: number; 
  readonly tradeId: string; 
  readonly orderbookId: Shared_orderBookId; 
  readonly takerPubkey: string; 
  readonly makerPubkey: string; 
  readonly side: Shared_Side_t; 
  readonly size: string; 
  readonly price: string; 
  readonly takerFee?: string; 
  readonly makerFee?: string; 
  readonly executedAt: number
};

export type TradesResponse_t = {
  readonly orderbookId: Shared_orderBookId; 
  readonly trades: TradeResponse_t[]; 
  readonly nextCursor?: number; 
  readonly hasMore: boolean
};

export type MarketTradesResponse_t = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly trades: TradeResponse_t[]; 
  readonly nextCursor?: number; 
  readonly hasMore: boolean
};
