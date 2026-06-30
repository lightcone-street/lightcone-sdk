/* TypeScript file generated from Metrics.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Resolution_t as Shared_Resolution_t} from '../../src/Shared.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

export type UniqueTradersHistoryScope_t = 
    "platform"
  | "market"
  | "orderbook"
  | "category"
  | "outcome";

export type depositTokenVolumeMetrics = {
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

export type orderbookTickerEntry = {
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

export type orderbookTickersResponse = { readonly tickers: orderbookTickerEntry[] };

export type platformMetrics = {
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
  readonly depositTokenVolumes: depositTokenVolumeMetrics[]; 
  readonly updatedAt?: string
};

export type marketVolumeMetrics = {
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

export type marketsMetrics = { readonly markets: marketVolumeMetrics[]; readonly total: number };

export type outcomeVolumeMetrics = {
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

export type marketOrderbookVolumeMetrics = {
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

export type marketDetailMetrics = {
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
  readonly outcomeVolumes: outcomeVolumeMetrics[]; 
  readonly orderbookVolumes: marketOrderbookVolumeMetrics[]; 
  readonly depositTokenVolumes: depositTokenVolumeMetrics[]
};

export type orderbookVolumeMetrics = {
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

export type categoryVolumeMetrics = {
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
  readonly depositTokenVolumes: depositTokenVolumeMetrics[]
};

export type categoriesMetrics = { readonly categories: categoryVolumeMetrics[] };

export type depositTokensMetrics = { readonly depositTokens: depositTokenVolumeMetrics[] };

export type depositTokenVolumeHistoryToken = {
  readonly rank: number; 
  readonly depositAsset: Shared_pubkeyStr; 
  readonly symbol?: string; 
  readonly volumeTotalUsd: string
};

export type depositTokenVolumeHistoryPointToken = {
  readonly depositAsset: Shared_pubkeyStr; 
  readonly symbol?: string; 
  readonly volumeUsd: string
};

export type depositTokenVolumeHistoryPoint = {
  readonly bucketStart: number; 
  readonly bucketStartDate: string; 
  readonly totalVolumeUsd: string; 
  readonly cumulativeVolumeUsd: string; 
  readonly depositTokenVolumes: depositTokenVolumeHistoryPointToken[]
};

export type depositTokenVolumeHistory = {
  readonly timestamp: number; 
  readonly resolution: Shared_Resolution_t; 
  readonly fromMs: number; 
  readonly toMs: number; 
  readonly volumeTotalUsd: string; 
  readonly totalDays: number; 
  readonly depositTokens: depositTokenVolumeHistoryToken[]; 
  readonly points: depositTokenVolumeHistoryPoint[]
};

export type openInterestHistoryDepositAsset = {
  readonly rank: number; 
  readonly depositAsset: Shared_pubkeyStr; 
  readonly symbol?: string; 
  readonly latestOpenInterestUsd: string; 
  readonly maxOpenInterestUsd: string
};

export type openInterestHistoryPointDepositAsset = {
  readonly depositAsset: Shared_pubkeyStr; 
  readonly symbol?: string; 
  readonly openInterestUsd: string
};

export type openInterestHistoryPoint = {
  readonly bucketStart: number; 
  readonly bucketStartDate: string; 
  readonly totalOpenInterestUsd: string; 
  readonly depositAssetOpenInterest: openInterestHistoryPointDepositAsset[]
};

export type openInterestHistory = {
  readonly timestamp: number; 
  readonly resolution: Shared_Resolution_t; 
  readonly fromMs: number; 
  readonly toMs: number; 
  readonly latestOpenInterestUsd: string; 
  readonly totalDays: number; 
  readonly depositAssets: openInterestHistoryDepositAsset[]; 
  readonly points: openInterestHistoryPoint[]
};

export type uniqueTradersHistoryPoint = {
  readonly bucketStart: number; 
  readonly bucketStartDate: string; 
  readonly uniqueTraders: number
};

export type uniqueTradersHistory = {
  readonly timestamp: number; 
  readonly resolution: Shared_Resolution_t; 
  readonly scope: UniqueTradersHistoryScope_t; 
  readonly scopeKey: string; 
  readonly fromMs: number; 
  readonly toMs: number; 
  readonly latestUniqueTraders: number; 
  readonly totalDays: number; 
  readonly points: uniqueTradersHistoryPoint[]
};

export type leaderboardEntry = {
  readonly rank: number; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly slug?: string; 
  readonly marketName?: string; 
  readonly category?: string; 
  readonly volume24hUsd: string; 
  readonly categoryVolumeShare24hPct: string; 
  readonly platformVolumeShare24hPct: string
};

export type leaderboard = { readonly entries: leaderboardEntry[]; readonly period: string };

export type historyPoint = { readonly bucketStart: number; readonly volumeUsd: string };

export type metricsHistory = {
  readonly scope: string; 
  readonly scopeKey: string; 
  readonly resolution: Shared_Resolution_t; 
  readonly points: historyPoint[]
};

export type userMetrics = {
  readonly walletAddress: Shared_pubkeyStr; 
  readonly totalOutcomesTraded: number; 
  readonly totalVolumeUsd: string; 
  readonly totalReferralsUsed: number
};
