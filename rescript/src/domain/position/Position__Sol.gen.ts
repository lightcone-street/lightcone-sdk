/* TypeScript file generated from Position__Sol.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Position__WalletState_solBalanceBreakdown as LightconeSdk_Position__WalletState_solBalanceBreakdown} from './LightconeSdk.gen.ts';

import type {V1Transaction_t as LightconeSdk_V1Transaction_t} from './LightconeSdk.gen.ts';

export type Kind_t =
    "Split"
  | "Merge"
  | "Redeem"
  | "NativeWithdraw"
  | "Wrap"
  | "UnwrapAll";

export type Costs_t = {
  readonly feeLamports: bigint;
  readonly upfrontRentLamports: bigint;
  readonly createsCanonicalWsolAccount: boolean;
  readonly sponsored: boolean
};

export type Availability_t = {
  readonly breakdown: LightconeSdk_Position__WalletState_solBalanceBreakdown;
  readonly displayedLamports: bigint;
  readonly reserveLamports: bigint;
  readonly spendableLamports: bigint
};

export type Delta_t = { readonly nativeLamports: bigint; readonly canonicalWsolLamports: bigint };

export type Plan_t = {
  readonly kind: Kind_t;
  readonly transaction: LightconeSdk_V1Transaction_t;
  readonly costs: Costs_t;
  readonly availability: Availability_t;
  readonly expectedDelta: Delta_t
};

export type CanonicalAccount_t = {
  readonly accountLamports: bigint;
  readonly tokenAmountLamports: bigint;
  readonly nativeReserveLamports: bigint
};
