/* TypeScript file generated from Auth.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type AuthMethod_t = "privy" | "lightcone";

export type ChainType_t = "solana" | "ethereum";

export type privyEmbeddedWallet = {
  readonly privyId: string; 
  readonly chain: ChainType_t; 
  readonly address: string
};

export type userPrivyData = { readonly id: string; readonly wallet: privyEmbeddedWallet };

export type xAccountData = {
  readonly userId?: string; 
  readonly username: string; 
  readonly displayName?: string; 
  readonly avatarUrl?: string
};

export type googleAccountData = {
  readonly email: string; 
  readonly name?: string; 
  readonly givenName?: string; 
  readonly familyName?: string; 
  readonly avatarUrl?: string
};

export type userIdentity = 
    { TAG: "Google"; readonly account: googleAccountData; readonly privy: userPrivyData }
  | { TAG: "X"; readonly account: xAccountData; readonly privy: userPrivyData }
  | { TAG: "Wallet"; readonly address: string; readonly chain: ChainType_t; readonly privy?: userPrivyData };

export type user = {
  readonly userId: string; 
  readonly identity: userIdentity; 
  readonly connectedX?: xAccountData
};

export type sessionResponse = {
  readonly user: user; 
  readonly expiresAt: number; 
  readonly authMethod: AuthMethod_t; 
  readonly isBeta: boolean
};

export type signedLogin = {
  readonly message: string; 
  readonly signatureBs58: string; 
  readonly pubkeyBytes: number[]
};
