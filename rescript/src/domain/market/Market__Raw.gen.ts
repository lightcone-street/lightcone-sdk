/* TypeScript file generated from Market__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Status_t as Market__Model_Status_t} from './Market__Model.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../../src/Shared.gen.ts';

export type ConditionalTokenResponse_t = {
  readonly id: number; 
  readonly outcomeIndex: number; 
  readonly tokenAddress: string; 
  readonly symbol?: string; 
  readonly uri?: string; 
  readonly outcome?: string; 
  readonly depositSymbol?: string; 
  readonly shortSymbol?: string; 
  readonly description?: string; 
  readonly iconUrlLow?: string; 
  readonly iconUrlMedium?: string; 
  readonly iconUrlHigh?: string; 
  readonly metadataUri?: string; 
  readonly decimals?: number; 
  readonly createdAt: string
};

export type DepositAssetResponse_t = {
  readonly displayName?: string; 
  readonly tokenSymbol?: string; 
  readonly symbol?: string; 
  readonly depositAsset: Shared_pubkeyStr; 
  readonly id: number; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly vault: string; 
  readonly numOutcomes: number; 
  readonly description?: string; 
  readonly iconUrlLow?: string; 
  readonly iconUrlMedium?: string; 
  readonly iconUrlHigh?: string; 
  readonly metadataUri?: string; 
  readonly decimals?: number; 
  readonly minOrderSize?: string; 
  readonly conditionalMints: ConditionalTokenResponse_t[]; 
  readonly createdAt: string
};

export type DepositMintsResponse_t = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly depositAssets: DepositAssetResponse_t[]; 
  readonly total: number
};

export type SearchOrderbook_t = {
  readonly orderbookId: Shared_orderBookId; 
  readonly outcomeName: string; 
  readonly outcomeNameLong?: string; 
  readonly outcomeIndex: number; 
  readonly depositBaseAsset: Shared_pubkeyStr; 
  readonly depositQuoteAsset: Shared_pubkeyStr; 
  readonly depositBaseSymbol: string; 
  readonly depositQuoteSymbol: string; 
  readonly baseIconUrlLow?: string; 
  readonly baseIconUrlMedium?: string; 
  readonly baseIconUrlHigh?: string; 
  readonly quoteIconUrlLow?: string; 
  readonly quoteIconUrlMedium?: string; 
  readonly quoteIconUrlHigh?: string; 
  readonly conditionalBaseMint: Shared_pubkeyStr; 
  readonly conditionalQuoteMint: Shared_pubkeyStr; 
  readonly outcomeIconUrlLow?: string; 
  readonly outcomeIconUrlMedium?: string; 
  readonly outcomeIconUrlHigh?: string; 
  readonly conditionalBaseSymbol?: string; 
  readonly conditionalQuoteSymbol?: string; 
  readonly latestMidPrice?: string
};

export type MarketSearchResult_t = {
  readonly slug: string; 
  readonly marketName: string; 
  readonly marketStatus: Market__Model_Status_t; 
  readonly category?: string; 
  readonly tags: string[]; 
  readonly featuredRank: number; 
  readonly description?: string; 
  readonly iconUrlLow?: string; 
  readonly iconUrlMedium?: string; 
  readonly iconUrlHigh?: string; 
  readonly orderbooks: SearchOrderbook_t[]
};
