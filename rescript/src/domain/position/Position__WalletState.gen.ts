/* TypeScript file generated from Position__WalletState.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Position__WalletStateJS from './Position__WalletState.res.mjs';

import type {Position__Raw_DepositTokenBalance_t as LightconeSdk_Position__Raw_DepositTokenBalance_t} from './LightconeSdk.gen.ts';

import type {Position__Raw_DepositTokenBalancesSnapshot_t as LightconeSdk_Position__Raw_DepositTokenBalancesSnapshot_t} from './LightconeSdk.gen.ts';

import type {Position__Raw_WalletDepositBalancesEvent_t as LightconeSdk_Position__Raw_WalletDepositBalancesEvent_t} from './LightconeSdk.gen.ts';

import type {SdkError_t as LightconeSdk_SdkError_t} from './LightconeSdk.gen.ts';

export type applyResult = "Applied" | "Ignored" | "Rejected";

export abstract class t { protected opaque!: any }; /* simulate opaque types */

export type solBalanceBreakdown = { readonly nativeLamports: bigint; readonly canonicalWsolLamports: bigint };

export const make: () => t = Position__WalletStateJS.make as any;

export const applyRestSnapshot: (_1:t, walletAddress:string, _3:LightconeSdk_Position__Raw_DepositTokenBalancesSnapshot_t, minimumSnapshotSlot:(undefined | bigint)) => applyResult = Position__WalletStateJS.applyRestSnapshot as any;

export const applyEvent: (_1:t, _2:LightconeSdk_Position__Raw_WalletDepositBalancesEvent_t, minimumSnapshotSlot:(undefined | bigint)) => applyResult = Position__WalletStateJS.applyEvent as any;

export const combinedSolBalance: (_1:t) =>
    { TAG: "Ok"; _0: string }
  | { TAG: "Error"; _0: LightconeSdk_SdkError_t } = Position__WalletStateJS.combinedSolBalance as any;

export const solBalanceBreakdown: (_1:t) =>
    { TAG: "Ok"; _0: solBalanceBreakdown }
  | { TAG: "Error"; _0: LightconeSdk_SdkError_t } = Position__WalletStateJS.solBalanceBreakdown as any;

export const nativeSolLamports: (_1:t) =>
    { TAG: "Ok"; _0: bigint }
  | { TAG: "Error"; _0: LightconeSdk_SdkError_t } = Position__WalletStateJS.nativeSolLamports as any;

export const canonicalWsolLamports: (_1:t) =>
    { TAG: "Ok"; _0: bigint }
  | { TAG: "Error"; _0: LightconeSdk_SdkError_t } = Position__WalletStateJS.canonicalWsolLamports as any;

export const walletAddress: (_1:t) => (undefined | string) = Position__WalletStateJS.walletAddress as any;

export const contextSlot: (_1:t) => (undefined | bigint) = Position__WalletStateJS.contextSlot as any;

export const nativeSolBalance: (_1:t) => (undefined | string) = Position__WalletStateJS.nativeSolBalance as any;

export const balances: (_1:t) => {[id: string]: LightconeSdk_Position__Raw_DepositTokenBalance_t} = Position__WalletStateJS.balances as any;

export const clear: (_1:t) => void = Position__WalletStateJS.clear as any;
