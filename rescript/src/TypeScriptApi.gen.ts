/* TypeScript file generated from TypeScriptApi.res by genType. */

/* eslint-disable */
/* tslint:disable */

import * as TypeScriptApiJS from './TypeScriptApi.res.mjs';

import type {Active_t as RpcFailover_Active_t} from './RpcFailover.gen.ts';

import type {Book_t as Orderbook__Raw_Book_t} from '../src/domain/orderbook/Orderbook__Raw.gen.ts';

import type {CancelAllSuccess_t as Order__Raw_CancelAllSuccess_t} from '../src/domain/order/Order__Raw.gen.ts';

import type {CancelSuccess_t as Order__Raw_CancelSuccess_t} from '../src/domain/order/Order__Raw.gen.ts';

import type {CancelTriggerSuccess_t as Order__Raw_CancelTriggerSuccess_t} from '../src/domain/order/Order__Raw.gen.ts';

import type {Categories_t as Metrics__Raw_Categories_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {DepositCandle_t as PriceHistory__Raw_DepositCandle_t} from '../src/domain/priceHistory/PriceHistory__Raw.gen.ts';

import type {DepositMintsResponse_t as Market__Raw_DepositMintsResponse_t} from '../src/domain/market/Market__Raw.gen.ts';

import type {DepositPricesSnapshotResponse_t as PriceHistory__Raw_DepositPricesSnapshotResponse_t} from '../src/domain/priceHistory/PriceHistory__Raw.gen.ts';

import type {DepositSource_t as Shared_DepositSource_t} from './Shared.gen.ts';

import type {DepositTokenBalance_t as Position__Raw_DepositTokenBalance_t} from '../src/domain/position/Position__Raw.gen.ts';

import type {DepositTokenVolumeHistory_t as Metrics__Raw_DepositTokenVolumeHistory_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {DepositTokens_t as Metrics__Raw_DepositTokens_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {DepthResponse_t as Orderbook__Raw_DepthResponse_t} from '../src/domain/orderbook/Orderbook__Raw.gen.ts';

import type {Exchange_t as Accounts_Exchange_t} from '../src/program/Accounts.gen.ts';

import type {GlobalDepositAssetsResult_t as Market__Model_GlobalDepositAssetsResult_t} from '../src/domain/market/Market__Model.gen.ts';

import type {History_t as Metrics__Raw_History_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {Leaderboard_t as Metrics__Raw_Leaderboard_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {Limit_t as Order__Model_Limit_t} from '../src/domain/order/Order__Model.gen.ts';

import type {Limits_t as Order__State_Limits_t} from '../src/domain/order/Order__State.gen.ts';

import type {LineData_t as PriceHistory__Model_LineData_t} from '../src/domain/priceHistory/PriceHistory__Model.gen.ts';

import type {MarketDetail_t as Metrics__Raw_MarketDetail_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {MarketPositionsResponse_t as Position__Raw_MarketPositionsResponse_t} from '../src/domain/position/Position__Raw.gen.ts';

import type {MarketSearchResult_t as Market__Raw_MarketSearchResult_t} from '../src/domain/market/Market__Raw.gen.ts';

import type {Market_t as Accounts_Market_t} from '../src/program/Accounts.gen.ts';

import type {MarketsResult_t as Market__Model_MarketsResult_t} from '../src/domain/market/Market__Model.gen.ts';

import type {Markets_t as Metrics__Raw_Markets_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {OpenInterestHistory_t as Metrics__Raw_OpenInterestHistory_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {OrderbookCandle_t as PriceHistory__Raw_OrderbookCandle_t} from '../src/domain/priceHistory/PriceHistory__Raw.gen.ts';

import type {OrderbookResponse_t as PriceHistory__Raw_OrderbookResponse_t} from '../src/domain/priceHistory/PriceHistory__Raw.gen.ts';

import type {OrderbookTickersResponse_t as Metrics__Raw_OrderbookTickersResponse_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {OrderbookVolume_t as Metrics__Raw_OrderbookVolume_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {Orderbook_t as Accounts_Orderbook_t} from '../src/program/Accounts.gen.ts';

import type {Page_t as Trade__Model_Page_t} from '../src/domain/trade/Trade__Model.gen.ts';

import type {Platform_t as Metrics__Raw_Platform_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {Position_t as Accounts_Position_t} from '../src/program/Accounts.gen.ts';

import type {PositionsResponse_t as Position__Raw_PositionsResponse_t} from '../src/domain/position/Position__Raw.gen.ts';

import type {ReadyState_t as Ws_ReadyState_t} from '../src/ws/Ws.gen.ts';

import type {RedeemResult_t as Referral__Model_RedeemResult_t} from '../src/domain/referral/Referral__Model.gen.ts';

import type {Resolution_t as Shared_Resolution_t} from './Shared.gen.ts';

import type {Response_t as Faucet__Raw_Response_t} from '../src/domain/faucet/Faucet__Raw.gen.ts';

import type {Session_t as Auth__Model_Session_t} from '../src/auth/Auth__Model.gen.ts';

import type {SnapshotOrder_t as Order__Raw_SnapshotOrder_t} from '../src/domain/order/Order__Raw.gen.ts';

import type {Status_t as Referral__Model_Status_t} from '../src/domain/referral/Referral__Model.gen.ts';

import type {SubmitResponse_t as Order__Raw_SubmitResponse_t} from '../src/domain/order/Order__Raw.gen.ts';

import type {SubscribeParams_t as Subscriptions_SubscribeParams_t} from '../src/ws/Subscriptions.gen.ts';

import type {TimeInForce_t as Shared_TimeInForce_t} from './Shared.gen.ts';

import type {TriggerResponse_t as Order__Raw_TriggerResponse_t} from '../src/domain/order/Order__Raw.gen.ts';

import type {TriggerType_t as Shared_TriggerType_t} from './Shared.gen.ts';

import type {TriggerUpdate_t as Order__Raw_TriggerUpdate_t} from '../src/domain/order/Order__Raw.gen.ts';

import type {Trigger_t as Order__Model_Trigger_t} from '../src/domain/order/Order__Model.gen.ts';

import type {Triggers_t as Order__State_Triggers_t} from '../src/domain/order/Order__State.gen.ts';

import type {UniqueTradersHistoryScope_t as Metrics__Raw_UniqueTradersHistoryScope_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {UniqueTradersHistory_t as Metrics__Raw_UniqueTradersHistory_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {UnsubscribeParams_t as Subscriptions_UnsubscribeParams_t} from '../src/ws/Subscriptions.gen.ts';

import type {Update_t as Order__Raw_Update_t} from '../src/domain/order/Order__Raw.gen.ts';

import type {UserFillsResponse_t as Order__Raw_UserFillsResponse_t} from '../src/domain/order/Order__Raw.gen.ts';

import type {UserMarketBalance_t as Order__Raw_UserMarketBalance_t} from '../src/domain/order/Order__Raw.gen.ts';

import type {UserOrdersResponse_t as Order__Raw_UserOrdersResponse_t} from '../src/domain/order/Order__Raw.gen.ts';

import type {User_t as Metrics__Raw_User_t} from '../src/domain/metrics/Metrics__Raw.gen.ts';

import type {applyResult as Orderbook__State_applyResult} from '../src/domain/orderbook/Orderbook__State.gen.ts';

import type {depositAssetBalanceIndex as Position__State_depositAssetBalanceIndex} from '../src/domain/position/Position__State.gen.ts';

import type {latestPrice as PriceHistory__DepositState_latestPrice} from '../src/domain/priceHistory/PriceHistory__DepositState.gen.ts';

import type {orderBookId as Shared_orderBookId} from './Shared.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from './Shared.gen.ts';

import type {t as Client_t} from './Client.gen.ts';

import type {t as Env_t} from './Env.gen.ts';

import type {t as Market__Model_t} from '../src/domain/market/Market__Model.gen.ts';

import type {t as Messages_t} from '../src/ws/Messages.gen.ts';

import type {t as Notification__Model_t} from '../src/domain/notification/Notification__Model.gen.ts';

import type {t as Orderbook__State_t} from '../src/domain/orderbook/Orderbook__State.gen.ts';

import type {t as Position__State_t} from '../src/domain/position/Position__State.gen.ts';

import type {t as PriceHistory__DepositState_t} from '../src/domain/priceHistory/PriceHistory__DepositState.gen.ts';

import type {t as PriceHistory__State_t} from '../src/domain/priceHistory/PriceHistory__State.gen.ts';

import type {t as SdkError_t} from './SdkError.gen.ts';

import type {t as Trade__Model_t} from '../src/domain/trade/Trade__Model.gen.ts';

import type {t as Trade__State_t} from '../src/domain/trade/Trade__State.gen.ts';

export abstract class WsClient_wsConnection { protected opaque!: any }; /* simulate opaque types */

export type WsClient_wsMessage = Messages_t;

export type WsClient_wsSubscription = Subscriptions_SubscribeParams_t;

export type WsClient_wsUnsubscription = Subscriptions_UnsubscribeParams_t;

export const make: (env:(undefined | Env_t), baseUrl:(undefined | string), wsUrl:(undefined | string), rpcUrl:(undefined | string), backupRpcUrl:(undefined | string), programId:(undefined | string), depositSource:(undefined | Shared_DepositSource_t), _8:void) => Client_t = TypeScriptApiJS.make as any;

export const makeForEnv: (env:Env_t) => Client_t = TypeScriptApiJS.makeForEnv as any;

export const useNativeSigner: (client:Client_t, secretKey:Uint8Array) => Promise<void> = TypeScriptApiJS.useNativeSigner as any;

export const useExternalSigner: (client:Client_t, address:string, signMessage:((_1:Uint8Array) => Promise<Uint8Array>), signTransaction:((_1:Uint8Array) => Promise<Uint8Array>)) => void = TypeScriptApiJS.useExternalSigner as any;

export const clearSigningStrategy: (client:Client_t) => void = TypeScriptApiJS.clearSigningStrategy as any;

export const clearOrderNonce: (client:Client_t) => void = TypeScriptApiJS.clearOrderNonce as any;

export const signerAddress: (client:Client_t) => (undefined | string) = TypeScriptApiJS.signerAddress as any;

export const unwrap: <T1>(_1:Promise<
    { TAG: "Ok"; _0: T1 }
  | { TAG: "Error"; _0: SdkError_t }>) => Promise<T1> = TypeScriptApiJS.unwrap as any;

export const AuthClient_getNonce: (client:Client_t) => Promise<string> = TypeScriptApiJS.AuthClient.getNonce as any;

export const AuthClient_login: (client:Client_t, useEmbeddedWallet:(undefined | boolean)) => Promise<Auth__Model_Session_t> = TypeScriptApiJS.AuthClient.login as any;

export const AuthClient_checkSession: (client:Client_t, cookieHeader:(undefined | string)) => Promise<Auth__Model_Session_t> = TypeScriptApiJS.AuthClient.checkSession as any;

export const AuthClient_logout: (client:Client_t) => Promise<void> = TypeScriptApiJS.AuthClient.logout as any;

export const AuthClient_isAuthenticated: (client:Client_t) => boolean = TypeScriptApiJS.AuthClient.isAuthenticated as any;

export const AuthClient_registerPrivy: (client:Client_t) => Promise<void> = TypeScriptApiJS.AuthClient.registerPrivy as any;

export const AuthClient_disconnectX: (client:Client_t) => Promise<void> = TypeScriptApiJS.AuthClient.disconnectX as any;

export const AuthClient_connectXUrl: (client:Client_t) => string = TypeScriptApiJS.AuthClient.connectXUrl as any;

export const MarketClient_get: (client:Client_t, cursor:(undefined | number), limit:(undefined | number)) => Promise<Market__Model_MarketsResult_t> = TypeScriptApiJS.MarketClient.get as any;

export const MarketClient_featured: (client:Client_t) => Promise<Market__Raw_MarketSearchResult_t[]> = TypeScriptApiJS.MarketClient.featured as any;

export const MarketClient_getBySlug: (client:Client_t, slug:string) => Promise<Market__Model_t> = TypeScriptApiJS.MarketClient.getBySlug as any;

export const MarketClient_getByPubkey: (client:Client_t, pubkey:string) => Promise<Market__Model_t> = TypeScriptApiJS.MarketClient.getByPubkey as any;

export const MarketClient_search: (client:Client_t, query:string, limit:(undefined | number)) => Promise<Market__Raw_MarketSearchResult_t[]> = TypeScriptApiJS.MarketClient.search as any;

export const MarketClient_globalDepositAssets: (client:Client_t) => Promise<Market__Model_GlobalDepositAssetsResult_t> = TypeScriptApiJS.MarketClient.globalDepositAssets as any;

export const MarketClient_depositMints: (client:Client_t, marketPubkey:string) => Promise<Market__Raw_DepositMintsResponse_t> = TypeScriptApiJS.MarketClient.depositMints as any;

export const OrderbookClient_get: (client:Client_t, orderbookId:string, depth:(undefined | number)) => Promise<Orderbook__Raw_DepthResponse_t> = TypeScriptApiJS.OrderbookClient.get as any;

export const TradeClient_forOrderbook: (client:Client_t, orderbookId:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<Trade__Model_Page_t> = TypeScriptApiJS.TradeClient.forOrderbook as any;

export const TradeClient_forMarket: (client:Client_t, marketPubkey:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<Trade__Model_Page_t> = TypeScriptApiJS.TradeClient.forMarket as any;

export const OrderClient_forUser: (client:Client_t, limit:(undefined | number), cursor:(undefined | string)) => Promise<Order__Raw_UserOrdersResponse_t> = TypeScriptApiJS.OrderClient.forUser as any;

export const OrderClient_submitLimit: (client:Client_t, market:string, baseMint:string, quoteMint:string, side:number, price:string, size:string, baseDecimals:number, quoteDecimals:number, priceDecimals:number, tickSize:number, orderbookId:string, timeInForce:(undefined | Shared_TimeInForce_t)) => Promise<Order__Raw_SubmitResponse_t> = TypeScriptApiJS.OrderClient.submitLimit as any;

export const OrderClient_submitTrigger: (client:Client_t, market:string, baseMint:string, quoteMint:string, side:number, price:string, size:string, baseDecimals:number, quoteDecimals:number, priceDecimals:number, tickSize:number, orderbookId:string, triggerPrice:number, triggerType:Shared_TriggerType_t, timeInForce:(undefined | Shared_TimeInForce_t)) => Promise<Order__Raw_TriggerResponse_t> = TypeScriptApiJS.OrderClient.submitTrigger as any;

export const OrderClient_cancel: (client:Client_t, orderHash:string) => Promise<Order__Raw_CancelSuccess_t> = TypeScriptApiJS.OrderClient.cancel as any;

export const OrderClient_cancelTrigger: (client:Client_t, triggerOrderId:string) => Promise<Order__Raw_CancelTriggerSuccess_t> = TypeScriptApiJS.OrderClient.cancelTrigger as any;

export const OrderClient_cancelAll: (client:Client_t, orderbookId:string) => Promise<Order__Raw_CancelAllSuccess_t> = TypeScriptApiJS.OrderClient.cancelAll as any;

export const OrderClient_fills: (client:Client_t, marketPubkey:(undefined | string), limit:(undefined | number), cursor:(undefined | string)) => Promise<Order__Raw_UserFillsResponse_t> = TypeScriptApiJS.OrderClient.fills as any;

export const OrderClient_fillsByWallet: (client:Client_t, walletAddress:string, marketPubkey:(undefined | string), limit:(undefined | number), cursor:(undefined | string)) => Promise<Order__Raw_UserFillsResponse_t> = TypeScriptApiJS.OrderClient.fillsByWallet as any;

export const PositionClient_forUser: (client:Client_t, userPubkey:string) => Promise<Position__Raw_PositionsResponse_t> = TypeScriptApiJS.PositionClient.forUser as any;

export const PositionClient_forMarket: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<Position__Raw_MarketPositionsResponse_t> = TypeScriptApiJS.PositionClient.forMarket as any;

export const PositionClient_mine: (client:Client_t) => Promise<Position__Raw_PositionsResponse_t> = TypeScriptApiJS.PositionClient.mine as any;

export const PositionClient_depositTokenBalances: (client:Client_t) => Promise<{[id: string]: Position__Raw_DepositTokenBalance_t}> = TypeScriptApiJS.PositionClient.depositTokenBalances as any;

export const PositionClient_depositToGlobal: (client:Client_t, mint:string, amount:bigint) => Promise<string> = TypeScriptApiJS.PositionClient.depositToGlobal as any;

export const PositionClient_withdrawFromGlobal: (client:Client_t, mint:string, amount:bigint) => Promise<string> = TypeScriptApiJS.PositionClient.withdrawFromGlobal as any;

export const PositionClient_globalToMarketDeposit: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string> = TypeScriptApiJS.PositionClient.globalToMarketDeposit as any;

export const PositionClient_merge: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string> = TypeScriptApiJS.PositionClient.merge as any;

export const PositionClient_redeemWinnings: (client:Client_t, market:string, mint:string, amount:bigint, outcomeIndex:number) => Promise<string> = TypeScriptApiJS.PositionClient.redeemWinnings as any;

export const PositionClient_deposit: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string> = TypeScriptApiJS.PositionClient.deposit as any;

export const PositionClient_withdrawFromPosition: (client:Client_t, market:string, mint:string, amount:bigint, outcomeIndex:number) => Promise<string> = TypeScriptApiJS.PositionClient.withdrawFromPosition as any;

export const PositionClient_extendPositionTokens: (client:Client_t, user:string, market:string, lookupTable:string, depositMints:string[], numOutcomes:number) => Promise<string> = TypeScriptApiJS.PositionClient.extendPositionTokens as any;

export const PositionClient_closePositionAlt: (client:Client_t, position:string, market:string, lookupTable:string) => Promise<string> = TypeScriptApiJS.PositionClient.closePositionAlt as any;

export const PositionClient_closePositionTokenAccounts: (client:Client_t, market:string, position:string, depositMints:string[], numOutcomes:number) => Promise<string> = TypeScriptApiJS.PositionClient.closePositionTokenAccounts as any;

export const MetricsClient_platform: (client:Client_t) => Promise<Metrics__Raw_Platform_t> = TypeScriptApiJS.MetricsClient.platform as any;

export const MetricsClient_markets: (client:Client_t) => Promise<Metrics__Raw_Markets_t> = TypeScriptApiJS.MetricsClient.markets as any;

export const MetricsClient_market: (client:Client_t, marketPubkey:string) => Promise<Metrics__Raw_MarketDetail_t> = TypeScriptApiJS.MetricsClient.market as any;

export const MetricsClient_orderbookTickers: (client:Client_t, depositAsset:(undefined | string)) => Promise<Metrics__Raw_OrderbookTickersResponse_t> = TypeScriptApiJS.MetricsClient.orderbookTickers as any;

export const MetricsClient_categories: (client:Client_t) => Promise<Metrics__Raw_Categories_t> = TypeScriptApiJS.MetricsClient.categories as any;

export const MetricsClient_depositTokens: (client:Client_t) => Promise<Metrics__Raw_DepositTokens_t> = TypeScriptApiJS.MetricsClient.depositTokens as any;

export const MetricsClient_leaderboard: (client:Client_t, limit:(undefined | number)) => Promise<Metrics__Raw_Leaderboard_t> = TypeScriptApiJS.MetricsClient.leaderboard as any;

export const MetricsClient_orderbook: (client:Client_t, orderbookId:string) => Promise<Metrics__Raw_OrderbookVolume_t> = TypeScriptApiJS.MetricsClient.orderbook as any;

export const MetricsClient_depositTokensVolumeHistory: (client:Client_t, fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics__Raw_DepositTokenVolumeHistory_t> = TypeScriptApiJS.MetricsClient.depositTokensVolumeHistory as any;

export const MetricsClient_openInterestHistory: (client:Client_t, fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics__Raw_OpenInterestHistory_t> = TypeScriptApiJS.MetricsClient.openInterestHistory as any;

export const MetricsClient_uniqueTradersHistory: (client:Client_t, scope:(undefined | Metrics__Raw_UniqueTradersHistoryScope_t), scopeKey:(undefined | string), fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics__Raw_UniqueTradersHistory_t> = TypeScriptApiJS.MetricsClient.uniqueTradersHistory as any;

export const MetricsClient_history: (client:Client_t, scope:string, scopeKey:string, resolution:(undefined | Shared_Resolution_t), fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics__Raw_History_t> = TypeScriptApiJS.MetricsClient.history as any;

export const MetricsClient_user: (client:Client_t) => Promise<Metrics__Raw_User_t> = TypeScriptApiJS.MetricsClient.user as any;

export const MetricsClient_userByWallet: (client:Client_t, walletAddress:string) => Promise<Metrics__Raw_User_t> = TypeScriptApiJS.MetricsClient.userByWallet as any;

export const PriceHistoryClient_get: (client:Client_t, orderbookId:string, resolution:Shared_Resolution_t, fromMs:(undefined | number), toMs:(undefined | number)) => Promise<PriceHistory__Raw_OrderbookResponse_t> = TypeScriptApiJS.PriceHistoryClient.get as any;

export const PriceHistoryClient_lineData: (client:Client_t, orderbookId:string, resolution:Shared_Resolution_t, fromMs:(undefined | number), toMs:(undefined | number), cursor:(undefined | number), limit:(undefined | number)) => Promise<PriceHistory__Model_LineData_t[]> = TypeScriptApiJS.PriceHistoryClient.lineData as any;

export const PriceHistoryClient_depositAssetSnapshot: (client:Client_t) => Promise<PriceHistory__Raw_DepositPricesSnapshotResponse_t> = TypeScriptApiJS.PriceHistoryClient.depositAssetSnapshot as any;

export const NotificationClient_list: (client:Client_t) => Promise<Notification__Model_t[]> = TypeScriptApiJS.NotificationClient.list as any;

export const NotificationClient_dismiss: (client:Client_t, notificationId:string) => Promise<void> = TypeScriptApiJS.NotificationClient.dismiss as any;

export const ReferralClient_status: (client:Client_t) => Promise<Referral__Model_Status_t> = TypeScriptApiJS.ReferralClient.status as any;

export const ReferralClient_redeem: (client:Client_t, code:string) => Promise<Referral__Model_RedeemResult_t> = TypeScriptApiJS.ReferralClient.redeem as any;

export const FaucetClient_claim: (client:Client_t, walletAddress:string) => Promise<Faucet__Raw_Response_t> = TypeScriptApiJS.FaucetClient.claim as any;

export const RpcClient_activeRpc: (client:Client_t) => RpcFailover_Active_t = TypeScriptApiJS.RpcClient.activeRpc as any;

export const RpcClient_latestBlockhash: (client:Client_t) => Promise<string> = TypeScriptApiJS.RpcClient.latestBlockhash as any;

export const RpcClient_exchange: (client:Client_t) => Promise<Accounts_Exchange_t> = TypeScriptApiJS.RpcClient.exchange as any;

export const RpcClient_market: (client:Client_t, marketPubkey:string) => Promise<Accounts_Market_t> = TypeScriptApiJS.RpcClient.market as any;

export const RpcClient_orderbook: (client:Client_t, baseMint:string, quoteMint:string) => Promise<Accounts_Orderbook_t> = TypeScriptApiJS.RpcClient.orderbook as any;

export const RpcClient_position: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<(undefined | Accounts_Position_t)> = TypeScriptApiJS.RpcClient.position as any;

export const RpcClient_nonce: (client:Client_t, userPubkey:string) => Promise<number> = TypeScriptApiJS.RpcClient.nonce as any;

export const RpcClient_exchangePda: (client:Client_t) => Promise<string> = TypeScriptApiJS.RpcClient.exchangePda as any;

export const RpcClient_marketPda: (client:Client_t, marketId:bigint) => Promise<string> = TypeScriptApiJS.RpcClient.marketPda as any;

export const RpcClient_positionPda: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<string> = TypeScriptApiJS.RpcClient.positionPda as any;

export const RpcClient_globalDepositTokenPda: (client:Client_t, mint:string) => Promise<string> = TypeScriptApiJS.RpcClient.globalDepositTokenPda as any;

export const WsClient_connect: (client:Client_t, onMessage:((_1:Messages_t) => void), onConnected:(undefined | ((() => void))), onError:(undefined | (((_1:SdkError_t) => void)))) => WsClient_wsConnection = TypeScriptApiJS.WsClient.connect as any;

export const WsClient_subscribe: (connection:WsClient_wsConnection, subscription:Subscriptions_SubscribeParams_t) => void = TypeScriptApiJS.WsClient.subscribe as any;

export const WsClient_unsubscribe: (connection:WsClient_wsConnection, subscription:Subscriptions_UnsubscribeParams_t) => void = TypeScriptApiJS.WsClient.unsubscribe as any;

export const WsClient_disconnect: (connection:WsClient_wsConnection) => void = TypeScriptApiJS.WsClient.disconnect as any;

export const WsClient_isConnected: (connection:WsClient_wsConnection) => boolean = TypeScriptApiJS.WsClient.isConnected as any;

export const WsClient_readyState: (connection:WsClient_wsConnection) => Ws_ReadyState_t = TypeScriptApiJS.WsClient.readyState as any;

export const WsClient_clearAuthedSubscriptions: (connection:WsClient_wsConnection) => void = TypeScriptApiJS.WsClient.clearAuthedSubscriptions as any;

export const LiveOrderbook_make: (_1:Shared_orderBookId) => Orderbook__State_t = TypeScriptApiJS.LiveOrderbook.make as any;

export const LiveOrderbook_apply: (_1:Orderbook__State_t, _2:Orderbook__Raw_Book_t) => Orderbook__State_applyResult = TypeScriptApiJS.LiveOrderbook.apply as any;

export const LiveOrderbook_bestBid: (_1:Orderbook__State_t) => (undefined | string) = TypeScriptApiJS.LiveOrderbook.bestBid as any;

export const LiveOrderbook_bestAsk: (_1:Orderbook__State_t) => (undefined | string) = TypeScriptApiJS.LiveOrderbook.bestAsk as any;

export const LiveOrderbook_midPrice: (_1:Orderbook__State_t) => (undefined | string) = TypeScriptApiJS.LiveOrderbook.midPrice as any;

export const LiveOrderbook_spread: (_1:Orderbook__State_t) => (undefined | string) = TypeScriptApiJS.LiveOrderbook.spread as any;

export const LiveOrderbook_bids: (_1:Orderbook__State_t) => Array<[string, string]> = TypeScriptApiJS.LiveOrderbook.bids as any;

export const LiveOrderbook_asks: (_1:Orderbook__State_t) => Array<[string, string]> = TypeScriptApiJS.LiveOrderbook.asks as any;

export const LiveOrderbook_isEmpty: (_1:Orderbook__State_t) => boolean = TypeScriptApiJS.LiveOrderbook.isEmpty as any;

export const LiveOrderbook_seq: (_1:Orderbook__State_t) => number = TypeScriptApiJS.LiveOrderbook.seq as any;

export const LiveOrderbook_orderbookId: (_1:Orderbook__State_t) => Shared_orderBookId = TypeScriptApiJS.LiveOrderbook.orderbookId as any;

export const LiveOrderbook_clear: (_1:Orderbook__State_t) => void = TypeScriptApiJS.LiveOrderbook.clear as any;

export const LivePriceHistory_make: () => PriceHistory__State_t = TypeScriptApiJS.LivePriceHistory.make as any;

export const LivePriceHistory_applySnapshot: (_1:PriceHistory__State_t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t, candles:PriceHistory__Raw_OrderbookCandle_t[]) => void = TypeScriptApiJS.LivePriceHistory.applySnapshot as any;

export const LivePriceHistory_applyUpdate: (_1:PriceHistory__State_t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t, candle:PriceHistory__Raw_OrderbookCandle_t) => void = TypeScriptApiJS.LivePriceHistory.applyUpdate as any;

export const LivePriceHistory_get: (_1:PriceHistory__State_t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t) => (undefined | PriceHistory__Model_LineData_t[]) = TypeScriptApiJS.LivePriceHistory.get as any;

export const LivePriceHistory_clear: (_1:PriceHistory__State_t) => void = TypeScriptApiJS.LivePriceHistory.clear as any;

export const LiveDepositPrice_make: () => PriceHistory__DepositState_t = TypeScriptApiJS.LiveDepositPrice.make as any;

export const LiveDepositPrice_applySnapshot: (_1:PriceHistory__DepositState_t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t, candles:PriceHistory__Raw_DepositCandle_t[]) => void = TypeScriptApiJS.LiveDepositPrice.applySnapshot as any;

export const LiveDepositPrice_applyCandle: (_1:PriceHistory__DepositState_t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t, candle:PriceHistory__Raw_DepositCandle_t) => void = TypeScriptApiJS.LiveDepositPrice.applyCandle as any;

export const LiveDepositPrice_applyPriceTick: (_1:PriceHistory__DepositState_t, depositAsset:Shared_pubkeyStr, price:string, eventTime:number) => void = TypeScriptApiJS.LiveDepositPrice.applyPriceTick as any;

export const LiveDepositPrice_applyAssetSnapshot: (_1:PriceHistory__DepositState_t, depositAsset:Shared_pubkeyStr, price:string) => void = TypeScriptApiJS.LiveDepositPrice.applyAssetSnapshot as any;

export const LiveDepositPrice_getCandles: (_1:PriceHistory__DepositState_t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t) => (undefined | PriceHistory__Raw_DepositCandle_t[]) = TypeScriptApiJS.LiveDepositPrice.getCandles as any;

export const LiveDepositPrice_getLatestPrice: (_1:PriceHistory__DepositState_t, depositAsset:Shared_pubkeyStr) => (undefined | PriceHistory__DepositState_latestPrice) = TypeScriptApiJS.LiveDepositPrice.getLatestPrice as any;

export const LiveDepositPrice_clear: (_1:PriceHistory__DepositState_t) => void = TypeScriptApiJS.LiveDepositPrice.clear as any;

export const LiveOpenLimitOrders_make: () => Order__State_Limits_t = TypeScriptApiJS.LiveOpenLimitOrders.make as any;

export const LiveOpenLimitOrders_get: (_1:Order__State_Limits_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | Order__Model_Limit_t[]) = TypeScriptApiJS.LiveOpenLimitOrders.get as any;

export const LiveOpenLimitOrders_getByMarket: (_1:Order__State_Limits_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: Order__Model_Limit_t[]}) = TypeScriptApiJS.LiveOpenLimitOrders.getByMarket as any;

export const LiveOpenLimitOrders_insert: (_1:Order__State_Limits_t, _2:Order__Model_Limit_t) => void = TypeScriptApiJS.LiveOpenLimitOrders.insert as any;

export const LiveOpenLimitOrders_upsert: (_1:Order__State_Limits_t, _2:Order__Raw_Update_t) => void = TypeScriptApiJS.LiveOpenLimitOrders.upsert as any;

export const LiveOpenLimitOrders_remove: (_1:Order__State_Limits_t, orderHash:string) => void = TypeScriptApiJS.LiveOpenLimitOrders.remove as any;

export const LiveOpenLimitOrders_clear: (_1:Order__State_Limits_t) => void = TypeScriptApiJS.LiveOpenLimitOrders.clear as any;

export const LiveOpenLimitOrders_isEmpty: (_1:Order__State_Limits_t) => boolean = TypeScriptApiJS.LiveOpenLimitOrders.isEmpty as any;

export const LiveOpenLimitOrders_limitOrderOfUpdate: (_1:Order__Raw_Update_t) => Order__Model_Limit_t = TypeScriptApiJS.LiveOpenLimitOrders.limitOrderOfUpdate as any;

export const LiveOpenLimitOrders_ofSnapshotOrders: (_1:Order__Raw_SnapshotOrder_t[]) => [Order__State_Limits_t, Order__State_Triggers_t] = TypeScriptApiJS.LiveOpenLimitOrders.ofSnapshotOrders as any;

export const LiveTriggerOrders_make: () => Order__State_Triggers_t = TypeScriptApiJS.LiveTriggerOrders.make as any;

export const LiveTriggerOrders_get: (_1:Order__State_Triggers_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | Order__Model_Trigger_t[]) = TypeScriptApiJS.LiveTriggerOrders.get as any;

export const LiveTriggerOrders_getByMarket: (_1:Order__State_Triggers_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: Order__Model_Trigger_t[]}) = TypeScriptApiJS.LiveTriggerOrders.getByMarket as any;

export const LiveTriggerOrders_all: (_1:Order__State_Triggers_t) => Order__Model_Trigger_t[] = TypeScriptApiJS.LiveTriggerOrders.all as any;

export const LiveTriggerOrders_getById: (_1:Order__State_Triggers_t, triggerOrderId:string) => (undefined | Order__Model_Trigger_t) = TypeScriptApiJS.LiveTriggerOrders.getById as any;

export const LiveTriggerOrders_insert: (_1:Order__State_Triggers_t, _2:Order__Model_Trigger_t) => void = TypeScriptApiJS.LiveTriggerOrders.insert as any;

export const LiveTriggerOrders_remove: (_1:Order__State_Triggers_t, triggerOrderId:string) => (undefined | Order__Model_Trigger_t) = TypeScriptApiJS.LiveTriggerOrders.remove as any;

export const LiveTriggerOrders_clear: (_1:Order__State_Triggers_t) => void = TypeScriptApiJS.LiveTriggerOrders.clear as any;

export const LiveTriggerOrders_isEmpty: (_1:Order__State_Triggers_t) => boolean = TypeScriptApiJS.LiveTriggerOrders.isEmpty as any;

export const LiveTriggerOrders_size: (_1:Order__State_Triggers_t) => number = TypeScriptApiJS.LiveTriggerOrders.size as any;

export const LiveTriggerOrders_triggerOrderOfUpdate: (_1:Order__Raw_TriggerUpdate_t) => Order__Model_Trigger_t = TypeScriptApiJS.LiveTriggerOrders.triggerOrderOfUpdate as any;

export const LiveTriggerOrders_limitPrice: (_1:Order__Model_Trigger_t) => (undefined | string) = TypeScriptApiJS.LiveTriggerOrders.limitPrice as any;

export const LiveTrades_make: (orderbookId:Shared_orderBookId, maxSize:number) => Trade__State_t = TypeScriptApiJS.LiveTrades.make as any;

export const LiveTrades_push: (_1:Trade__State_t, _2:Trade__Model_t) => void = TypeScriptApiJS.LiveTrades.push as any;

export const LiveTrades_replace: (_1:Trade__State_t, _2:Trade__Model_t[]) => void = TypeScriptApiJS.LiveTrades.replace as any;

export const LiveTrades_trades: (_1:Trade__State_t) => Trade__Model_t[] = TypeScriptApiJS.LiveTrades.trades as any;

export const LiveTrades_latest: (_1:Trade__State_t) => (undefined | Trade__Model_t) = TypeScriptApiJS.LiveTrades.latest as any;

export const LiveTrades_clear: (_1:Trade__State_t) => void = TypeScriptApiJS.LiveTrades.clear as any;

export const LiveTrades_size: (_1:Trade__State_t) => number = TypeScriptApiJS.LiveTrades.size as any;

export const LiveTrades_isEmpty: (_1:Trade__State_t) => boolean = TypeScriptApiJS.LiveTrades.isEmpty as any;

export const LiveUserBalances_make: () => Position__State_t = TypeScriptApiJS.LiveUserBalances.make as any;

export const LiveUserBalances_get: (_1:Position__State_t, marketPubkey:Shared_pubkeyStr) => (undefined | Position__State_depositAssetBalanceIndex) = TypeScriptApiJS.LiveUserBalances.get as any;

export const LiveUserBalances_insert: (_1:Position__State_t, marketPubkey:Shared_pubkeyStr, _3:Position__State_depositAssetBalanceIndex) => void = TypeScriptApiJS.LiveUserBalances.insert as any;

export const LiveUserBalances_remove: (_1:Position__State_t, marketPubkey:Shared_pubkeyStr) => void = TypeScriptApiJS.LiveUserBalances.remove as any;

export const LiveUserBalances_extend: (_1:Position__State_t, _2:Position__State_t) => void = TypeScriptApiJS.LiveUserBalances.extend as any;

export const LiveUserBalances_marketPubkeys: (_1:Position__State_t) => Shared_pubkeyStr[] = TypeScriptApiJS.LiveUserBalances.marketPubkeys as any;

export const LiveUserBalances_ofMarketBalance: (_1:Order__Raw_UserMarketBalance_t) => (undefined | Position__State_t) = TypeScriptApiJS.LiveUserBalances.ofMarketBalance as any;

export const LiveUserBalances_ofMarketBalances: (_1:Order__Raw_UserMarketBalance_t[]) => Position__State_t = TypeScriptApiJS.LiveUserBalances.ofMarketBalances as any;

export const OrderbookClient: { get: (client:Client_t, orderbookId:string, depth:(undefined | number)) => Promise<Orderbook__Raw_DepthResponse_t> } = TypeScriptApiJS.OrderbookClient as any;

export const OrderClient: {
  submitLimit: (client:Client_t, market:string, baseMint:string, quoteMint:string, side:number, price:string, size:string, baseDecimals:number, quoteDecimals:number, priceDecimals:number, tickSize:number, orderbookId:string, timeInForce:(undefined | Shared_TimeInForce_t)) => Promise<Order__Raw_SubmitResponse_t>; 
  forUser: (client:Client_t, limit:(undefined | number), cursor:(undefined | string)) => Promise<Order__Raw_UserOrdersResponse_t>; 
  cancel: (client:Client_t, orderHash:string) => Promise<Order__Raw_CancelSuccess_t>; 
  fillsByWallet: (client:Client_t, walletAddress:string, marketPubkey:(undefined | string), limit:(undefined | number), cursor:(undefined | string)) => Promise<Order__Raw_UserFillsResponse_t>; 
  fills: (client:Client_t, marketPubkey:(undefined | string), limit:(undefined | number), cursor:(undefined | string)) => Promise<Order__Raw_UserFillsResponse_t>; 
  submitTrigger: (client:Client_t, market:string, baseMint:string, quoteMint:string, side:number, price:string, size:string, baseDecimals:number, quoteDecimals:number, priceDecimals:number, tickSize:number, orderbookId:string, triggerPrice:number, triggerType:Shared_TriggerType_t, timeInForce:(undefined | Shared_TimeInForce_t)) => Promise<Order__Raw_TriggerResponse_t>; 
  cancelAll: (client:Client_t, orderbookId:string) => Promise<Order__Raw_CancelAllSuccess_t>; 
  cancelTrigger: (client:Client_t, triggerOrderId:string) => Promise<Order__Raw_CancelTriggerSuccess_t>
} = TypeScriptApiJS.OrderClient as any;

export const WsClient: {
  unsubscribe: (connection:WsClient_wsConnection, subscription:Subscriptions_UnsubscribeParams_t) => void; 
  isConnected: (connection:WsClient_wsConnection) => boolean; 
  connect: (client:Client_t, onMessage:((_1:Messages_t) => void), onConnected:(undefined | ((() => void))), onError:(undefined | (((_1:SdkError_t) => void)))) => WsClient_wsConnection; 
  subscribe: (connection:WsClient_wsConnection, subscription:Subscriptions_SubscribeParams_t) => void; 
  readyState: (connection:WsClient_wsConnection) => Ws_ReadyState_t; 
  disconnect: (connection:WsClient_wsConnection) => void; 
  clearAuthedSubscriptions: (connection:WsClient_wsConnection) => void
} = TypeScriptApiJS.WsClient as any;

export const PriceHistoryClient: {
  lineData: (client:Client_t, orderbookId:string, resolution:Shared_Resolution_t, fromMs:(undefined | number), toMs:(undefined | number), cursor:(undefined | number), limit:(undefined | number)) => Promise<PriceHistory__Model_LineData_t[]>; 
  get: (client:Client_t, orderbookId:string, resolution:Shared_Resolution_t, fromMs:(undefined | number), toMs:(undefined | number)) => Promise<PriceHistory__Raw_OrderbookResponse_t>; 
  depositAssetSnapshot: (client:Client_t) => Promise<PriceHistory__Raw_DepositPricesSnapshotResponse_t>
} = TypeScriptApiJS.PriceHistoryClient as any;

export const FaucetClient: { claim: (client:Client_t, walletAddress:string) => Promise<Faucet__Raw_Response_t> } = TypeScriptApiJS.FaucetClient as any;

export const AuthClient: {
  logout: (client:Client_t) => Promise<void>; 
  login: (client:Client_t, useEmbeddedWallet:(undefined | boolean)) => Promise<Auth__Model_Session_t>; 
  disconnectX: (client:Client_t) => Promise<void>; 
  registerPrivy: (client:Client_t) => Promise<void>; 
  getNonce: (client:Client_t) => Promise<string>; 
  isAuthenticated: (client:Client_t) => boolean; 
  checkSession: (client:Client_t, cookieHeader:(undefined | string)) => Promise<Auth__Model_Session_t>; 
  connectXUrl: (client:Client_t) => string
} = TypeScriptApiJS.AuthClient as any;

export const NotificationClient: { dismiss: (client:Client_t, notificationId:string) => Promise<void>; list: (client:Client_t) => Promise<Notification__Model_t[]> } = TypeScriptApiJS.NotificationClient as any;

export const ReferralClient: { status: (client:Client_t) => Promise<Referral__Model_Status_t>; redeem: (client:Client_t, code:string) => Promise<Referral__Model_RedeemResult_t> } = TypeScriptApiJS.ReferralClient as any;

export const LiveUserBalances: {
  extend: (_1:Position__State_t, _2:Position__State_t) => void; 
  insert: (_1:Position__State_t, marketPubkey:Shared_pubkeyStr, _3:Position__State_depositAssetBalanceIndex) => void; 
  ofMarketBalances: (_1:Order__Raw_UserMarketBalance_t[]) => Position__State_t; 
  get: (_1:Position__State_t, marketPubkey:Shared_pubkeyStr) => (undefined | Position__State_depositAssetBalanceIndex); 
  remove: (_1:Position__State_t, marketPubkey:Shared_pubkeyStr) => void; 
  marketPubkeys: (_1:Position__State_t) => Shared_pubkeyStr[]; 
  make: () => Position__State_t; 
  ofMarketBalance: (_1:Order__Raw_UserMarketBalance_t) => (undefined | Position__State_t)
} = TypeScriptApiJS.LiveUserBalances as any;

export const TradeClient: { forOrderbook: (client:Client_t, orderbookId:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<Trade__Model_Page_t>; forMarket: (client:Client_t, marketPubkey:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<Trade__Model_Page_t> } = TypeScriptApiJS.TradeClient as any;

export const LiveTrades: {
  push: (_1:Trade__State_t, _2:Trade__Model_t) => void; 
  trades: (_1:Trade__State_t) => Trade__Model_t[]; 
  size: (_1:Trade__State_t) => number; 
  latest: (_1:Trade__State_t) => (undefined | Trade__Model_t); 
  make: (orderbookId:Shared_orderBookId, maxSize:number) => Trade__State_t; 
  clear: (_1:Trade__State_t) => void; 
  replace: (_1:Trade__State_t, _2:Trade__Model_t[]) => void; 
  isEmpty: (_1:Trade__State_t) => boolean
} = TypeScriptApiJS.LiveTrades as any;

export const MetricsClient: {
  depositTokens: (client:Client_t) => Promise<Metrics__Raw_DepositTokens_t>; 
  userByWallet: (client:Client_t, walletAddress:string) => Promise<Metrics__Raw_User_t>; 
  user: (client:Client_t) => Promise<Metrics__Raw_User_t>; 
  categories: (client:Client_t) => Promise<Metrics__Raw_Categories_t>; 
  depositTokensVolumeHistory: (client:Client_t, fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics__Raw_DepositTokenVolumeHistory_t>; 
  uniqueTradersHistory: (client:Client_t, scope:(undefined | Metrics__Raw_UniqueTradersHistoryScope_t), scopeKey:(undefined | string), fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics__Raw_UniqueTradersHistory_t>; 
  markets: (client:Client_t) => Promise<Metrics__Raw_Markets_t>; 
  orderbookTickers: (client:Client_t, depositAsset:(undefined | string)) => Promise<Metrics__Raw_OrderbookTickersResponse_t>; 
  orderbook: (client:Client_t, orderbookId:string) => Promise<Metrics__Raw_OrderbookVolume_t>; 
  platform: (client:Client_t) => Promise<Metrics__Raw_Platform_t>; 
  leaderboard: (client:Client_t, limit:(undefined | number)) => Promise<Metrics__Raw_Leaderboard_t>; 
  openInterestHistory: (client:Client_t, fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics__Raw_OpenInterestHistory_t>; 
  market: (client:Client_t, marketPubkey:string) => Promise<Metrics__Raw_MarketDetail_t>; 
  history: (client:Client_t, scope:string, scopeKey:string, resolution:(undefined | Shared_Resolution_t), fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics__Raw_History_t>
} = TypeScriptApiJS.MetricsClient as any;

export const MarketClient: {
  globalDepositAssets: (client:Client_t) => Promise<Market__Model_GlobalDepositAssetsResult_t>; 
  get: (client:Client_t, cursor:(undefined | number), limit:(undefined | number)) => Promise<Market__Model_MarketsResult_t>; 
  search: (client:Client_t, query:string, limit:(undefined | number)) => Promise<Market__Raw_MarketSearchResult_t[]>; 
  featured: (client:Client_t) => Promise<Market__Raw_MarketSearchResult_t[]>; 
  getByPubkey: (client:Client_t, pubkey:string) => Promise<Market__Model_t>; 
  getBySlug: (client:Client_t, slug:string) => Promise<Market__Model_t>; 
  depositMints: (client:Client_t, marketPubkey:string) => Promise<Market__Raw_DepositMintsResponse_t>
} = TypeScriptApiJS.MarketClient as any;

export const RpcClient: {
  globalDepositTokenPda: (client:Client_t, mint:string) => Promise<string>; 
  nonce: (client:Client_t, userPubkey:string) => Promise<number>; 
  marketPda: (client:Client_t, marketId:bigint) => Promise<string>; 
  position: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<(undefined | Accounts_Position_t)>; 
  exchangePda: (client:Client_t) => Promise<string>; 
  positionPda: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<string>; 
  exchange: (client:Client_t) => Promise<Accounts_Exchange_t>; 
  orderbook: (client:Client_t, baseMint:string, quoteMint:string) => Promise<Accounts_Orderbook_t>; 
  latestBlockhash: (client:Client_t) => Promise<string>; 
  activeRpc: (client:Client_t) => RpcFailover_Active_t; 
  market: (client:Client_t, marketPubkey:string) => Promise<Accounts_Market_t>
} = TypeScriptApiJS.RpcClient as any;

export const LiveOrderbook: {
  seq: (_1:Orderbook__State_t) => number; 
  spread: (_1:Orderbook__State_t) => (undefined | string); 
  bestAsk: (_1:Orderbook__State_t) => (undefined | string); 
  orderbookId: (_1:Orderbook__State_t) => Shared_orderBookId; 
  midPrice: (_1:Orderbook__State_t) => (undefined | string); 
  asks: (_1:Orderbook__State_t) => Array<[string, string]>; 
  apply: (_1:Orderbook__State_t, _2:Orderbook__Raw_Book_t) => Orderbook__State_applyResult; 
  make: (_1:Shared_orderBookId) => Orderbook__State_t; 
  bestBid: (_1:Orderbook__State_t) => (undefined | string); 
  clear: (_1:Orderbook__State_t) => void; 
  bids: (_1:Orderbook__State_t) => Array<[string, string]>; 
  isEmpty: (_1:Orderbook__State_t) => boolean
} = TypeScriptApiJS.LiveOrderbook as any;

export const LiveDepositPrice: {
  applyPriceTick: (_1:PriceHistory__DepositState_t, depositAsset:Shared_pubkeyStr, price:string, eventTime:number) => void; 
  applyAssetSnapshot: (_1:PriceHistory__DepositState_t, depositAsset:Shared_pubkeyStr, price:string) => void; 
  applySnapshot: (_1:PriceHistory__DepositState_t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t, candles:PriceHistory__Raw_DepositCandle_t[]) => void; 
  applyCandle: (_1:PriceHistory__DepositState_t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t, candle:PriceHistory__Raw_DepositCandle_t) => void; 
  getCandles: (_1:PriceHistory__DepositState_t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t) => (undefined | PriceHistory__Raw_DepositCandle_t[]); 
  make: () => PriceHistory__DepositState_t; 
  getLatestPrice: (_1:PriceHistory__DepositState_t, depositAsset:Shared_pubkeyStr) => (undefined | PriceHistory__DepositState_latestPrice); 
  clear: (_1:PriceHistory__DepositState_t) => void
} = TypeScriptApiJS.LiveDepositPrice as any;

export const LiveTriggerOrders: {
  insert: (_1:Order__State_Triggers_t, _2:Order__Model_Trigger_t) => void; 
  triggerOrderOfUpdate: (_1:Order__Raw_TriggerUpdate_t) => Order__Model_Trigger_t; 
  size: (_1:Order__State_Triggers_t) => number; 
  get: (_1:Order__State_Triggers_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | Order__Model_Trigger_t[]); 
  remove: (_1:Order__State_Triggers_t, triggerOrderId:string) => (undefined | Order__Model_Trigger_t); 
  getById: (_1:Order__State_Triggers_t, triggerOrderId:string) => (undefined | Order__Model_Trigger_t); 
  limitPrice: (_1:Order__Model_Trigger_t) => (undefined | string); 
  make: () => Order__State_Triggers_t; 
  clear: (_1:Order__State_Triggers_t) => void; 
  getByMarket: (_1:Order__State_Triggers_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: Order__Model_Trigger_t[]}); 
  all: (_1:Order__State_Triggers_t) => Order__Model_Trigger_t[]; 
  isEmpty: (_1:Order__State_Triggers_t) => boolean
} = TypeScriptApiJS.LiveTriggerOrders as any;

export const LivePriceHistory: {
  get: (_1:PriceHistory__State_t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t) => (undefined | PriceHistory__Model_LineData_t[]); 
  applySnapshot: (_1:PriceHistory__State_t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t, candles:PriceHistory__Raw_OrderbookCandle_t[]) => void; 
  applyUpdate: (_1:PriceHistory__State_t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t, candle:PriceHistory__Raw_OrderbookCandle_t) => void; 
  make: () => PriceHistory__State_t; 
  clear: (_1:PriceHistory__State_t) => void
} = TypeScriptApiJS.LivePriceHistory as any;

export const PositionClient: {
  redeemWinnings: (client:Client_t, market:string, mint:string, amount:bigint, outcomeIndex:number) => Promise<string>; 
  extendPositionTokens: (client:Client_t, user:string, market:string, lookupTable:string, depositMints:string[], numOutcomes:number) => Promise<string>; 
  mine: (client:Client_t) => Promise<Position__Raw_PositionsResponse_t>; 
  forUser: (client:Client_t, userPubkey:string) => Promise<Position__Raw_PositionsResponse_t>; 
  deposit: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string>; 
  depositTokenBalances: (client:Client_t) => Promise<{[id: string]: Position__Raw_DepositTokenBalance_t}>; 
  depositToGlobal: (client:Client_t, mint:string, amount:bigint) => Promise<string>; 
  closePositionTokenAccounts: (client:Client_t, market:string, position:string, depositMints:string[], numOutcomes:number) => Promise<string>; 
  merge: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string>; 
  globalToMarketDeposit: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string>; 
  withdrawFromGlobal: (client:Client_t, mint:string, amount:bigint) => Promise<string>; 
  closePositionAlt: (client:Client_t, position:string, market:string, lookupTable:string) => Promise<string>; 
  withdrawFromPosition: (client:Client_t, market:string, mint:string, amount:bigint, outcomeIndex:number) => Promise<string>; 
  forMarket: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<Position__Raw_MarketPositionsResponse_t>
} = TypeScriptApiJS.PositionClient as any;

export const LiveOpenLimitOrders: {
  upsert: (_1:Order__State_Limits_t, _2:Order__Raw_Update_t) => void; 
  insert: (_1:Order__State_Limits_t, _2:Order__Model_Limit_t) => void; 
  get: (_1:Order__State_Limits_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | Order__Model_Limit_t[]); 
  remove: (_1:Order__State_Limits_t, orderHash:string) => void; 
  limitOrderOfUpdate: (_1:Order__Raw_Update_t) => Order__Model_Limit_t; 
  ofSnapshotOrders: (_1:Order__Raw_SnapshotOrder_t[]) => [Order__State_Limits_t, Order__State_Triggers_t]; 
  make: () => Order__State_Limits_t; 
  clear: (_1:Order__State_Limits_t) => void; 
  getByMarket: (_1:Order__State_Limits_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: Order__Model_Limit_t[]}); 
  isEmpty: (_1:Order__State_Limits_t) => boolean
} = TypeScriptApiJS.LiveOpenLimitOrders as any;
