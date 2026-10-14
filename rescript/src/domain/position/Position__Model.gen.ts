/* TypeScript file generated from Position__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Position__ModelJS from './Position__Model.res.mjs';

import type {Order__Raw_UserOutcomeBalance_t as LightconeSdk_Order__Raw_UserOutcomeBalance_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

export type Outcome_t = {
  readonly conditionId: number;
  readonly conditionName: string;
  readonly tokenMint: LightconeSdk_Shared_pubkeyStr;
  readonly amount: string;
  readonly usdValue: string
};

export type WalletHolding_t = {
  readonly tokenMint: LightconeSdk_Shared_pubkeyStr;
  readonly symbol: string;
  readonly amount: string;
  readonly decimals: number;
  readonly usdValue: string;
  readonly imgSrc: string
};

export type t = {
  readonly eventPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly eventName: string;
  readonly eventImgSrc: string;
  readonly outcomes: Outcome_t[];
  readonly totalValue: string;
  readonly createdAt: number
};

export type Portfolio_t = {
  readonly userAddress: LightconeSdk_Shared_pubkeyStr;
  readonly walletHoldings: WalletHolding_t[];
  readonly positions: t[];
  readonly totalWalletValue: string;
  readonly totalPositionsValue: string
};

export type DepositAssetMetadata_t = {
  readonly symbol: string;
  readonly shortSymbol: string;
  readonly name: string;
  readonly depositAsset: LightconeSdk_Shared_pubkeyStr;
  readonly iconUrlLow: string;
  readonly iconUrlMedium: string;
  readonly iconUrlHigh: string;
  readonly description?: string;
  readonly decimals: number
};

export type TokenBalance_kind =
    "DepositAsset"
  | { TAG: "ConditionalToken"; readonly orderbookId: LightconeSdk_Shared_orderBookId; readonly marketPubkey: LightconeSdk_Shared_pubkeyStr; readonly outcomeIndex: number };

export type TokenBalance_t = {
  readonly mint: LightconeSdk_Shared_pubkeyStr;
  readonly idle: string;
  readonly onBook: string;
  readonly tokenType: TokenBalance_kind
};

export type TokenBalance_computedBase = {
  readonly value: string;
  readonly size: string;
  readonly price: string
};

export type ConditionalBalanceDelta_t = {
  readonly marketPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly orderbookId?: LightconeSdk_Shared_orderBookId;
  readonly outcomeIndex: number;
  readonly conditionalToken: LightconeSdk_Shared_pubkeyStr;
  readonly idle: string;
  readonly onBook: string
};

export const TokenBalance_computedBase: (_1:TokenBalance_t, conditionalPrice:string) => TokenBalance_computedBase = Position__ModelJS.TokenBalance.computedBase as any;

export const TokenBalance_computedQuote: (_1:TokenBalance_t) => string = Position__ModelJS.TokenBalance.computedQuote as any;

export const ConditionalBalanceDelta_total: (_1:ConditionalBalanceDelta_t) => string = Position__ModelJS.ConditionalBalanceDelta.total as any;

export const ConditionalBalanceDelta_isZero: (_1:ConditionalBalanceDelta_t) => boolean = Position__ModelJS.ConditionalBalanceDelta.isZero as any;

export const ConditionalBalanceDelta_toTokenBalance: (_1:ConditionalBalanceDelta_t) => TokenBalance_t = Position__ModelJS.ConditionalBalanceDelta.toTokenBalance as any;

export const ConditionalBalanceDelta_toUserOutcomeBalance: (_1:ConditionalBalanceDelta_t) => LightconeSdk_Order__Raw_UserOutcomeBalance_t = Position__ModelJS.ConditionalBalanceDelta.toUserOutcomeBalance as any;

export const ConditionalBalanceDelta: {
  toUserOutcomeBalance: (_1:ConditionalBalanceDelta_t) => LightconeSdk_Order__Raw_UserOutcomeBalance_t;
  total: (_1:ConditionalBalanceDelta_t) => string;
  isZero: (_1:ConditionalBalanceDelta_t) => boolean;
  toTokenBalance: (_1:ConditionalBalanceDelta_t) => TokenBalance_t
} = Position__ModelJS.ConditionalBalanceDelta as any;

export const TokenBalance: { computedBase: (_1:TokenBalance_t, conditionalPrice:string) => TokenBalance_computedBase; computedQuote: (_1:TokenBalance_t) => string } = Position__ModelJS.TokenBalance as any;
