/* TypeScript file generated from Messages.resi by genType. */

/* eslint-disable */
/* tslint:disable */

import type {Notification__Model_t as LightconeSdk_Notification__Model_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_Event_t as LightconeSdk_Order__Raw_Event_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_UserMarketBalance_t as LightconeSdk_Order__Raw_UserMarketBalance_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_UserSnapshot_t as LightconeSdk_Order__Raw_UserSnapshot_t} from './LightconeSdk.gen.ts';

import type {Orderbook__Raw_Book_t as LightconeSdk_Orderbook__Raw_Book_t} from './LightconeSdk.gen.ts';

import type {Position__Raw_WalletDepositBalancesEvent_t as LightconeSdk_Position__Raw_WalletDepositBalancesEvent_t} from './LightconeSdk.gen.ts';

import type {PriceHistory__Raw_DepositCandle_t as LightconeSdk_PriceHistory__Raw_DepositCandle_t} from './LightconeSdk.gen.ts';

import type {PriceHistory__Raw_OrderbookCandle_t as LightconeSdk_PriceHistory__Raw_OrderbookCandle_t} from './LightconeSdk.gen.ts';

import type {Shared_Resolution_t as LightconeSdk_Shared_Resolution_t} from './LightconeSdk.gen.ts';

import type {Shared_Side_t as LightconeSdk_Shared_Side_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

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
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly tradeId: string;
  readonly timestamp: string;
  readonly price: string;
  readonly size: string;
  readonly side: LightconeSdk_Shared_Side_t;
  readonly sequence: number
};

export type Ticker_t = {
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly bestBid?: string;
  readonly bestAsk?: string;
  readonly mid?: string
};

export type MarketEvent_t =
    { TAG: "Settled"; _0: LightconeSdk_Shared_pubkeyStr }
  | { TAG: "Created"; _0: LightconeSdk_Shared_pubkeyStr }
  | { TAG: "Opened"; _0: LightconeSdk_Shared_pubkeyStr }
  | { TAG: "Paused"; _0: LightconeSdk_Shared_pubkeyStr }
  | { TAG: "OrderbookCreated"; _0: LightconeSdk_Shared_pubkeyStr; _1: LightconeSdk_Shared_orderBookId };

export type AuthUpdate_t =
    { TAG: "Authenticated"; _0: LightconeSdk_Shared_pubkeyStr }
  | { TAG: "Anonymous"; _0: (undefined | string) };

export type WsPriceHistory_snapshot = {
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly resolution: LightconeSdk_Shared_Resolution_t;
  readonly prices: LightconeSdk_PriceHistory__Raw_OrderbookCandle_t[];
  readonly lastTimestamp?: number;
  readonly serverTime?: number
};

export type WsPriceHistory_update = {
  readonly orderbookId: LightconeSdk_Shared_orderBookId;
  readonly resolution: LightconeSdk_Shared_Resolution_t;
  readonly candle: LightconeSdk_PriceHistory__Raw_OrderbookCandle_t
};

export type WsPriceHistory_heartbeat = { readonly serverTime: number; readonly lastProcessed?: number };

export type WsPriceHistory_t =
    { TAG: "Snapshot"; _0: WsPriceHistory_snapshot }
  | { TAG: "Update"; _0: WsPriceHistory_update }
  | { TAG: "Heartbeat"; _0: WsPriceHistory_heartbeat };

export type WsDepositPrice_snapshot = {
  readonly depositAsset: LightconeSdk_Shared_pubkeyStr;
  readonly resolution: LightconeSdk_Shared_Resolution_t;
  readonly prices: LightconeSdk_PriceHistory__Raw_DepositCandle_t[]
};

export type WsDepositPrice_candle = {
  readonly depositAsset: LightconeSdk_Shared_pubkeyStr;
  readonly resolution: LightconeSdk_Shared_Resolution_t;
  readonly t: number;
  readonly tc: number;
  readonly c: string
};

export type WsDepositPrice_tick = {
  readonly depositAsset: LightconeSdk_Shared_pubkeyStr;
  readonly price: string;
  readonly eventTime: number
};

export type WsDepositPrice_t =
    { TAG: "Snapshot"; _0: WsDepositPrice_snapshot }
  | { TAG: "Candle"; _0: WsDepositPrice_candle }
  | { TAG: "Price"; _0: WsDepositPrice_tick };

export type WsDepositAssetPrice_snapshot = { readonly depositAsset: LightconeSdk_Shared_pubkeyStr; readonly price: string };

export type WsDepositAssetPrice_tick = {
  readonly depositAsset: LightconeSdk_Shared_pubkeyStr;
  readonly price: string;
  readonly eventTime: number
};

export type WsDepositAssetPrice_t =
    { TAG: "Snapshot"; _0: WsDepositAssetPrice_snapshot }
  | { TAG: "Price"; _0: WsDepositAssetPrice_tick };

export type UserBalanceUpdate_t = {
  readonly marketPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly marketBalance: LightconeSdk_Order__Raw_UserMarketBalance_t;
  readonly timestamp: string
};

export type GlobalDepositUpdate_t = {
  readonly mint: LightconeSdk_Shared_pubkeyStr;
  readonly balance: string;
  readonly timestamp: string
};

export type NonceUpdate_t = {
  readonly userPubkey: LightconeSdk_Shared_pubkeyStr;
  readonly newNonce: number;
  readonly timestamp: string
};

export type UserUpdate_t =
    { TAG: "Snapshot"; _0: LightconeSdk_Order__Raw_UserSnapshot_t }
  | { TAG: "Order"; _0: LightconeSdk_Order__Raw_Event_t }
  | { TAG: "BalanceUpdate"; _0: UserBalanceUpdate_t }
  | { TAG: "GlobalDepositUpdate"; _0: GlobalDepositUpdate_t }
  | { TAG: "NonceUpdate"; _0: NonceUpdate_t }
  | { TAG: "NotificationPush"; _0: LightconeSdk_Notification__Model_t };

export type Kind_t =
    "Pong"
  | { TAG: "BookUpdate"; _0: LightconeSdk_Orderbook__Raw_Book_t }
  | { TAG: "Trades"; _0: Trade_t }
  | { TAG: "User"; _0: UserUpdate_t }
  | { TAG: "WalletDepositBalances"; _0: LightconeSdk_Position__Raw_WalletDepositBalancesEvent_t }
  | { TAG: "Ticker"; _0: Ticker_t }
  | { TAG: "PriceHistory"; _0: WsPriceHistory_t }
  | { TAG: "Market"; _0: MarketEvent_t }
  | { TAG: "DepositPrice"; _0: WsDepositPrice_t }
  | { TAG: "DepositAssetPrice"; _0: WsDepositAssetPrice_t }
  | { TAG: "Auth"; _0: AuthUpdate_t }
  | { TAG: "ErrorFrame"; _0: ErrorFrame_t };

export type t = { readonly kind: Kind_t; readonly version: number };
