/* TypeScript file generated from Metrics__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Resolution_t as Shared_Resolution_t} from '../../../src/Shared.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../../src/Shared.gen.ts';

export type UniqueTradersHistoryScope_t = 
    "platform"
  | "market"
  | "orderbook"
  | "category"
  | "outcome";

export type DepositTokenVolume_t = {
  readonly depositAsset: Shared_pubkeyStr; 
  readonly symbol?: string; 
  readonly volume24hUsd: string; 
  readonly volume7dUsd: string; 
  readonly volume30dUsd: string; 
  readonly volumeTotalUsd: string; 
  readonly takerBidVolume24hUsd: string; 
  readonly takerBidVolume7dUsd: string; 
  readonly takerBidVolume30dUsd: string; 
  readonly takerBidVolumeTotalUsd: string; 
  readonly takerAskVolume24hUsd: string; 
  readonly takerAskVolume7dUsd: string; 
  readonly takerAskVolume30dUsd: string; 
  readonly takerAskVolumeTotalUsd: string; 
  readonly takerBidAskImbalance24hPct: string; 
  readonly takerBidAskImbalance7dPct: string; 
  readonly takerBidAskImbalance30dPct: string; 
  readonly takerBidAskImbalanceTotalPct: string; 
  readonly volumeShare24hPct: string
};

export type OrderbookTickerEntry_t = {
  readonly orderbookId: Shared_orderBookId; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly outcomeIndex?: number; 
  readonly outcomeName?: string; 
  readonly outcomeNameLong?: string; 
  readonly baseDepositAsset: Shared_pubkeyStr; 
  readonly quoteDepositAsset: Shared_pubkeyStr; 
  readonly bestBid?: string; 
  readonly bestAsk?: string; 
  readonly midpoint?: string; 
  readonly computedAt?: string
};

export type OrderbookTickersResponse_t = { readonly tickers: OrderbookTickerEntry_t[] };

export type Platform_t = {
  readonly volume24hUsd: string; 
  readonly volume7dUsd: string; 
  readonly volume30dUsd: string; 
  readonly volumeTotalUsd: string; 
  readonly takerBidVolume24hUsd: string; 
  readonly takerBidVolume7dUsd: string; 
  readonly takerBidVolume30dUsd: string; 
  readonly takerBidVolumeTotalUsd: string; 
  readonly takerAskVolume24hUsd: string; 
  readonly takerAskVolume7dUsd: string; 
  readonly takerAskVolume30dUsd: string; 
  readonly takerAskVolumeTotalUsd: string; 
  readonly takerBidAskImbalance24hPct: string; 
  readonly takerBidAskImbalance7dPct: string; 
  readonly takerBidAskImbalance30dPct: string; 
  readonly takerBidAskImbalanceTotalPct: string; 
  readonly openInterestUsd: string; 
  readonly fees24hUsd: string; 
  readonly fees7dUsd: string; 
  readonly fees30dUsd: string; 
  readonly uniqueTraders24h: number; 
  readonly uniqueTraders7d: number; 
  readonly uniqueTraders30d: number; 
  readonly activeMarkets: number; 
  readonly activeOrderbooks: number; 
  readonly depositTokenVolumes: DepositTokenVolume_t[]; 
  readonly updatedAt?: string
};

export type MarketVolume_t = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly slug?: string; 
  readonly marketName?: string; 
  readonly category?: string; 
  readonly volume24hUsd: string; 
  readonly volume7dUsd: string; 
  readonly volume30dUsd: string; 
  readonly volumeTotalUsd: string; 
  readonly takerBidVolume24hUsd: string; 
  readonly takerBidVolume7dUsd: string; 
  readonly takerBidVolume30dUsd: string; 
  readonly takerBidVolumeTotalUsd: string; 
  readonly takerAskVolume24hUsd: string; 
  readonly takerAskVolume7dUsd: string; 
  readonly takerAskVolume30dUsd: string; 
  readonly takerAskVolumeTotalUsd: string; 
  readonly takerBidAskImbalance24hPct: string; 
  readonly takerBidAskImbalance7dPct: string; 
  readonly takerBidAskImbalance30dPct: string; 
  readonly takerBidAskImbalanceTotalPct: string; 
  readonly uniqueTraders24h: number; 
  readonly uniqueTraders7d: number; 
  readonly uniqueTraders30d: number; 
  readonly categoryVolumeShare24hPct: string; 
  readonly platformVolumeShare24hPct: string
};

export type Markets_t = { readonly markets: MarketVolume_t[]; readonly total: number };

export type OutcomeVolume_t = {
  readonly outcomeIndex?: number; 
  readonly outcomeName?: string; 
  readonly outcomeNameLong?: string; 
  readonly volume24hUsd: string; 
  readonly volume7dUsd: string; 
  readonly volume30dUsd: string; 
  readonly volumeTotalUsd: string; 
  readonly takerBidVolume24hUsd: string; 
  readonly takerBidVolume7dUsd: string; 
  readonly takerBidVolume30dUsd: string; 
  readonly takerBidVolumeTotalUsd: string; 
  readonly takerAskVolume24hUsd: string; 
  readonly takerAskVolume7dUsd: string; 
  readonly takerAskVolume30dUsd: string; 
  readonly takerAskVolumeTotalUsd: string; 
  readonly takerBidAskImbalance24hPct: string; 
  readonly takerBidAskImbalance7dPct: string; 
  readonly takerBidAskImbalance30dPct: string; 
  readonly takerBidAskImbalanceTotalPct: string; 
  readonly uniqueTraders24h: number; 
  readonly uniqueTraders7d: number; 
  readonly uniqueTraders30d: number; 
  readonly volumeShare24hPct: string
};

export type MarketOrderbookVolume_t = {
  readonly orderbookId: Shared_orderBookId; 
  readonly outcomeIndex?: number; 
  readonly outcomeName?: string; 
  readonly outcomeNameLong?: string; 
  readonly baseDepositAsset: Shared_pubkeyStr; 
  readonly baseDepositSymbol?: string; 
  readonly quoteDepositAsset: Shared_pubkeyStr; 
  readonly quoteDepositSymbol?: string; 
  readonly volume24hUsd: string; 
  readonly volume7dUsd: string; 
  readonly volume30dUsd: string; 
  readonly volumeTotalUsd: string; 
  readonly volume24hBase: string; 
  readonly volume7dBase: string; 
  readonly volume30dBase: string; 
  readonly volumeTotalBase: string; 
  readonly volume24hQuote: string; 
  readonly volume7dQuote: string; 
  readonly volume30dQuote: string; 
  readonly volumeTotalQuote: string; 
  readonly takerBidVolume24hUsd: string; 
  readonly takerBidVolume7dUsd: string; 
  readonly takerBidVolume30dUsd: string; 
  readonly takerBidVolumeTotalUsd: string; 
  readonly takerBidVolume24hBase: string; 
  readonly takerBidVolume7dBase: string; 
  readonly takerBidVolume30dBase: string; 
  readonly takerBidVolumeTotalBase: string; 
  readonly takerBidVolume24hQuote: string; 
  readonly takerBidVolume7dQuote: string; 
  readonly takerBidVolume30dQuote: string; 
  readonly takerBidVolumeTotalQuote: string; 
  readonly takerAskVolume24hUsd: string; 
  readonly takerAskVolume7dUsd: string; 
  readonly takerAskVolume30dUsd: string; 
  readonly takerAskVolumeTotalUsd: string; 
  readonly takerAskVolume24hBase: string; 
  readonly takerAskVolume7dBase: string; 
  readonly takerAskVolume30dBase: string; 
  readonly takerAskVolumeTotalBase: string; 
  readonly takerAskVolume24hQuote: string; 
  readonly takerAskVolume7dQuote: string; 
  readonly takerAskVolume30dQuote: string; 
  readonly takerAskVolumeTotalQuote: string; 
  readonly takerBidAskImbalance24hPct: string; 
  readonly takerBidAskImbalance7dPct: string; 
  readonly takerBidAskImbalance30dPct: string; 
  readonly takerBidAskImbalanceTotalPct: string; 
  readonly volumeShare24hPct: string
};

export type MarketDetail_t = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly slug?: string; 
  readonly marketName?: string; 
  readonly category?: string; 
  readonly volume24hUsd: string; 
  readonly volume7dUsd: string; 
  readonly volume30dUsd: string; 
  readonly volumeTotalUsd: string; 
  readonly takerBidVolume24hUsd: string; 
  readonly takerBidVolume7dUsd: string; 
  readonly takerBidVolume30dUsd: string; 
  readonly takerBidVolumeTotalUsd: string; 
  readonly takerAskVolume24hUsd: string; 
  readonly takerAskVolume7dUsd: string; 
  readonly takerAskVolume30dUsd: string; 
  readonly takerAskVolumeTotalUsd: string; 
  readonly takerBidAskImbalance24hPct: string; 
  readonly takerBidAskImbalance7dPct: string; 
  readonly takerBidAskImbalance30dPct: string; 
  readonly takerBidAskImbalanceTotalPct: string; 
  readonly uniqueTraders24h: number; 
  readonly uniqueTraders7d: number; 
  readonly uniqueTraders30d: number; 
  readonly categoryVolumeShare24hPct: string; 
  readonly platformVolumeShare24hPct: string; 
  readonly outcomeVolumes: OutcomeVolume_t[]; 
  readonly orderbookVolumes: MarketOrderbookVolume_t[]; 
  readonly depositTokenVolumes: DepositTokenVolume_t[]
};

export type OrderbookVolume_t = {
  readonly orderbookId: Shared_orderBookId; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly outcomeIndex?: number; 
  readonly outcomeName?: string; 
  readonly outcomeNameLong?: string; 
  readonly baseDepositAsset: Shared_pubkeyStr; 
  readonly baseDepositSymbol?: string; 
  readonly quoteDepositAsset: Shared_pubkeyStr; 
  readonly quoteDepositSymbol?: string; 
  readonly volume24hUsd: string; 
  readonly volume7dUsd: string; 
  readonly volume30dUsd: string; 
  readonly volumeTotalUsd: string; 
  readonly volume24hBase: string; 
  readonly volume7dBase: string; 
  readonly volume30dBase: string; 
  readonly volumeTotalBase: string; 
  readonly volume24hQuote: string; 
  readonly volume7dQuote: string; 
  readonly volume30dQuote: string; 
  readonly volumeTotalQuote: string; 
  readonly takerBidVolume24hUsd: string; 
  readonly takerBidVolume7dUsd: string; 
  readonly takerBidVolume30dUsd: string; 
  readonly takerBidVolumeTotalUsd: string; 
  readonly takerBidVolume24hBase: string; 
  readonly takerBidVolume7dBase: string; 
  readonly takerBidVolume30dBase: string; 
  readonly takerBidVolumeTotalBase: string; 
  readonly takerBidVolume24hQuote: string; 
  readonly takerBidVolume7dQuote: string; 
  readonly takerBidVolume30dQuote: string; 
  readonly takerBidVolumeTotalQuote: string; 
  readonly takerAskVolume24hUsd: string; 
  readonly takerAskVolume7dUsd: string; 
  readonly takerAskVolume30dUsd: string; 
  readonly takerAskVolumeTotalUsd: string; 
  readonly takerAskVolume24hBase: string; 
  readonly takerAskVolume7dBase: string; 
  readonly takerAskVolume30dBase: string; 
  readonly takerAskVolumeTotalBase: string; 
  readonly takerAskVolume24hQuote: string; 
  readonly takerAskVolume7dQuote: string; 
  readonly takerAskVolume30dQuote: string; 
  readonly takerAskVolumeTotalQuote: string; 
  readonly takerBidAskImbalance24hPct: string; 
  readonly takerBidAskImbalance7dPct: string; 
  readonly takerBidAskImbalance30dPct: string; 
  readonly takerBidAskImbalanceTotalPct: string; 
  readonly uniqueTraders24h: number; 
  readonly uniqueTraders7d: number; 
  readonly uniqueTraders30d: number; 
  readonly marketVolumeShare24hPct: string
};

export type CategoryVolume_t = {
  readonly category: string; 
  readonly volume24hUsd: string; 
  readonly volume7dUsd: string; 
  readonly volume30dUsd: string; 
  readonly volumeTotalUsd: string; 
  readonly takerBidVolume24hUsd: string; 
  readonly takerBidVolume7dUsd: string; 
  readonly takerBidVolume30dUsd: string; 
  readonly takerBidVolumeTotalUsd: string; 
  readonly takerAskVolume24hUsd: string; 
  readonly takerAskVolume7dUsd: string; 
  readonly takerAskVolume30dUsd: string; 
  readonly takerAskVolumeTotalUsd: string; 
  readonly takerBidAskImbalance24hPct: string; 
  readonly takerBidAskImbalance7dPct: string; 
  readonly takerBidAskImbalance30dPct: string; 
  readonly takerBidAskImbalanceTotalPct: string; 
  readonly uniqueTraders24h: number; 
  readonly uniqueTraders7d: number; 
  readonly uniqueTraders30d: number; 
  readonly platformVolumeShare24hPct: string; 
  readonly depositTokenVolumes: DepositTokenVolume_t[]
};

export type Categories_t = { readonly categories: CategoryVolume_t[] };

export type DepositTokens_t = { readonly depositTokens: DepositTokenVolume_t[] };

export type DepositTokenVolumeHistory_token = {
  readonly rank: number; 
  readonly depositAsset: Shared_pubkeyStr; 
  readonly symbol?: string; 
  readonly volumeTotalUsd: string
};

export type DepositTokenVolumeHistory_pointToken = {
  readonly depositAsset: Shared_pubkeyStr; 
  readonly symbol?: string; 
  readonly volumeUsd: string
};

export type DepositTokenVolumeHistory_point = {
  readonly bucketStart: number; 
  readonly bucketStartDate: string; 
  readonly totalVolumeUsd: string; 
  readonly cumulativeVolumeUsd: string; 
  readonly depositTokenVolumes: DepositTokenVolumeHistory_pointToken[]
};

export type DepositTokenVolumeHistory_t = {
  readonly timestamp: number; 
  readonly resolution: Shared_Resolution_t; 
  readonly fromMs: number; 
  readonly toMs: number; 
  readonly volumeTotalUsd: string; 
  readonly totalDays: number; 
  readonly depositTokens: DepositTokenVolumeHistory_token[]; 
  readonly points: DepositTokenVolumeHistory_point[]
};

export type OpenInterestHistory_depositAsset = {
  readonly rank: number; 
  readonly depositAsset: Shared_pubkeyStr; 
  readonly symbol?: string; 
  readonly latestOpenInterestUsd: string; 
  readonly maxOpenInterestUsd: string
};

export type OpenInterestHistory_pointDepositAsset = {
  readonly depositAsset: Shared_pubkeyStr; 
  readonly symbol?: string; 
  readonly openInterestUsd: string
};

export type OpenInterestHistory_point = {
  readonly bucketStart: number; 
  readonly bucketStartDate: string; 
  readonly totalOpenInterestUsd: string; 
  readonly depositAssetOpenInterest: OpenInterestHistory_pointDepositAsset[]
};

export type OpenInterestHistory_t = {
  readonly timestamp: number; 
  readonly resolution: Shared_Resolution_t; 
  readonly fromMs: number; 
  readonly toMs: number; 
  readonly latestOpenInterestUsd: string; 
  readonly totalDays: number; 
  readonly depositAssets: OpenInterestHistory_depositAsset[]; 
  readonly points: OpenInterestHistory_point[]
};

export type UniqueTradersHistory_point = {
  readonly bucketStart: number; 
  readonly bucketStartDate: string; 
  readonly uniqueTraders: number
};

export type UniqueTradersHistory_t = {
  readonly timestamp: number; 
  readonly resolution: Shared_Resolution_t; 
  readonly scope: UniqueTradersHistoryScope_t; 
  readonly scopeKey: string; 
  readonly fromMs: number; 
  readonly toMs: number; 
  readonly latestUniqueTraders: number; 
  readonly totalDays: number; 
  readonly points: UniqueTradersHistory_point[]
};

export type Leaderboard_entry = {
  readonly rank: number; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly slug?: string; 
  readonly marketName?: string; 
  readonly category?: string; 
  readonly volume24hUsd: string; 
  readonly categoryVolumeShare24hPct: string; 
  readonly platformVolumeShare24hPct: string
};

export type Leaderboard_t = { readonly entries: Leaderboard_entry[]; readonly period: string };

export type History_point = { readonly bucketStart: number; readonly volumeUsd: string };

export type History_t = {
  readonly scope: string; 
  readonly scopeKey: string; 
  readonly resolution: Shared_Resolution_t; 
  readonly points: History_point[]
};

export type User_t = {
  readonly walletAddress: Shared_pubkeyStr; 
  readonly totalOutcomesTraded: number; 
  readonly totalVolumeUsd: string; 
  readonly totalReferralsUsed: number
};
