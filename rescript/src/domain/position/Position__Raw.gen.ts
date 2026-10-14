/* TypeScript file generated from Position__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Position__RawJS from './Position__Raw.res.mjs';

import type {Position__Model_TokenBalance_t as LightconeSdk_Position__Model_TokenBalance_t} from './LightconeSdk.gen.ts';

import type {ProtocolInt_U64_t as LightconeSdk_ProtocolInt_U64_t} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

export type OutcomeBalance_t = {
  readonly outcomeIndex: number;
  readonly conditionalToken: LightconeSdk_Shared_pubkeyStr;
  readonly balance: string;
  readonly balanceIdle: string;
  readonly balanceOnBook: string
};

export type VaultBalance_t = {
  readonly depositMint: LightconeSdk_Shared_pubkeyStr;
  readonly vault: LightconeSdk_Shared_pubkeyStr;
  readonly balance: string
};

export type GlobalDeposit_t = {
  readonly depositMint: LightconeSdk_Shared_pubkeyStr;
  readonly symbol: string;
  readonly balance: string
};

export type Entry_t = {
  readonly id: number;
  readonly positionPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly owner: LightconeSdk_Shared_pubkeyStr;
  readonly marketPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly outcomes: OutcomeBalance_t[];
  readonly vaultBalances: VaultBalance_t[];
  readonly createdAt: string;
  readonly updatedAt: string
};

export type PositionsResponse_t = {
  readonly owner: LightconeSdk_Shared_pubkeyStr;
  readonly totalMarkets: number;
  readonly positions: Entry_t[];
  readonly globalDeposits: GlobalDeposit_t[];
  readonly decimals: {[id: string]: number}
};

export type MarketPositionsResponse_t = {
  readonly owner: LightconeSdk_Shared_pubkeyStr;
  readonly marketPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly positions: Entry_t[];
  readonly globalDeposits: GlobalDeposit_t[];
  readonly decimals: {[id: string]: number}
};

export type DepositTokenBalance_t = {
  readonly mint: LightconeSdk_Shared_pubkeyStr;
  readonly idle: string;
  readonly symbol: string;
  readonly name: string;
  readonly iconUrlLow?: string;
  readonly iconUrlMedium?: string;
  readonly iconUrlHigh?: string
};

export type NativeSolBalance_t = string;

export type DepositTokenBalancesSnapshot_t = {
  readonly contextSlot: LightconeSdk_ProtocolInt_U64_t;
  readonly balances: {[id: string]: DepositTokenBalance_t};
  readonly nativeSolBalance: NativeSolBalance_t
};

export type WalletDepositBalanceStatus_t = "reconnecting" | "metadata_unavailable";

export type WalletDepositBalancesEvent_snapshot = {
  readonly walletAddress: string;
  readonly contextSlot: LightconeSdk_ProtocolInt_U64_t;
  readonly balances: {[id: string]: DepositTokenBalance_t};
  readonly nativeSolBalance: NativeSolBalance_t
};

export type WalletDepositBalancesEvent_balanceUpdate = {
  readonly walletAddress: string;
  readonly contextSlot: LightconeSdk_ProtocolInt_U64_t;
  readonly balance: DepositTokenBalance_t
};

export type WalletDepositBalancesEvent_nativeUpdate = {
  readonly walletAddress: string;
  readonly contextSlot: LightconeSdk_ProtocolInt_U64_t;
  readonly nativeSolBalance: NativeSolBalance_t
};

export type WalletDepositBalancesEvent_status = {
  readonly walletAddress: string;
  readonly status: WalletDepositBalanceStatus_t;
  readonly code: string
};

export type WalletDepositBalancesEvent_t =
    { TAG: "Snapshot"; _0: WalletDepositBalancesEvent_snapshot }
  | { TAG: "BalanceUpdate"; _0: WalletDepositBalancesEvent_balanceUpdate }
  | { TAG: "NativeSolBalanceUpdate"; _0: WalletDepositBalancesEvent_nativeUpdate }
  | { TAG: "Status"; _0: WalletDepositBalancesEvent_status };

export const DepositTokenBalance_toTokenBalance: (_1:DepositTokenBalance_t) => LightconeSdk_Position__Model_TokenBalance_t = Position__RawJS.DepositTokenBalance.toTokenBalance as any;

export const DepositTokenBalance: { toTokenBalance: (_1:DepositTokenBalance_t) => LightconeSdk_Position__Model_TokenBalance_t } = Position__RawJS.DepositTokenBalance as any;
