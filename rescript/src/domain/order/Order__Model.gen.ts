/* TypeScript file generated from Order__Model.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import * as Order__ModelJS from './Order__Model.res.mjs';

import type {OrderStatus_t as Shared_OrderStatus_t} from '../../../src/Shared.gen.ts';

import type {Side_t as Shared_Side_t} from '../../../src/Shared.gen.ts';

import type {TimeInForce_t as Shared_TimeInForce_t} from '../../../src/Shared.gen.ts';

import type {TriggerType_t as Shared_TriggerType_t} from '../../../src/Shared.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../../src/Shared.gen.ts';

export type Type_t = 
    "limit"
  | "market"
  | "deposit"
  | "merge"
  | "withdraw"
  | "stop_limit"
  | "take_profit_limit";

export type Limit_t = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly txSignature?: string; 
  readonly baseMint: Shared_pubkeyStr; 
  readonly quoteMint: Shared_pubkeyStr; 
  readonly orderHash: string; 
  readonly side: Shared_Side_t; 
  readonly size: string; 
  readonly price: string; 
  readonly filledSize: string; 
  readonly remainingSize: string; 
  readonly createdAt: number; 
  readonly status: Shared_OrderStatus_t; 
  readonly outcomeIndex: number
};

export type Trigger_t = {
  readonly triggerOrderId: string; 
  readonly orderHash: string; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly triggerPrice: string; 
  readonly triggerType: Shared_TriggerType_t; 
  readonly side: Shared_Side_t; 
  readonly amountIn: string; 
  readonly amountOut: string; 
  readonly timeInForce: Shared_TimeInForce_t; 
  readonly createdAt: number
};

export const Trigger_limitPrice: (_1:Trigger_t) => (undefined | string) = Order__ModelJS.Trigger.limitPrice as any;

export const Trigger: { limitPrice: (_1:Trigger_t) => (undefined | string) } = Order__ModelJS.Trigger as any;
