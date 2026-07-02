/* TypeScript file generated from Position.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as PositionJS from './Position.res.mjs';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

import type {userMarketBalance as Order_userMarketBalance} from './Order.gen.ts';

import type {userOutcomeBalance as Order_userOutcomeBalance} from './Order.gen.ts';

export type outcomeBalance = {
  readonly outcomeIndex: number; 
  readonly conditionalToken: Shared_pubkeyStr; 
  readonly balance: string; 
  readonly balanceIdle: string; 
  readonly balanceOnBook: string
};

export type vaultBalance = {
  readonly depositMint: Shared_pubkeyStr; 
  readonly vault: Shared_pubkeyStr; 
  readonly balance: string
};

export type globalDeposit = {
  readonly depositMint: Shared_pubkeyStr; 
  readonly symbol: string; 
  readonly balance: string
};

export type positionEntry = {
  readonly id: number; 
  readonly positionPubkey: Shared_pubkeyStr; 
  readonly owner: Shared_pubkeyStr; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly outcomes: outcomeBalance[]; 
  readonly vaultBalances: vaultBalance[]; 
  readonly createdAt: string; 
  readonly updatedAt: string
};

export type positionsResponse = {
  readonly owner: Shared_pubkeyStr; 
  readonly totalMarkets: number; 
  readonly positions: positionEntry[]; 
  readonly globalDeposits: globalDeposit[]; 
  readonly decimals: {[id: string]: number}
};

export type marketPositionsResponse = {
  readonly owner: Shared_pubkeyStr; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly positions: positionEntry[]; 
  readonly globalDeposits: globalDeposit[]; 
  readonly decimals: {[id: string]: number}
};

export type depositTokenBalance = {
  readonly mint: Shared_pubkeyStr; 
  readonly idle: string; 
  readonly symbol: string; 
  readonly name: string; 
  readonly iconUrlLow?: string; 
  readonly iconUrlMedium?: string; 
  readonly iconUrlHigh?: string
};

export type positionOutcome = {
  readonly conditionId: number; 
  readonly conditionName: string; 
  readonly tokenMint: Shared_pubkeyStr; 
  readonly amount: string; 
  readonly usdValue: string
};

export type walletHolding = {
  readonly tokenMint: Shared_pubkeyStr; 
  readonly symbol: string; 
  readonly amount: string; 
  readonly decimals: number; 
  readonly usdValue: string; 
  readonly imgSrc: string
};

export type position = {
  readonly eventPubkey: Shared_pubkeyStr; 
  readonly eventName: string; 
  readonly eventImgSrc: string; 
  readonly outcomes: positionOutcome[]; 
  readonly totalValue: string; 
  readonly createdAt: number
};

export type portfolio = {
  readonly userAddress: Shared_pubkeyStr; 
  readonly walletHoldings: walletHolding[]; 
  readonly positions: position[]; 
  readonly totalWalletValue: string; 
  readonly totalPositionsValue: string
};

export type tokenBalanceTokenType = 
    "DepositAsset"
  | { TAG: "ConditionalToken"; readonly orderbookId: Shared_orderBookId; readonly marketPubkey: Shared_pubkeyStr; readonly outcomeIndex: number };

export type tokenBalance = {
  readonly mint: Shared_pubkeyStr; 
  readonly idle: string; 
  readonly onBook: string; 
  readonly tokenType: tokenBalanceTokenType
};

export type depositAssetMetadata = {
  readonly symbol: string; 
  readonly shortSymbol: string; 
  readonly name: string; 
  readonly depositAsset: Shared_pubkeyStr; 
  readonly iconUrlLow: string; 
  readonly iconUrlMedium: string; 
  readonly iconUrlHigh: string; 
  readonly description?: string; 
  readonly decimals: number
};

export type ConditionalBalanceDelta_t = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly orderbookId?: Shared_orderBookId; 
  readonly outcomeIndex: number; 
  readonly conditionalToken: Shared_pubkeyStr; 
  readonly idle: string; 
  readonly onBook: string
};

export type tokenBalanceComputedBase = {
  readonly value: string; 
  readonly size: string; 
  readonly price: string
};

export type UserMarketBalanceIndex_conditionalTokenBalanceIndex = {[id: string]: Order_userOutcomeBalance};

export type UserMarketBalanceIndex_depositAssetBalanceIndex = {[id: string]: UserMarketBalanceIndex_conditionalTokenBalanceIndex};

export type UserMarketBalanceIndex_t = {[id: string]: UserMarketBalanceIndex_depositAssetBalanceIndex};

export const ConditionalBalanceDelta_total: (_1:ConditionalBalanceDelta_t) => string = PositionJS.ConditionalBalanceDelta.total as any;

export const ConditionalBalanceDelta_isZero: (_1:ConditionalBalanceDelta_t) => boolean = PositionJS.ConditionalBalanceDelta.isZero as any;

export const tokenBalanceOfConditionalBalanceDelta: (_1:ConditionalBalanceDelta_t) => tokenBalance = PositionJS.tokenBalanceOfConditionalBalanceDelta as any;

export const userOutcomeBalanceOfConditionalBalanceDelta: (_1:ConditionalBalanceDelta_t) => Order_userOutcomeBalance = PositionJS.userOutcomeBalanceOfConditionalBalanceDelta as any;

export const computedBase: (_1:tokenBalance, conditionalPrice:string) => tokenBalanceComputedBase = PositionJS.computedBase as any;

export const computedQuote: (_1:tokenBalance) => string = PositionJS.computedQuote as any;

export const UserMarketBalanceIndex_make: () => UserMarketBalanceIndex_t = PositionJS.UserMarketBalanceIndex.make as any;

export const UserMarketBalanceIndex_get: (_1:UserMarketBalanceIndex_t, marketPubkey:Shared_pubkeyStr) => (undefined | UserMarketBalanceIndex_depositAssetBalanceIndex) = PositionJS.UserMarketBalanceIndex.get as any;

export const UserMarketBalanceIndex_insert: (_1:UserMarketBalanceIndex_t, marketPubkey:Shared_pubkeyStr, _3:UserMarketBalanceIndex_depositAssetBalanceIndex) => void = PositionJS.UserMarketBalanceIndex.insert as any;

export const UserMarketBalanceIndex_remove: (_1:UserMarketBalanceIndex_t, marketPubkey:Shared_pubkeyStr) => void = PositionJS.UserMarketBalanceIndex.remove as any;

export const UserMarketBalanceIndex_extend: (_1:UserMarketBalanceIndex_t, _2:UserMarketBalanceIndex_t) => void = PositionJS.UserMarketBalanceIndex.extend as any;

export const UserMarketBalanceIndex_marketPubkeys: (_1:UserMarketBalanceIndex_t) => Shared_pubkeyStr[] = PositionJS.UserMarketBalanceIndex.marketPubkeys as any;

export const UserMarketBalanceIndex_ofMarketBalance: (_1:Order_userMarketBalance) => (undefined | UserMarketBalanceIndex_t) = PositionJS.UserMarketBalanceIndex.ofMarketBalance as any;

export const UserMarketBalanceIndex_ofMarketBalances: (_1:Order_userMarketBalance[]) => UserMarketBalanceIndex_t = PositionJS.UserMarketBalanceIndex.ofMarketBalances as any;

export const ConditionalBalanceDelta: { total: (_1:ConditionalBalanceDelta_t) => string; isZero: (_1:ConditionalBalanceDelta_t) => boolean } = PositionJS.ConditionalBalanceDelta as any;

export const UserMarketBalanceIndex: {
  extend: (_1:UserMarketBalanceIndex_t, _2:UserMarketBalanceIndex_t) => void; 
  insert: (_1:UserMarketBalanceIndex_t, marketPubkey:Shared_pubkeyStr, _3:UserMarketBalanceIndex_depositAssetBalanceIndex) => void; 
  ofMarketBalances: (_1:Order_userMarketBalance[]) => UserMarketBalanceIndex_t; 
  get: (_1:UserMarketBalanceIndex_t, marketPubkey:Shared_pubkeyStr) => (undefined | UserMarketBalanceIndex_depositAssetBalanceIndex); 
  remove: (_1:UserMarketBalanceIndex_t, marketPubkey:Shared_pubkeyStr) => void; 
  marketPubkeys: (_1:UserMarketBalanceIndex_t) => Shared_pubkeyStr[]; 
  make: () => UserMarketBalanceIndex_t; 
  ofMarketBalance: (_1:Order_userMarketBalance) => (undefined | UserMarketBalanceIndex_t)
} = PositionJS.UserMarketBalanceIndex as any;
