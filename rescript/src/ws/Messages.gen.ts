/* TypeScript file generated from Messages.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Book_t as Orderbook__Raw_Book_t} from '../../src/domain/orderbook/Orderbook__Raw.gen.ts';

import type {DepositCandle_t as PriceHistory__Raw_DepositCandle_t} from '../../src/domain/priceHistory/PriceHistory__Raw.gen.ts';

import type {Event_t as Order__Raw_Event_t} from '../../src/domain/order/Order__Raw.gen.ts';

import type {OrderbookCandle_t as PriceHistory__Raw_OrderbookCandle_t} from '../../src/domain/priceHistory/PriceHistory__Raw.gen.ts';

import type {Resolution_t as Shared_Resolution_t} from '../../src/Shared.gen.ts';

import type {Side_t as Shared_Side_t} from '../../src/Shared.gen.ts';

import type {UserMarketBalance_t as Order__Raw_UserMarketBalance_t} from '../../src/domain/order/Order__Raw.gen.ts';

import type {UserSnapshot_t as Order__Raw_UserSnapshot_t} from '../../src/domain/order/Order__Raw.gen.ts';

import type {orderBookId as Shared_orderBookId} from '../../src/Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from '../../src/Shared.gen.ts';

import type {t as Notification__Model_t} from '../../src/domain/notification/Notification__Model.gen.ts';

export type ErrorFrame_t = {
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

export type Trade_t = {
  readonly orderbookId: Shared_orderBookId; 
  readonly tradeId: string; 
  readonly timestamp: string; 
  readonly price: string; 
  readonly size: string; 
  readonly side: Shared_Side_t; 
  readonly sequence: number
};

export type Ticker_t = {
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
  readonly prices: PriceHistory__Raw_OrderbookCandle_t[]; 
  readonly lastTimestamp?: number; 
  readonly serverTime?: number
};

export type WsPriceHistory_update = {
  readonly orderbookId: Shared_orderBookId; 
  readonly resolution: Shared_Resolution_t; 
  readonly candle: PriceHistory__Raw_OrderbookCandle_t
};

export type WsPriceHistory_heartbeat = { readonly serverTime: number; readonly lastProcessed?: number };

export type WsPriceHistory_t = 
    { TAG: "Snapshot"; _0: WsPriceHistory_snapshot }
  | { TAG: "Update"; _0: WsPriceHistory_update }
  | { TAG: "Heartbeat"; _0: WsPriceHistory_heartbeat };

export type WsDepositPrice_snapshot = {
  readonly depositAsset: Shared_pubkeyStr; 
  readonly resolution: Shared_Resolution_t; 
  readonly prices: PriceHistory__Raw_DepositCandle_t[]
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

export type UserBalanceUpdate_t = {
  readonly marketPubkey: Shared_pubkeyStr; 
  readonly marketBalance: Order__Raw_UserMarketBalance_t; 
  readonly timestamp: string
};

export type GlobalDepositUpdate_t = {
  readonly mint: Shared_pubkeyStr; 
  readonly balance: string; 
  readonly timestamp: string
};

export type NonceUpdate_t = {
  readonly userPubkey: Shared_pubkeyStr; 
  readonly newNonce: number; 
  readonly timestamp: string
};

export type UserUpdate_t = 
    { TAG: "Snapshot"; _0: Order__Raw_UserSnapshot_t }
  | { TAG: "Order"; _0: Order__Raw_Event_t }
  | { TAG: "BalanceUpdate"; _0: UserBalanceUpdate_t }
  | { TAG: "GlobalDepositUpdate"; _0: GlobalDepositUpdate_t }
  | { TAG: "NonceUpdate"; _0: NonceUpdate_t }
  | { TAG: "NotificationPush"; _0: Notification__Model_t };

export type Kind_t = 
    "Pong"
  | { TAG: "BookUpdate"; _0: Orderbook__Raw_Book_t }
  | { TAG: "Trades"; _0: Trade_t }
  | { TAG: "User"; _0: UserUpdate_t }
  | { TAG: "Ticker"; _0: Ticker_t }
  | { TAG: "PriceHistory"; _0: WsPriceHistory_t }
  | { TAG: "Market"; _0: MarketEvent_t }
  | { TAG: "DepositPrice"; _0: WsDepositPrice_t }
  | { TAG: "DepositAssetPrice"; _0: WsDepositAssetPrice_t }
  | { TAG: "Auth"; _0: AuthUpdate_t }
  | { TAG: "ErrorFrame"; _0: ErrorFrame_t };

export type t = { readonly kind: Kind_t; readonly version: number };
