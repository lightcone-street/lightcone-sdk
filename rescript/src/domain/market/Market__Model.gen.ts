/* TypeScript file generated from Market__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Market__ModelJS from './Market__Model.res.mjs';

import type {ProtocolInt_U64_t as LightconeSdk_ProtocolInt_U64_t} from './LightconeSdk.gen.ts';

import type {Shared_Denominator_t as LightconeSdk_Shared_Denominator_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

export type Status_t =
    "Pending"
  | "Active"
  | "Resolved"
  | "Cancelled";

export type Resolution_Kind_t = "single_winner" | "scalar";

export type Resolution_payout = { readonly outcomeIndex: number; readonly payoutNumerator: number };

export type Resolution_t = {
  readonly kind: Resolution_Kind_t;
  readonly payoutDenominator: number;
  readonly payouts: Resolution_payout[];
  readonly singleWinningOutcome?: number
};

export type Outcome_t = {
  readonly index: number;
  readonly iconUrlLow?: string;
  readonly iconUrlMedium?: string;
  readonly iconUrlHigh?: string;
  readonly name: string;
  readonly nameLong?: string
};

export type ConditionalToken_t = {
  readonly id: number;
  readonly outcomeIndex: number;
  readonly outcome: string;
  readonly depositAsset: LightconeSdk_Shared_pubkeyStr;
  readonly depositSymbol: string;
  readonly mint: LightconeSdk_Shared_pubkeyStr;
  readonly name: string;
  readonly symbol: string;
  readonly shortSymbol: string;
  readonly description?: string;
  readonly decimals: number;
  readonly iconUrlLow: string;
  readonly iconUrlMedium: string;
  readonly iconUrlHigh: string
};

export type DepositAsset_t = {
  readonly id: number;
  readonly marketPda: LightconeSdk_Shared_pubkeyStr;
  readonly depositAsset: LightconeSdk_Shared_pubkeyStr;
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

export type TokenMetadata_t = {
  readonly pubkey: LightconeSdk_Shared_pubkeyStr;
  readonly symbol: string;
  readonly shortSymbol: string;
  readonly decimals: number;
  readonly iconUrlLow: string;
  readonly iconUrlMedium: string;
  readonly iconUrlHigh: string;
  readonly name: string
};

export type DepositAssetPair_t = {
  readonly id: string;
  readonly base: DepositAsset_t;
  readonly quote: DepositAsset_t
};

export type GlobalDepositAsset_t = {
  readonly id: number;
  readonly depositAsset: LightconeSdk_Shared_pubkeyStr;
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

export type OrderBookPair_t = {
  readonly id: number;
  readonly marketPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly base: ConditionalToken_t;
  readonly quote: ConditionalToken_t;
  readonly outcomeIndex: number;
  readonly tickSize: number;
  readonly totalBids: number;
  readonly totalAsks: number;
  readonly lastTradePrice?: string;
  readonly lastTradeTime?: number;
  readonly active: boolean
};

export type Impact_t = {
  readonly sign: string;
  readonly pct: number;
  readonly dollar: string;
  readonly isPositive: boolean
};

export type t = {
  readonly id: number;
  readonly pubkey: LightconeSdk_Shared_pubkeyStr;
  readonly name: string;
  readonly bannerImageUrlLow?: string;
  readonly bannerImageUrlMedium?: string;
  readonly bannerImageUrlHigh?: string;
  readonly iconUrlLow: string;
  readonly iconUrlMedium: string;
  readonly iconUrlHigh: string;
  readonly featuredRank?: number;
  readonly slug: string;
  readonly status: Status_t;
  readonly createdAt: number;
  readonly activatedAt?: number;
  readonly settledAt?: number;
  readonly resolution?: Resolution_t;
  readonly description?: string;
  readonly definition: string;
  readonly category?: string;
  readonly subcategory?: string;
  readonly makerFeeBps: number;
  readonly takerFeeBps: number;
  readonly resolutionBy?: bigint;
  readonly numOutcomes: number;
  readonly tags: string[];
  readonly depositAssets: DepositAsset_t[];
  readonly depositAssetPairs: DepositAssetPair_t[];
  readonly conditionalTokens: ConditionalToken_t[];
  readonly outcomes: Outcome_t[];
  readonly orderbookPairs: OrderBookPair_t[];
  readonly orderbookIds: LightconeSdk_Shared_orderBookId[];
  readonly tokenMetadata: {[id: string]: TokenMetadata_t}
};

export type MarketsResult_t = { readonly markets: t[]; readonly validationErrors: string[] };

export type GlobalDepositAssetsResult_t = { readonly assets: GlobalDepositAsset_t[]; readonly validationErrors: string[] };

export type FavoriteMarkets_t = {
  readonly marketPubkeys: string[];
  readonly nextCursor?: LightconeSdk_ProtocolInt_U64_t;
  readonly hasMore: boolean
};

export type FavoriteMarketUpdate_t = { readonly marketPubkey: string; readonly favorited: boolean };

export const usdcMainnet: LightconeSdk_Shared_pubkeyStr = Market__ModelJS.usdcMainnet as any;

export const usdtMainnet: LightconeSdk_Shared_pubkeyStr = Market__ModelJS.usdtMainnet as any;

export const usdcDevnetLc: LightconeSdk_Shared_pubkeyStr = Market__ModelJS.usdcDevnetLc as any;

export const isUsdStablecoin: (_1:LightconeSdk_Shared_pubkeyStr) => boolean = Market__ModelJS.isUsdStablecoin as any;

export const currencySymbol: (_1:LightconeSdk_Shared_pubkeyStr) => string = Market__ModelJS.currencySymbol as any;

export const sortByDisplayPriority: <a>(_1:a[], symbolOf:((_1:a) => string)) => a[] = Market__ModelJS.sortByDisplayPriority as any;

export const ConditionalToken_isUsdStableCoin: (_1:ConditionalToken_t) => boolean = Market__ModelJS.ConditionalToken.isUsdStableCoin as any;

export const ConditionalToken_currencySymbol: (_1:ConditionalToken_t) => string = Market__ModelJS.ConditionalToken.currencySymbol as any;

export const DepositAsset_isUsdStableCoin: (_1:DepositAsset_t) => boolean = Market__ModelJS.DepositAsset.isUsdStableCoin as any;

export const DepositAsset_currencySymbol: (_1:DepositAsset_t) => string = Market__ModelJS.DepositAsset.currencySymbol as any;

export const GlobalDepositAsset_isUsdStableCoin: (_1:GlobalDepositAsset_t) => boolean = Market__ModelJS.GlobalDepositAsset.isUsdStableCoin as any;

export const GlobalDepositAsset_currencySymbol: (_1:GlobalDepositAsset_t) => string = Market__ModelJS.GlobalDepositAsset.currencySymbol as any;

export const OrderBookPair_denominatorToken: (_1:LightconeSdk_Shared_Denominator_t, _2:OrderBookPair_t) => ConditionalToken_t = Market__ModelJS.OrderBookPair.denominatorToken as any;

export const OrderBookPair_denominatorSymbol: (_1:LightconeSdk_Shared_Denominator_t, _2:OrderBookPair_t) => string = Market__ModelJS.OrderBookPair.denominatorSymbol as any;

export const OrderBookPair_denominatorDepositSymbol: (_1:LightconeSdk_Shared_Denominator_t, _2:OrderBookPair_t) => string = Market__ModelJS.OrderBookPair.denominatorDepositSymbol as any;

export const Impact_pct: (depositPrice:string, conditionalPrice:string) => [number, string] = Market__ModelJS.Impact.pct as any;

export const Impact_make: (depositAssetPrice:string, conditionalPrice:string) => Impact_t = Market__ModelJS.Impact.make as any;

export const depositSymbol: (mint:string, symbol:string) => string = Market__ModelJS.depositSymbol as any;

export const ConditionalToken: { currencySymbol: (_1:ConditionalToken_t) => string; isUsdStableCoin: (_1:ConditionalToken_t) => boolean } = Market__ModelJS.ConditionalToken as any;

export const OrderBookPair: {
  denominatorToken: (_1:LightconeSdk_Shared_Denominator_t, _2:OrderBookPair_t) => ConditionalToken_t;
  denominatorSymbol: (_1:LightconeSdk_Shared_Denominator_t, _2:OrderBookPair_t) => string;
  denominatorDepositSymbol: (_1:LightconeSdk_Shared_Denominator_t, _2:OrderBookPair_t) => string
} = Market__ModelJS.OrderBookPair as any;

export const GlobalDepositAsset: { currencySymbol: (_1:GlobalDepositAsset_t) => string; isUsdStableCoin: (_1:GlobalDepositAsset_t) => boolean } = Market__ModelJS.GlobalDepositAsset as any;

export const Impact: { pct: (depositPrice:string, conditionalPrice:string) => [number, string]; make: (depositAssetPrice:string, conditionalPrice:string) => Impact_t } = Market__ModelJS.Impact as any;

export const DepositAsset: { currencySymbol: (_1:DepositAsset_t) => string; isUsdStableCoin: (_1:DepositAsset_t) => boolean } = Market__ModelJS.DepositAsset as any;
