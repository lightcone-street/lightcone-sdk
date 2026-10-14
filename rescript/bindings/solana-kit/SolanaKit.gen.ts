/* TypeScript file generated from SolanaKit.res by genType. */

/* eslint-disable */
/* tslint:disable */

export abstract class address { protected opaque!: any }; /* simulate opaque types */

export abstract class cryptoKey { protected opaque!: any }; /* simulate opaque types */

export type cryptoKeyPair = { readonly privateKey: cryptoKey; readonly publicKey: cryptoKey };

export abstract class keyPairSigner { protected opaque!: any }; /* simulate opaque types */

export type accountMeta = { readonly address: address; readonly role: number };

export type instruction = {
  readonly programAddress: address;
  readonly accounts: accountMeta[];
  readonly data: Uint8Array
};
