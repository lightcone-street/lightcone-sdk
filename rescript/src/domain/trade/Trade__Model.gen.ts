/* TypeScript file generated from Trade__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Shared_Side_t as LightconeSdk_Shared_Side_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

export type t = {
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly tradeId: string;
  readonly cursorId?: number;
  readonly timestamp: number;
  readonly price: string;
  readonly size: string;
  readonly side: LightconeSdk_Shared_Side_t;
  readonly sequence: number
};

export type Page_t = {
  readonly trades: t[];
  readonly nextCursor?: number;
  readonly hasMore: boolean
};
