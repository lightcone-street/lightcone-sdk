/* TypeScript file generated from PriceHistory__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Shared_Resolution_t as LightconeSdk_Shared_Resolution_t} from './LightconeSdk.gen.ts';

export type LineData_t = { readonly time: number; readonly value: string };

export type OrderbookQuery_t = {
  readonly resolution: LightconeSdk_Shared_Resolution_t;
  readonly fromMs?: number;
  readonly toMs?: number;
  readonly cursor?: number;
  readonly limit?: number;
  readonly includeOhlcv: boolean
};

export type DepositQuery_t = {
  readonly resolution: LightconeSdk_Shared_Resolution_t;
  readonly fromMs?: number;
  readonly toMs?: number;
  readonly cursor?: number;
  readonly limit?: number
};
