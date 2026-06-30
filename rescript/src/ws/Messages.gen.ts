/* TypeScript file generated from Messages.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Resolution_t as Shared_Resolution_t} from '../../src/Shared.gen.ts';

import type {Side_t as Shared_Side_t} from '../../src/Shared.gen.ts';

import type {depositPriceCandle as PriceHistory_depositPriceCandle} from '../../src/domain/PriceHistory.gen.ts';

import type {notification as Notification_notification} from '../../src/domain/Notification.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {orderBook as Orderbook_orderBook} from '../../src/domain/Orderbook.gen.ts';

import type {orderbookPriceCandle as PriceHistory_orderbookPriceCandle} from '../../src/domain/PriceHistory.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

export type wsErrorFrame = {
  readonly error: string; 
  readonly code?: string; 
  readonly orderbookId?: string; 
  readonly nSigFigs?: number; 
  readonly mantissa?: number; 
  readonly walletAddress?: string; 
  readonly depositAsset?: string; 
  readonly hint?: string; 
  readonly details?: string
};

export type wsTrade = {
  readonly orderbookId: Shared_orderBookId; 
  readonly tradeId: string; 
  readonly timestamp: string; 
  readonly price: string; 
  readonly size: string; 
  readonly side: Shared_Side_t; 
  readonly sequence: number
};

export type wsTicker = {
  readonly orderbookId: Shared_orderBookId; 
  readonly bestBid?: string; 
  readonly bestAsk?: string; 
  readonly mid?: string
};

export type MarketEvent_t = 
    { TAG: "Settled"; _0: Shared_pubkeyStr }
  | { TAG: "Created"; _0: Shared_pubkeyStr }
  | { TAG: "Opened"; _0: Shared_pubkeyStr }
  | { TAG: "Paused"; _0: Shared_pubkeyStr }
  | { TAG: "OrderbookCreated"; _0: Shared_pubkeyStr; _1: Shared_orderBookId };

export type AuthUpdate_t = 
    { TAG: "Authenticated"; _0: Shared_pubkeyStr }
  | { TAG: "Anonymous"; _0: (undefined | string) };

export type WsPriceHistory_snapshot = {
  readonly orderbookId: Shared_orderBookId; 
  readonly resolution: Shared_Resolution_t; 
  readonly prices: PriceHistory_orderbookPriceCandle[]; 
  readonly lastTimestamp?: number; 
  readonly serverTime?: number
};

export type WsPriceHistory_update = {
  readonly orderbookId: Shared_orderBookId; 
  readonly resolution: Shared_Resolution_t; 
  readonly candle: PriceHistory_orderbookPriceCandle
};

export type WsPriceHistory_heartbeat = { readonly serverTime: number; readonly lastProcessed?: number };

export type WsPriceHistory_t = 
    { TAG: "Snapshot"; _0: WsPriceHistory_snapshot }
  | { TAG: "Update"; _0: WsPriceHistory_update }
  | { TAG: "Heartbeat"; _0: WsPriceHistory_heartbeat };

export type WsDepositPrice_snapshot = {
  readonly depositAsset: Shared_pubkeyStr; 
  readonly resolution: Shared_Resolution_t; 
  readonly prices: PriceHistory_depositPriceCandle[]
};

export type WsDepositPrice_candle = {
  readonly depositAsset: Shared_pubkeyStr; 
  readonly resolution: Shared_Resolution_t; 
  readonly t: number; 
  readonly tc: number; 
  readonly c: string
};

export type WsDepositPrice_tick = {
  readonly depositAsset: Shared_pubkeyStr; 
  readonly price: string; 
  readonly eventTime: number
};

export type WsDepositPrice_t = 
    { TAG: "Snapshot"; _0: WsDepositPrice_snapshot }
  | { TAG: "Candle"; _0: WsDepositPrice_candle }
  | { TAG: "Price"; _0: WsDepositPrice_tick };

export type WsDepositAssetPrice_snapshot = { readonly depositAsset: Shared_pubkeyStr; readonly price: string };

export type WsDepositAssetPrice_tick = {
  readonly depositAsset: Shared_pubkeyStr; 
  readonly price: string; 
  readonly eventTime: number
};

export type WsDepositAssetPrice_t = 
    { TAG: "Snapshot"; _0: WsDepositAssetPrice_snapshot }
  | { TAG: "Price"; _0: WsDepositAssetPrice_tick };

export type userOutcomeBalance = {
  readonly outcomeIndex: number; 
  readonly conditionalToken: Shared_pubkeyStr; 
  readonly balance: string; 
  readonly balanceIdle: string; 
  readonly balanceOnBook: string
};

export type userDepositAssetBalance = { readonly depositAsset: Shared_pubkeyStr; readonly outcomes: userOutcomeBalance[] };

export type userMarketBalance = { readonly marketPubkey: Shared_pubkeyStr; readonly depositAssets: userDepositAssetBalance[] };

export type userBalanceUpdate = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly marketBalance: userMarketBalance; 
  readonly timestamp: string
};

export type globalDepositUpdate = {
  readonly mint: Shared_pubkeyStr; 
  readonly balance: string; 
  readonly timestamp: string
};

export type nonceUpdate = {
  readonly userPubkey: Shared_pubkeyStr; 
  readonly newNonce: number; 
  readonly timestamp: string
};

export type UserUpdate_t = 
    { TAG: "Snapshot"; _0: unknown }
  | { TAG: "Order"; _0: unknown }
  | { TAG: "BalanceUpdate"; _0: userBalanceUpdate }
  | { TAG: "GlobalDepositUpdate"; _0: globalDepositUpdate }
  | { TAG: "NonceUpdate"; _0: nonceUpdate }
  | { TAG: "NotificationPush"; _0: Notification_notification };

export type kind = 
    "Pong"
  | { TAG: "BookUpdate"; _0: Orderbook_orderBook }
  | { TAG: "Trades"; _0: wsTrade }
  | { TAG: "User"; _0: UserUpdate_t }
  | { TAG: "Ticker"; _0: wsTicker }
  | { TAG: "PriceHistory"; _0: WsPriceHistory_t }
  | { TAG: "Market"; _0: MarketEvent_t }
  | { TAG: "DepositPrice"; _0: WsDepositPrice_t }
  | { TAG: "DepositAssetPrice"; _0: WsDepositAssetPrice_t }
  | { TAG: "Auth"; _0: AuthUpdate_t }
  | { TAG: "ErrorFrame"; _0: wsErrorFrame };

export type messageIn = { readonly kind: kind; readonly version: number };
