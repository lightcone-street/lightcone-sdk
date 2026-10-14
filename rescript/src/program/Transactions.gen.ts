/* TypeScript file generated from Transactions.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type Simulation_t = {
  readonly slot: bigint;
  readonly unitsConsumed: (undefined | bigint);
  readonly loadedAccountsDataSize: (undefined | bigint);
  readonly logs: string[]
};

export type Confirmed_t = { readonly signature: string; readonly slot: bigint };
