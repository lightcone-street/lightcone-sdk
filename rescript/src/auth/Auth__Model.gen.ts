/* TypeScript file generated from Auth__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Auth__ModelJS from './Auth__Model.res.mjs';

import type {SdkError_LinkedIdentityType_t as LightconeSdk_SdkError_LinkedIdentityType_t} from './LightconeSdk.gen.ts';

import type {SdkError_t as LightconeSdk_SdkError_t} from './LightconeSdk.gen.ts';

export type Method_t = "privy" | "lightcone";

export type ChainType_t = "solana" | "ethereum";

export type PrivyEmbeddedWallet_t = {
  readonly privyId: string;
  readonly chain: ChainType_t;
  readonly address: string
};

export type PrivyData_t = { readonly id: string; readonly wallet: PrivyEmbeddedWallet_t };

export type XAccount_t = {
  readonly userId?: string;
  readonly username: string;
  readonly displayName?: string;
  readonly avatarUrl?: string
};

export type GoogleAccount_t = {
  readonly email: string;
  readonly name?: string;
  readonly givenName?: string;
  readonly familyName?: string;
  readonly avatarUrl?: string
};

export type EmailAccount_t = { readonly email: string };

export type LinkedIdentity_t =
    { TAG: "Email"; _0: EmailAccount_t }
  | { TAG: "Google"; _0: GoogleAccount_t }
  | { TAG: "X"; _0: XAccount_t }
  | { TAG: "Wallet"; readonly address: string; readonly chain: ChainType_t };

export type LinkedIdentitySelector_t =
    { TAG: "Email"; _0: string }
  | { TAG: "Google"; _0: string }
  | { TAG: "X"; _0: string }
  | { TAG: "Wallet"; readonly address: string; readonly chain: ChainType_t };

export type RegisterPrivyRequest_t = { readonly attemptedIdentity: LinkedIdentitySelector_t };

export type Credentials_t = {
  readonly userId: string;
  readonly walletAddress: string;
  readonly expiresAt: number
};

export type RegisterPrivyConflict_t =
    "MultipleAccounts"
  | { TAG: "IdentityOwned"; _0: (undefined | LightconeSdk_SdkError_LinkedIdentityType_t) }
  | { TAG: "WalletOwned"; _0: (undefined | LightconeSdk_SdkError_LinkedIdentityType_t) };

export type Identity_t =
    { TAG: "Email"; readonly account: EmailAccount_t; readonly privy: PrivyData_t }
  | { TAG: "Google"; readonly account: GoogleAccount_t; readonly privy: PrivyData_t }
  | { TAG: "X"; readonly account: XAccount_t; readonly privy: PrivyData_t }
  | { TAG: "Wallet"; readonly address: string; readonly chain: ChainType_t; readonly privy?: PrivyData_t };

export type User_t = {
  readonly userId: string;
  readonly identity: Identity_t;
  readonly connectedX?: XAccount_t;
  readonly linkedIdentities: LinkedIdentity_t[];
  readonly maxSlippagePreference?: string;
  readonly telegramInviteUrl?: string
};

export type Session_t = {
  readonly user: User_t;
  readonly expiresAt: number;
  readonly authMethod: Method_t;
  readonly isBeta: boolean
};

export type SignedLogin_t = {
  readonly message: string;
  readonly signatureBs58: string;
  readonly pubkeyBytes: number[]
};

export const tradingWallet: (_1:User_t, _2:Method_t) => string = Auth__ModelJS.tradingWallet as any;

export const credentialsFromSession: (_1:Session_t) => Credentials_t = Auth__ModelJS.credentialsFromSession as any;

export const walletDisplayName: (_1:User_t, _2:Method_t) => string = Auth__ModelJS.walletDisplayName as any;

export const displayName: (_1:User_t) => string = Auth__ModelJS.displayName as any;

export const avatarUrl: (_1:User_t) => (undefined | string) = Auth__ModelJS.avatarUrl as any;

export const classifyRegisterPrivyConflict: (_1:LightconeSdk_SdkError_t) => (undefined | RegisterPrivyConflict_t) = Auth__ModelJS.classifyRegisterPrivyConflict as any;
