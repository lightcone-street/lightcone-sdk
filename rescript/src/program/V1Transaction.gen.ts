/* TypeScript file generated from V1Transaction.resi by genType. */

/* eslint-disable */
/* tslint:disable */

export type Resources_t = {
  readonly computeUnitLimit: number;
  readonly loadedAccountsDataSizeLimit: number;
  readonly priorityFeeLamports: bigint;
  readonly heapSize?: number
};

export type Context_t = {
  readonly blockhash: string;
  readonly lastValidBlockHeight: bigint;
  readonly resources: Resources_t
};

export abstract class t { protected opaque!: any }; /* simulate opaque types */
