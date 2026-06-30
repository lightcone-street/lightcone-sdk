/* TypeScript file generated from Faucet.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type faucetToken = { readonly symbol: string; readonly amount: number };

export type faucetResponse = {
  readonly signature: string; 
  readonly sol: number; 
  readonly tokens: faucetToken[]
};
