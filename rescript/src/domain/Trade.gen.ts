/* TypeScript file generated from Trade.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Side_t as Shared_Side_t} from '../../src/Shared.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

export type trade = {
  readonly orderbookId: Shared_orderBookId; 
  readonly tradeId: string; 
  readonly cursorId?: number; 
  readonly timestamp: number; 
  readonly price: string; 
  readonly size: string; 
  readonly side: Shared_Side_t; 
  readonly sequence: number
};

export type tradesPage = {
  readonly trades: trade[]; 
  readonly nextCursor?: number; 
  readonly hasMore: boolean
};
