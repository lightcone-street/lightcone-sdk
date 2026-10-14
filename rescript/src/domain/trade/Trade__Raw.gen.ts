/* TypeScript file generated from Trade__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Shared_Side_t as LightconeSdk_Shared_Side_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

export type TradeResponse_t = {
  readonly id: number;
  readonly tradeId: string;
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly takerPubkey: string;
  readonly makerPubkey: string;
  readonly side: LightconeSdk_Shared_Side_t;
  readonly size: string;
  readonly price: string;
  readonly takerFee?: string;
  readonly makerFee?: string;
  readonly executedAt: number
};

export type TradesResponse_t = {
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly trades: TradeResponse_t[];
  readonly nextCursor?: number;
  readonly hasMore: boolean
};

export type MarketTradesResponse_t = {
  readonly marketPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly trades: TradeResponse_t[];
  readonly nextCursor?: number;
  readonly hasMore: boolean
};
