/* TypeScript file generated from Position__State.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Position__StateJS from './Position__State.res.mjs';

import type {Order__Raw_UserMarketBalance_t as LightconeSdk_Order__Raw_UserMarketBalance_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_UserOutcomeBalance_t as LightconeSdk_Order__Raw_UserOutcomeBalance_t} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

export type conditionalTokenBalanceIndex = {[id: string]: LightconeSdk_Order__Raw_UserOutcomeBalance_t};

export type depositAssetBalanceIndex = {[id: string]: conditionalTokenBalanceIndex};

export type t = {[id: string]: depositAssetBalanceIndex};

export const make: () => t = Position__StateJS.make as any;

export const get: (_1:t, marketPubkey:LightconeSdk_Shared_pubkeyStr) => (undefined | depositAssetBalanceIndex) = Position__StateJS.get as any;

export const insert: (_1:t, marketPubkey:LightconeSdk_Shared_pubkeyStr, _3:depositAssetBalanceIndex) => void = Position__StateJS.insert as any;

export const remove: (_1:t, marketPubkey:LightconeSdk_Shared_pubkeyStr) => void = Position__StateJS.remove as any;

export const extend: (_1:t, _2:t) => void = Position__StateJS.extend as any;

export const marketPubkeys: (_1:t) => LightconeSdk_Shared_pubkeyStr[] = Position__StateJS.marketPubkeys as any;

export const fromMarketBalance: (_1:LightconeSdk_Order__Raw_UserMarketBalance_t) => (undefined | t) = Position__StateJS.fromMarketBalance as any;

export const fromMarketBalances: (_1:LightconeSdk_Order__Raw_UserMarketBalance_t[]) => t = Position__StateJS.fromMarketBalances as any;
