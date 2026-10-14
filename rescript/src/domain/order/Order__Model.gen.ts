/* TypeScript file generated from Order__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Shared_OrderStatus_t as LightconeSdk_Shared_OrderStatus_t} from './LightconeSdk.gen.ts';

import type {Shared_Side_t as LightconeSdk_Shared_Side_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

export type Type_t = "limit" | "market" | "split" | "merge";

export type Limit_t = {
  readonly marketPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly txSignature?: string;
  readonly baseMint: LightconeSdk_Shared_pubkeyStr;
  readonly quoteMint: LightconeSdk_Shared_pubkeyStr;
  readonly orderHash: string;
  readonly side: LightconeSdk_Shared_Side_t;
  readonly size: string;
  readonly price: string;
  readonly filledSize: string;
  readonly remainingSize: string;
  readonly createdAt: number;
  readonly status: LightconeSdk_Shared_OrderStatus_t;
  readonly outcomeIndex: number
};
