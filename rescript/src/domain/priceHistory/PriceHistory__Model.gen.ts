/* TypeScript file generated from PriceHistory__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Resolution_t as Shared_Resolution_t} from '../../../src/Shared.gen.ts';

export type LineData_t = { readonly time: number; readonly value: string };

export type OrderbookQuery_t = {
  readonly resolution: Shared_Resolution_t; 
  readonly fromMs?: number; 
  readonly toMs?: number; 
  readonly cursor?: number; 
  readonly limit?: number; 
  readonly includeOhlcv: boolean
};

export type DepositQuery_t = {
  readonly resolution: Shared_Resolution_t; 
  readonly fromMs?: number; 
  readonly toMs?: number; 
  readonly cursor?: number; 
  readonly limit?: number
};
