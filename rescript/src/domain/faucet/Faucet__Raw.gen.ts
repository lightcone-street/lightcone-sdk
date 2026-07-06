/* TypeScript file generated from Faucet__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type Token_t = { readonly symbol: string; readonly amount: number };

export type Response_t = {
  readonly signature: string; 
  readonly sol: number; 
  readonly tokens: Token_t[]
};
