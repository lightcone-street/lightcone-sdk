/* TypeScript file generated from Orderbook__Raw.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {ProtocolInt_U64_t as LightconeSdk_ProtocolInt_U64_t} from './LightconeSdk.gen.ts';

import type {Scaling_TradingRules_t as LightconeSdk_Scaling_TradingRules_t} from './LightconeSdk.gen.ts';

import type {Shared_Side_t as LightconeSdk_Shared_Side_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

export type WsLevel_t = {
  readonly side: LightconeSdk_Shared_Side_t;
  readonly price: string;
  readonly size: string;
  readonly quoteNotional: string
};

export type RestLevel_t = {
  readonly price: string;
  readonly size: string;
  readonly orders?: number
};

export type DepthDecimals_t = { readonly price: number; readonly size: number };

export type DepthResponse_t = {
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly marketPubkey?: string;
  readonly bestBid?: string;
  readonly bestAsk?: string;
  readonly spread?: string;
  readonly tickSize?: string;
  readonly bids: RestLevel_t[];
  readonly asks: RestLevel_t[];
  readonly decimals: DepthDecimals_t;
  readonly priceQuantum: string;
  readonly tradingRules: LightconeSdk_Scaling_TradingRules_t;
  readonly bidsTruncated: boolean;
  readonly asksTruncated: boolean;
  readonly revision: LightconeSdk_ProtocolInt_U64_t;
  readonly capturedAtMs: LightconeSdk_ProtocolInt_U64_t
};

export type Book_t = {
  readonly id: LightconeSdk_Shared_orderBookId;
  readonly isSnapshot: boolean;
  readonly seq: LightconeSdk_ProtocolInt_U64_t;
  readonly bidsTruncated: boolean;
  readonly asksTruncated: boolean;
  readonly timestamp?: string;
  readonly resync: boolean;
  readonly bids: WsLevel_t[];
  readonly asks: WsLevel_t[];
  readonly nSigFigs?: number;
  readonly mantissa?: number
};
