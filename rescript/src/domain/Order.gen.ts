/* TypeScript file generated from Order.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {DepositSource_t as Shared_DepositSource_t} from '../../src/Shared.gen.ts';

import type {Side_t as Shared_Side_t} from '../../src/Shared.gen.ts';

import type {TimeInForce_t as Shared_TimeInForce_t} from '../../src/Shared.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

export type fillInfo = {
  readonly counterparty: Shared_pubkeyStr; 
  readonly counterpartyOrderHash: string; 
  readonly fillAmount: string; 
  readonly price: string; 
  readonly isMaker: boolean
};

export type SubmitOrderStatus_t = "accepted" | "partial_fill" | "filled";

export type submitOrderResponse = {
  readonly orderHash: string; 
  readonly status: SubmitOrderStatus_t; 
  readonly remaining: string; 
  readonly filled: string; 
  readonly fills: fillInfo[]
};

export type cancelSuccess = { readonly orderHash: string; readonly remaining: string };

export type cancelAllSuccess = {
  readonly cancelledOrderHashes: string[]; 
  readonly count: number; 
  readonly userPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly message: string
};

export type submitOrderRequest = {
  readonly maker: string; 
  readonly nonce: bigint; 
  readonly salt: bigint; 
  readonly marketPubkey: string; 
  readonly baseToken: string; 
  readonly quoteToken: string; 
  readonly side: number; 
  readonly amountIn: bigint; 
  readonly amountOut: bigint; 
  readonly expiration: bigint; 
  readonly signatureHex: string; 
  readonly orderbookId: string; 
  readonly timeInForce?: Shared_TimeInForce_t; 
  readonly depositSource?: Shared_DepositSource_t
};

export type cancelBody = {
  readonly orderHash: string; 
  readonly maker: string; 
  readonly signatureHex: string
};

export type cancelAllBody = {
  readonly userPubkey: string; 
  readonly orderbookId: string; 
  readonly signatureHex: string; 
  readonly timestamp: number; 
  readonly salt: string
};

export type userSnapshotOrder = {
  readonly orderType: string; 
  readonly orderHash: string; 
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly orderbookId: Shared_orderBookId; 
  readonly side: Shared_Side_t; 
  readonly amountIn: string; 
  readonly amountOut: string; 
  readonly remaining: string; 
  readonly filled: string; 
  readonly price: string; 
  readonly createdAt: number; 
  readonly expiration: number; 
  readonly baseMint: Shared_pubkeyStr; 
  readonly quoteMint: Shared_pubkeyStr; 
  readonly outcomeIndex: number; 
  readonly txSignature?: string; 
  readonly triggerOrderId?: string; 
  readonly triggerPrice?: string
};

export type userOutcomeBalance = {
  readonly outcomeIndex: number; 
  readonly conditionalToken: Shared_pubkeyStr; 
  readonly balance: string; 
  readonly balanceIdle: string; 
  readonly balanceOnBook: string
};

export type userDepositAssetBalance = { readonly depositAsset: Shared_pubkeyStr; readonly outcomes: userOutcomeBalance[] };

export type userMarketBalance = { readonly marketPubkey: Shared_pubkeyStr; readonly depositAssets: userDepositAssetBalance[] };

export type userOrdersResponse = {
  readonly userPubkey: Shared_pubkeyStr; 
  readonly orders: userSnapshotOrder[]; 
  readonly marketBalances: userMarketBalance[]; 
  readonly nextCursor?: string; 
  readonly hasMore: boolean
};
