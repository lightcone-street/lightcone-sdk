/* TypeScript file generated from Market.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

export type Status_t = 
    "Pending"
  | "Active"
  | "Resolved"
  | "Cancelled";

export type ResolutionKind_t = "single_winner" | "scalar";

export type marketResolutionPayout = { readonly outcomeIndex: number; readonly payoutNumerator: number };

export type marketResolutionResponse = {
  readonly kind: ResolutionKind_t; 
  readonly payoutDenominator: number; 
  readonly payouts: marketResolutionPayout[]; 
  readonly singleWinningOutcome?: number
};

export type conditionalTokenResponse = {
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

export type depositAssetResponse = {
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
  readonly conditionalMints: conditionalTokenResponse[]; 
  readonly createdAt: string
};

export type depositMintsResponse = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly depositAssets: depositAssetResponse[]; 
  readonly total: number
};

export type searchOrderbook = {
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

export type marketSearchResult = {
  readonly slug: string; 
  readonly marketName: string; 
  readonly marketStatus: Status_t; 
  readonly category?: string; 
  readonly tags: string[]; 
  readonly featuredRank: number; 
  readonly description?: string; 
  readonly iconUrlLow?: string; 
  readonly iconUrlMedium?: string; 
  readonly iconUrlHigh?: string; 
  readonly orderbooks: searchOrderbook[]
};

export type outcome = {
  readonly index: number; 
  readonly iconUrlLow: string; 
  readonly iconUrlMedium: string; 
  readonly iconUrlHigh: string; 
  readonly name: string; 
  readonly nameLong?: string
};

export type conditionalToken = {
  readonly id: number; 
  readonly outcomeIndex: number; 
  readonly outcome: string; 
  readonly depositAsset: Shared_pubkeyStr; 
  readonly depositSymbol: string; 
  readonly mint: Shared_pubkeyStr; 
  readonly name: string; 
  readonly symbol: string; 
  readonly shortSymbol: string; 
  readonly description?: string; 
  readonly decimals: number; 
  readonly iconUrlLow: string; 
  readonly iconUrlMedium: string; 
  readonly iconUrlHigh: string
};

export type depositAsset = {
  readonly id: number; 
  readonly marketPda: Shared_pubkeyStr; 
  readonly depositAsset: Shared_pubkeyStr; 
  readonly numOutcomes: number; 
  readonly name: string; 
  readonly symbol: string; 
  readonly shortSymbol: string; 
  readonly description?: string; 
  readonly decimals: number; 
  readonly minOrderSize?: string; 
  readonly iconUrlLow: string; 
  readonly iconUrlMedium: string; 
  readonly iconUrlHigh: string
};

export type tokenMetadata = {
  readonly pubkey: Shared_pubkeyStr; 
  readonly symbol: string; 
  readonly shortSymbol: string; 
  readonly decimals: number; 
  readonly iconUrlLow: string; 
  readonly iconUrlMedium: string; 
  readonly iconUrlHigh: string; 
  readonly name: string
};

export type depositAssetPair = {
  readonly id: string; 
  readonly base: depositAsset; 
  readonly quote: depositAsset
};

export type globalDepositAsset = {
  readonly id: number; 
  readonly depositAsset: Shared_pubkeyStr; 
  readonly name: string; 
  readonly symbol: string; 
  readonly shortSymbol: string; 
  readonly description?: string; 
  readonly decimals: number; 
  readonly iconUrlLow: string; 
  readonly iconUrlMedium: string; 
  readonly iconUrlHigh: string; 
  readonly whitelistIndex: number; 
  readonly active: boolean
};

export type orderBookPair = {
  readonly id: number; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly base: conditionalToken; 
  readonly quote: conditionalToken; 
  readonly outcomeIndex: number; 
  readonly tickSize: number; 
  readonly totalBids: number; 
  readonly totalAsks: number; 
  readonly lastTradePrice?: string; 
  readonly lastTradeTime?: number; 
  readonly active: boolean
};

export type market = {
  readonly id: number; 
  readonly pubkey: Shared_pubkeyStr; 
  readonly name: string; 
  readonly bannerImageUrlLow: string; 
  readonly bannerImageUrlMedium: string; 
  readonly bannerImageUrlHigh: string; 
  readonly iconUrlLow: string; 
  readonly iconUrlMedium: string; 
  readonly iconUrlHigh: string; 
  readonly featuredRank?: number; 
  readonly slug: string; 
  readonly status: Status_t; 
  readonly createdAt: number; 
  readonly activatedAt?: number; 
  readonly settledAt?: number; 
  readonly resolution?: marketResolutionResponse; 
  readonly description: string; 
  readonly definition: string; 
  readonly category?: string; 
  readonly tags: string[]; 
  readonly depositAssets: depositAsset[]; 
  readonly depositAssetPairs: depositAssetPair[]; 
  readonly conditionalTokens: conditionalToken[]; 
  readonly outcomes: outcome[]; 
  readonly orderbookPairs: orderBookPair[]; 
  readonly orderbookIds: Shared_orderBookId[]; 
  readonly tokenMetadata: {[id: string]: tokenMetadata}
};

export type marketsResult = { readonly markets: market[]; readonly validationErrors: string[] };

export type globalDepositAssetsResult = { readonly assets: globalDepositAsset[]; readonly validationErrors: string[] };
