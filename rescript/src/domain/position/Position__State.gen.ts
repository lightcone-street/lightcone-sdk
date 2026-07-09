/* TypeScript file generated from Position__State.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Position__StateJS from './Position__State.res.mjs';

import type {UserMarketBalance_t as Order__Raw_UserMarketBalance_t} from '../../../src/domain/order/Order__Raw.gen.ts';

import type {UserOutcomeBalance_t as Order__Raw_UserOutcomeBalance_t} from '../../../src/domain/order/Order__Raw.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../../src/Shared.gen.ts';

export type conditionalTokenBalanceIndex = {[id: string]: Order__Raw_UserOutcomeBalance_t};

export type depositAssetBalanceIndex = {[id: string]: conditionalTokenBalanceIndex};

export type t = {[id: string]: depositAssetBalanceIndex};

export const make: () => t = Position__StateJS.make as any;

export const get: (_1:t, marketPubkey:Shared_pubkeyStr) => (undefined | depositAssetBalanceIndex) = Position__StateJS.get as any;

export const insert: (_1:t, marketPubkey:Shared_pubkeyStr, _3:depositAssetBalanceIndex) => void = Position__StateJS.insert as any;

export const remove: (_1:t, marketPubkey:Shared_pubkeyStr) => void = Position__StateJS.remove as any;

export const extend: (_1:t, _2:t) => void = Position__StateJS.extend as any;

export const marketPubkeys: (_1:t) => Shared_pubkeyStr[] = Position__StateJS.marketPubkeys as any;

export const fromMarketBalance: (_1:Order__Raw_UserMarketBalance_t) => (undefined | t) = Position__StateJS.fromMarketBalance as any;

export const fromMarketBalances: (_1:Order__Raw_UserMarketBalance_t[]) => t = Position__StateJS.fromMarketBalances as any;
