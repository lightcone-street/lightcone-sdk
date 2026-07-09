/* TypeScript file generated from Position__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Position__RawJS from './Position__Raw.res.mjs';

import type {TokenBalance_t as Position__Model_TokenBalance_t} from './Position__Model.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../../src/Shared.gen.ts';

export type OutcomeBalance_t = {
  readonly outcomeIndex: number; 
  readonly conditionalToken: Shared_pubkeyStr; 
  readonly balance: string; 
  readonly balanceIdle: string; 
  readonly balanceOnBook: string
};

export type VaultBalance_t = {
  readonly depositMint: Shared_pubkeyStr; 
  readonly vault: Shared_pubkeyStr; 
  readonly balance: string
};

export type GlobalDeposit_t = {
  readonly depositMint: Shared_pubkeyStr; 
  readonly symbol: string; 
  readonly balance: string
};

export type Entry_t = {
  readonly id: number; 
  readonly positionPubkey: Shared_pubkeyStr; 
  readonly owner: Shared_pubkeyStr; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly outcomes: OutcomeBalance_t[]; 
  readonly vaultBalances: VaultBalance_t[]; 
  readonly createdAt: string; 
  readonly updatedAt: string
};

export type PositionsResponse_t = {
  readonly owner: Shared_pubkeyStr; 
  readonly totalMarkets: number; 
  readonly positions: Entry_t[]; 
  readonly globalDeposits: GlobalDeposit_t[]; 
  readonly decimals: {[id: string]: number}
};

export type MarketPositionsResponse_t = {
  readonly owner: Shared_pubkeyStr; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly positions: Entry_t[]; 
  readonly globalDeposits: GlobalDeposit_t[]; 
  readonly decimals: {[id: string]: number}
};

export type DepositTokenBalance_t = {
  readonly mint: Shared_pubkeyStr; 
  readonly idle: string; 
  readonly symbol: string; 
  readonly name: string; 
  readonly iconUrlLow?: string; 
  readonly iconUrlMedium?: string; 
  readonly iconUrlHigh?: string
};

export const DepositTokenBalance_toTokenBalance: (_1:DepositTokenBalance_t) => Position__Model_TokenBalance_t = Position__RawJS.DepositTokenBalance.toTokenBalance as any;

export const DepositTokenBalance: { toTokenBalance: (_1:DepositTokenBalance_t) => Position__Model_TokenBalance_t } = Position__RawJS.DepositTokenBalance as any;
