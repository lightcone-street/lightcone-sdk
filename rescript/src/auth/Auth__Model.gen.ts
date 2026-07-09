/* TypeScript file generated from Auth__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

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

export type Identity_t = 
    { TAG: "Google"; readonly account: GoogleAccount_t; readonly privy: PrivyData_t }
  | { TAG: "X"; readonly account: XAccount_t; readonly privy: PrivyData_t }
  | { TAG: "Wallet"; readonly address: string; readonly chain: ChainType_t; readonly privy?: PrivyData_t };

export type User_t = {
  readonly userId: string; 
  readonly identity: Identity_t; 
  readonly connectedX?: XAccount_t
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
