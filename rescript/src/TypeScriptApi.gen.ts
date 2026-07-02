/* TypeScript file generated from TypeScriptApi.res by genType. */

/* eslint-disable */
/* tslint:disable */

import * as TypeScriptApiJS from './TypeScriptApi.res.mjs';

import type {DepositSource_t as Shared_DepositSource_t} from './Shared.gen.ts';

import type {Resolution_t as Shared_Resolution_t} from './Shared.gen.ts';

import type {SubscribeParams_t as Subscriptions_SubscribeParams_t} from '../src/ws/Subscriptions.gen.ts';

import type {TimeInForce_t as Shared_TimeInForce_t} from './Shared.gen.ts';

import type {TriggerType_t as Shared_TriggerType_t} from './Shared.gen.ts';

import type {UniqueTradersHistoryScope_t as Metrics_UniqueTradersHistoryScope_t} from '../src/domain/Metrics.gen.ts';

import type {UnsubscribeParams_t as Subscriptions_UnsubscribeParams_t} from '../src/ws/Subscriptions.gen.ts';

import type {UserMarketBalanceIndex_depositAssetBalanceIndex as Position_UserMarketBalanceIndex_depositAssetBalanceIndex} from '../src/domain/Position.gen.ts';

import type {UserMarketBalanceIndex_t as Position_UserMarketBalanceIndex_t} from '../src/domain/Position.gen.ts';

import type {UserOpenLimitOrders_t as OrderState_UserOpenLimitOrders_t} from '../src/domain/OrderState.gen.ts';

import type {UserSnapshotOrder_t as Order_UserSnapshotOrder_t} from '../src/domain/Order.gen.ts';

import type {UserTriggerOrders_t as OrderState_UserTriggerOrders_t} from '../src/domain/OrderState.gen.ts';

import type {activeRpc as RpcFailover_activeRpc} from './RpcFailover.gen.ts';

import type {applyResult as OrderbookState_applyResult} from '../src/domain/OrderbookState.gen.ts';

import type {cancelAllSuccess as Order_cancelAllSuccess} from '../src/domain/Order.gen.ts';

import type {cancelSuccess as Order_cancelSuccess} from '../src/domain/Order.gen.ts';

import type {cancelTriggerSuccess as Order_cancelTriggerSuccess} from '../src/domain/Order.gen.ts';

import type {categoriesMetrics as Metrics_categoriesMetrics} from '../src/domain/Metrics.gen.ts';

import type {depositAssetPricesSnapshotResponse as PriceHistory_depositAssetPricesSnapshotResponse} from '../src/domain/PriceHistory.gen.ts';

import type {depositMintsResponse as Market_depositMintsResponse} from '../src/domain/Market.gen.ts';

import type {depositPriceCandle as PriceHistory_depositPriceCandle} from '../src/domain/PriceHistory.gen.ts';

import type {depositTokenBalance as Position_depositTokenBalance} from '../src/domain/Position.gen.ts';

import type {depositTokenVolumeHistory as Metrics_depositTokenVolumeHistory} from '../src/domain/Metrics.gen.ts';

import type {depositTokensMetrics as Metrics_depositTokensMetrics} from '../src/domain/Metrics.gen.ts';

import type {exchange as Accounts_exchange} from '../src/program/Accounts.gen.ts';

import type {faucetResponse as Faucet_faucetResponse} from '../src/domain/Faucet.gen.ts';

import type {globalDepositAssetsResult as Market_globalDepositAssetsResult} from '../src/domain/Market.gen.ts';

import type {latestDepositPrice as DepositPriceState_latestDepositPrice} from '../src/domain/DepositPriceState.gen.ts';

import type {leaderboard as Metrics_leaderboard} from '../src/domain/Metrics.gen.ts';

import type {limitOrder as OrderState_limitOrder} from '../src/domain/OrderState.gen.ts';

import type {lineData as PriceHistory_lineData} from '../src/domain/PriceHistory.gen.ts';

import type {marketDetailMetrics as Metrics_marketDetailMetrics} from '../src/domain/Metrics.gen.ts';

import type {marketPositionsResponse as Position_marketPositionsResponse} from '../src/domain/Position.gen.ts';

import type {marketSearchResult as Market_marketSearchResult} from '../src/domain/Market.gen.ts';

import type {market as Accounts_market} from '../src/program/Accounts.gen.ts';

import type {market as Market_market} from '../src/domain/Market.gen.ts';

import type {marketsMetrics as Metrics_marketsMetrics} from '../src/domain/Metrics.gen.ts';

import type {marketsResult as Market_marketsResult} from '../src/domain/Market.gen.ts';

import type {messageIn as Messages_messageIn} from '../src/ws/Messages.gen.ts';

import type {metricsHistory as Metrics_metricsHistory} from '../src/domain/Metrics.gen.ts';

import type {notification as Notification_notification} from '../src/domain/Notification.gen.ts';

import type {openInterestHistory as Metrics_openInterestHistory} from '../src/domain/Metrics.gen.ts';

import type {orderBookId as Shared_orderBookId} from './Shared.gen.ts';

import type {orderBook as Orderbook_orderBook} from '../src/domain/Orderbook.gen.ts';

import type {orderUpdate as Order_orderUpdate} from '../src/domain/Order.gen.ts';

import type {orderbookDepthResponse as Orderbook_orderbookDepthResponse} from '../src/domain/Orderbook.gen.ts';

import type {orderbookPriceCandle as PriceHistory_orderbookPriceCandle} from '../src/domain/PriceHistory.gen.ts';

import type {orderbookPriceHistoryResponse as PriceHistory_orderbookPriceHistoryResponse} from '../src/domain/PriceHistory.gen.ts';

import type {orderbookTickersResponse as Metrics_orderbookTickersResponse} from '../src/domain/Metrics.gen.ts';

import type {orderbookVolumeMetrics as Metrics_orderbookVolumeMetrics} from '../src/domain/Metrics.gen.ts';

import type {orderbook as Accounts_orderbook} from '../src/program/Accounts.gen.ts';

import type {platformMetrics as Metrics_platformMetrics} from '../src/domain/Metrics.gen.ts';

import type {position as Accounts_position} from '../src/program/Accounts.gen.ts';

import type {positionsResponse as Position_positionsResponse} from '../src/domain/Position.gen.ts';

import type {pubkeyStr as Shared_pubkeyStr} from './Shared.gen.ts';

import type {readyState as Ws_readyState} from '../src/ws/Ws.gen.ts';

import type {redeemResult as Referral_redeemResult} from '../src/domain/Referral.gen.ts';

import type {referralStatus as Referral_referralStatus} from '../src/domain/Referral.gen.ts';

import type {sessionResponse as Auth_sessionResponse} from './Auth.gen.ts';

import type {submitOrderResponse as Order_submitOrderResponse} from '../src/domain/Order.gen.ts';

import type {t as Client_t} from './Client.gen.ts';

import type {t as DepositPriceState_t} from '../src/domain/DepositPriceState.gen.ts';

import type {t as Env_t} from './Env.gen.ts';

import type {t as OrderbookState_t} from '../src/domain/OrderbookState.gen.ts';

import type {t as PriceHistoryState_t} from '../src/domain/PriceHistoryState.gen.ts';

import type {t as SdkError_t} from './SdkError.gen.ts';

import type {t as TradeState_t} from '../src/domain/TradeState.gen.ts';

import type {trade as Trade_trade} from '../src/domain/Trade.gen.ts';

import type {tradesPage as Trade_tradesPage} from '../src/domain/Trade.gen.ts';

import type {triggerOrderResponse as Order_triggerOrderResponse} from '../src/domain/Order.gen.ts';

import type {triggerOrderUpdate as Order_triggerOrderUpdate} from '../src/domain/Order.gen.ts';

import type {triggerOrder as OrderState_triggerOrder} from '../src/domain/OrderState.gen.ts';

import type {uniqueTradersHistory as Metrics_uniqueTradersHistory} from '../src/domain/Metrics.gen.ts';

import type {userMarketBalance as Order_userMarketBalance} from '../src/domain/Order.gen.ts';

import type {userMetrics as Metrics_userMetrics} from '../src/domain/Metrics.gen.ts';

import type {userOrderFillsResponse as Order_userOrderFillsResponse} from '../src/domain/Order.gen.ts';

import type {userOrdersResponse as Order_userOrdersResponse} from '../src/domain/Order.gen.ts';

export abstract class WsClient_wsConnection { protected opaque!: any }; /* simulate opaque types */

export type WsClient_wsMessage = Messages_messageIn;

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

export const AuthClient_login: (client:Client_t, useEmbeddedWallet:(undefined | boolean)) => Promise<Auth_sessionResponse> = TypeScriptApiJS.AuthClient.login as any;

export const AuthClient_checkSession: (client:Client_t, cookieHeader:(undefined | string)) => Promise<Auth_sessionResponse> = TypeScriptApiJS.AuthClient.checkSession as any;

export const AuthClient_logout: (client:Client_t) => Promise<void> = TypeScriptApiJS.AuthClient.logout as any;

export const AuthClient_isAuthenticated: (client:Client_t) => boolean = TypeScriptApiJS.AuthClient.isAuthenticated as any;

export const AuthClient_registerPrivy: (client:Client_t) => Promise<void> = TypeScriptApiJS.AuthClient.registerPrivy as any;

export const AuthClient_disconnectX: (client:Client_t) => Promise<void> = TypeScriptApiJS.AuthClient.disconnectX as any;

export const AuthClient_connectXUrl: (client:Client_t) => string = TypeScriptApiJS.AuthClient.connectXUrl as any;

export const MarketClient_get: (client:Client_t, cursor:(undefined | number), limit:(undefined | number)) => Promise<Market_marketsResult> = TypeScriptApiJS.MarketClient.get as any;

export const MarketClient_featured: (client:Client_t) => Promise<Market_marketSearchResult[]> = TypeScriptApiJS.MarketClient.featured as any;

export const MarketClient_getBySlug: (client:Client_t, slug:string) => Promise<Market_market> = TypeScriptApiJS.MarketClient.getBySlug as any;

export const MarketClient_getByPubkey: (client:Client_t, pubkey:string) => Promise<Market_market> = TypeScriptApiJS.MarketClient.getByPubkey as any;

export const MarketClient_search: (client:Client_t, query:string, limit:(undefined | number)) => Promise<Market_marketSearchResult[]> = TypeScriptApiJS.MarketClient.search as any;

export const MarketClient_globalDepositAssets: (client:Client_t) => Promise<Market_globalDepositAssetsResult> = TypeScriptApiJS.MarketClient.globalDepositAssets as any;

export const MarketClient_depositMints: (client:Client_t, marketPubkey:string) => Promise<Market_depositMintsResponse> = TypeScriptApiJS.MarketClient.depositMints as any;

export const OrderbookClient_get: (client:Client_t, orderbookId:string, depth:(undefined | number)) => Promise<Orderbook_orderbookDepthResponse> = TypeScriptApiJS.OrderbookClient.get as any;

export const TradeClient_forOrderbook: (client:Client_t, orderbookId:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<Trade_tradesPage> = TypeScriptApiJS.TradeClient.forOrderbook as any;

export const TradeClient_forMarket: (client:Client_t, marketPubkey:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<Trade_tradesPage> = TypeScriptApiJS.TradeClient.forMarket as any;

export const OrderClient_forUser: (client:Client_t, limit:(undefined | number), cursor:(undefined | string)) => Promise<Order_userOrdersResponse> = TypeScriptApiJS.OrderClient.forUser as any;

export const OrderClient_submitLimit: (client:Client_t, market:string, baseMint:string, quoteMint:string, side:number, price:string, size:string, baseDecimals:number, quoteDecimals:number, priceDecimals:number, tickSize:number, orderbookId:string, timeInForce:(undefined | Shared_TimeInForce_t)) => Promise<Order_submitOrderResponse> = TypeScriptApiJS.OrderClient.submitLimit as any;

export const OrderClient_submitTrigger: (client:Client_t, market:string, baseMint:string, quoteMint:string, side:number, price:string, size:string, baseDecimals:number, quoteDecimals:number, priceDecimals:number, tickSize:number, orderbookId:string, triggerPrice:number, triggerType:Shared_TriggerType_t, timeInForce:(undefined | Shared_TimeInForce_t)) => Promise<Order_triggerOrderResponse> = TypeScriptApiJS.OrderClient.submitTrigger as any;

export const OrderClient_cancel: (client:Client_t, orderHash:string) => Promise<Order_cancelSuccess> = TypeScriptApiJS.OrderClient.cancel as any;

export const OrderClient_cancelTrigger: (client:Client_t, triggerOrderId:string) => Promise<Order_cancelTriggerSuccess> = TypeScriptApiJS.OrderClient.cancelTrigger as any;

export const OrderClient_cancelAll: (client:Client_t, orderbookId:string) => Promise<Order_cancelAllSuccess> = TypeScriptApiJS.OrderClient.cancelAll as any;

export const OrderClient_fills: (client:Client_t, marketPubkey:(undefined | string), limit:(undefined | number), cursor:(undefined | string)) => Promise<Order_userOrderFillsResponse> = TypeScriptApiJS.OrderClient.fills as any;

export const OrderClient_fillsByWallet: (client:Client_t, walletAddress:string, marketPubkey:(undefined | string), limit:(undefined | number), cursor:(undefined | string)) => Promise<Order_userOrderFillsResponse> = TypeScriptApiJS.OrderClient.fillsByWallet as any;

export const PositionClient_forUser: (client:Client_t, userPubkey:string) => Promise<Position_positionsResponse> = TypeScriptApiJS.PositionClient.forUser as any;

export const PositionClient_forMarket: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<Position_marketPositionsResponse> = TypeScriptApiJS.PositionClient.forMarket as any;

export const PositionClient_mine: (client:Client_t) => Promise<Position_positionsResponse> = TypeScriptApiJS.PositionClient.mine as any;

export const PositionClient_depositTokenBalances: (client:Client_t) => Promise<{[id: string]: Position_depositTokenBalance}> = TypeScriptApiJS.PositionClient.depositTokenBalances as any;

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

export const MetricsClient_platform: (client:Client_t) => Promise<Metrics_platformMetrics> = TypeScriptApiJS.MetricsClient.platform as any;

export const MetricsClient_markets: (client:Client_t) => Promise<Metrics_marketsMetrics> = TypeScriptApiJS.MetricsClient.markets as any;

export const MetricsClient_market: (client:Client_t, marketPubkey:string) => Promise<Metrics_marketDetailMetrics> = TypeScriptApiJS.MetricsClient.market as any;

export const MetricsClient_orderbookTickers: (client:Client_t, depositAsset:(undefined | string)) => Promise<Metrics_orderbookTickersResponse> = TypeScriptApiJS.MetricsClient.orderbookTickers as any;

export const MetricsClient_categories: (client:Client_t) => Promise<Metrics_categoriesMetrics> = TypeScriptApiJS.MetricsClient.categories as any;

export const MetricsClient_depositTokens: (client:Client_t) => Promise<Metrics_depositTokensMetrics> = TypeScriptApiJS.MetricsClient.depositTokens as any;

export const MetricsClient_leaderboard: (client:Client_t, limit:(undefined | number)) => Promise<Metrics_leaderboard> = TypeScriptApiJS.MetricsClient.leaderboard as any;

export const MetricsClient_orderbook: (client:Client_t, orderbookId:string) => Promise<Metrics_orderbookVolumeMetrics> = TypeScriptApiJS.MetricsClient.orderbook as any;

export const MetricsClient_depositTokensVolumeHistory: (client:Client_t, fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics_depositTokenVolumeHistory> = TypeScriptApiJS.MetricsClient.depositTokensVolumeHistory as any;

export const MetricsClient_openInterestHistory: (client:Client_t, fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics_openInterestHistory> = TypeScriptApiJS.MetricsClient.openInterestHistory as any;

export const MetricsClient_uniqueTradersHistory: (client:Client_t, scope:(undefined | Metrics_UniqueTradersHistoryScope_t), scopeKey:(undefined | string), fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics_uniqueTradersHistory> = TypeScriptApiJS.MetricsClient.uniqueTradersHistory as any;

export const MetricsClient_history: (client:Client_t, scope:string, scopeKey:string, resolution:(undefined | Shared_Resolution_t), fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics_metricsHistory> = TypeScriptApiJS.MetricsClient.history as any;

export const MetricsClient_user: (client:Client_t) => Promise<Metrics_userMetrics> = TypeScriptApiJS.MetricsClient.user as any;

export const MetricsClient_userByWallet: (client:Client_t, walletAddress:string) => Promise<Metrics_userMetrics> = TypeScriptApiJS.MetricsClient.userByWallet as any;

export const PriceHistoryClient_get: (client:Client_t, orderbookId:string, resolution:Shared_Resolution_t, fromMs:(undefined | number), toMs:(undefined | number)) => Promise<PriceHistory_orderbookPriceHistoryResponse> = TypeScriptApiJS.PriceHistoryClient.get as any;

export const PriceHistoryClient_lineData: (client:Client_t, orderbookId:string, resolution:Shared_Resolution_t, fromMs:(undefined | number), toMs:(undefined | number), cursor:(undefined | number), limit:(undefined | number)) => Promise<PriceHistory_lineData[]> = TypeScriptApiJS.PriceHistoryClient.lineData as any;

export const PriceHistoryClient_depositAssetSnapshot: (client:Client_t) => Promise<PriceHistory_depositAssetPricesSnapshotResponse> = TypeScriptApiJS.PriceHistoryClient.depositAssetSnapshot as any;

export const NotificationClient_list: (client:Client_t) => Promise<Notification_notification[]> = TypeScriptApiJS.NotificationClient.list as any;

export const NotificationClient_dismiss: (client:Client_t, notificationId:string) => Promise<void> = TypeScriptApiJS.NotificationClient.dismiss as any;

export const ReferralClient_status: (client:Client_t) => Promise<Referral_referralStatus> = TypeScriptApiJS.ReferralClient.status as any;

export const ReferralClient_redeem: (client:Client_t, code:string) => Promise<Referral_redeemResult> = TypeScriptApiJS.ReferralClient.redeem as any;

export const FaucetClient_claim: (client:Client_t, walletAddress:string) => Promise<Faucet_faucetResponse> = TypeScriptApiJS.FaucetClient.claim as any;

export const RpcClient_activeRpc: (client:Client_t) => RpcFailover_activeRpc = TypeScriptApiJS.RpcClient.activeRpc as any;

export const RpcClient_latestBlockhash: (client:Client_t) => Promise<string> = TypeScriptApiJS.RpcClient.latestBlockhash as any;

export const RpcClient_exchange: (client:Client_t) => Promise<Accounts_exchange> = TypeScriptApiJS.RpcClient.exchange as any;

export const RpcClient_market: (client:Client_t, marketPubkey:string) => Promise<Accounts_market> = TypeScriptApiJS.RpcClient.market as any;

export const RpcClient_orderbook: (client:Client_t, baseMint:string, quoteMint:string) => Promise<Accounts_orderbook> = TypeScriptApiJS.RpcClient.orderbook as any;

export const RpcClient_position: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<(undefined | Accounts_position)> = TypeScriptApiJS.RpcClient.position as any;

export const RpcClient_nonce: (client:Client_t, userPubkey:string) => Promise<number> = TypeScriptApiJS.RpcClient.nonce as any;

export const RpcClient_exchangePda: (client:Client_t) => Promise<string> = TypeScriptApiJS.RpcClient.exchangePda as any;

export const RpcClient_marketPda: (client:Client_t, marketId:bigint) => Promise<string> = TypeScriptApiJS.RpcClient.marketPda as any;

export const RpcClient_positionPda: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<string> = TypeScriptApiJS.RpcClient.positionPda as any;

export const RpcClient_globalDepositTokenPda: (client:Client_t, mint:string) => Promise<string> = TypeScriptApiJS.RpcClient.globalDepositTokenPda as any;

export const WsClient_connect: (client:Client_t, onMessage:((_1:Messages_messageIn) => void), onConnected:(undefined | ((() => void))), onError:(undefined | (((_1:SdkError_t) => void)))) => WsClient_wsConnection = TypeScriptApiJS.WsClient.connect as any;

export const WsClient_subscribe: (connection:WsClient_wsConnection, subscription:Subscriptions_SubscribeParams_t) => void = TypeScriptApiJS.WsClient.subscribe as any;

export const WsClient_unsubscribe: (connection:WsClient_wsConnection, subscription:Subscriptions_UnsubscribeParams_t) => void = TypeScriptApiJS.WsClient.unsubscribe as any;

export const WsClient_disconnect: (connection:WsClient_wsConnection) => void = TypeScriptApiJS.WsClient.disconnect as any;

export const WsClient_isConnected: (connection:WsClient_wsConnection) => boolean = TypeScriptApiJS.WsClient.isConnected as any;

export const WsClient_readyState: (connection:WsClient_wsConnection) => Ws_readyState = TypeScriptApiJS.WsClient.readyState as any;

export const WsClient_clearAuthedSubscriptions: (connection:WsClient_wsConnection) => void = TypeScriptApiJS.WsClient.clearAuthedSubscriptions as any;

export const LiveOrderbook_make: (_1:Shared_orderBookId) => OrderbookState_t = TypeScriptApiJS.LiveOrderbook.make as any;

export const LiveOrderbook_apply: (_1:OrderbookState_t, _2:Orderbook_orderBook) => OrderbookState_applyResult = TypeScriptApiJS.LiveOrderbook.apply as any;

export const LiveOrderbook_bestBid: (_1:OrderbookState_t) => (undefined | string) = TypeScriptApiJS.LiveOrderbook.bestBid as any;

export const LiveOrderbook_bestAsk: (_1:OrderbookState_t) => (undefined | string) = TypeScriptApiJS.LiveOrderbook.bestAsk as any;

export const LiveOrderbook_midPrice: (_1:OrderbookState_t) => (undefined | string) = TypeScriptApiJS.LiveOrderbook.midPrice as any;

export const LiveOrderbook_spread: (_1:OrderbookState_t) => (undefined | string) = TypeScriptApiJS.LiveOrderbook.spread as any;

export const LiveOrderbook_bids: (_1:OrderbookState_t) => Array<[string, string]> = TypeScriptApiJS.LiveOrderbook.bids as any;

export const LiveOrderbook_asks: (_1:OrderbookState_t) => Array<[string, string]> = TypeScriptApiJS.LiveOrderbook.asks as any;

export const LiveOrderbook_isEmpty: (_1:OrderbookState_t) => boolean = TypeScriptApiJS.LiveOrderbook.isEmpty as any;

export const LiveOrderbook_seq: (_1:OrderbookState_t) => number = TypeScriptApiJS.LiveOrderbook.seq as any;

export const LiveOrderbook_orderbookId: (_1:OrderbookState_t) => Shared_orderBookId = TypeScriptApiJS.LiveOrderbook.orderbookId as any;

export const LiveOrderbook_clear: (_1:OrderbookState_t) => void = TypeScriptApiJS.LiveOrderbook.clear as any;

export const LivePriceHistory_make: () => PriceHistoryState_t = TypeScriptApiJS.LivePriceHistory.make as any;

export const LivePriceHistory_applySnapshot: (_1:PriceHistoryState_t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t, candles:PriceHistory_orderbookPriceCandle[]) => void = TypeScriptApiJS.LivePriceHistory.applySnapshot as any;

export const LivePriceHistory_applyUpdate: (_1:PriceHistoryState_t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t, candle:PriceHistory_orderbookPriceCandle) => void = TypeScriptApiJS.LivePriceHistory.applyUpdate as any;

export const LivePriceHistory_get: (_1:PriceHistoryState_t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t) => (undefined | PriceHistory_lineData[]) = TypeScriptApiJS.LivePriceHistory.get as any;

export const LivePriceHistory_clear: (_1:PriceHistoryState_t) => void = TypeScriptApiJS.LivePriceHistory.clear as any;

export const LiveDepositPrice_make: () => DepositPriceState_t = TypeScriptApiJS.LiveDepositPrice.make as any;

export const LiveDepositPrice_applySnapshot: (_1:DepositPriceState_t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t, candles:PriceHistory_depositPriceCandle[]) => void = TypeScriptApiJS.LiveDepositPrice.applySnapshot as any;

export const LiveDepositPrice_applyCandle: (_1:DepositPriceState_t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t, candle:PriceHistory_depositPriceCandle) => void = TypeScriptApiJS.LiveDepositPrice.applyCandle as any;

export const LiveDepositPrice_applyPriceTick: (_1:DepositPriceState_t, depositAsset:Shared_pubkeyStr, price:string, eventTime:number) => void = TypeScriptApiJS.LiveDepositPrice.applyPriceTick as any;

export const LiveDepositPrice_applyAssetSnapshot: (_1:DepositPriceState_t, depositAsset:Shared_pubkeyStr, price:string) => void = TypeScriptApiJS.LiveDepositPrice.applyAssetSnapshot as any;

export const LiveDepositPrice_getCandles: (_1:DepositPriceState_t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t) => (undefined | PriceHistory_depositPriceCandle[]) = TypeScriptApiJS.LiveDepositPrice.getCandles as any;

export const LiveDepositPrice_getLatestPrice: (_1:DepositPriceState_t, depositAsset:Shared_pubkeyStr) => (undefined | DepositPriceState_latestDepositPrice) = TypeScriptApiJS.LiveDepositPrice.getLatestPrice as any;

export const LiveDepositPrice_clear: (_1:DepositPriceState_t) => void = TypeScriptApiJS.LiveDepositPrice.clear as any;

export const LiveOpenLimitOrders_make: () => OrderState_UserOpenLimitOrders_t = TypeScriptApiJS.LiveOpenLimitOrders.make as any;

export const LiveOpenLimitOrders_get: (_1:OrderState_UserOpenLimitOrders_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | OrderState_limitOrder[]) = TypeScriptApiJS.LiveOpenLimitOrders.get as any;

export const LiveOpenLimitOrders_getByMarket: (_1:OrderState_UserOpenLimitOrders_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: OrderState_limitOrder[]}) = TypeScriptApiJS.LiveOpenLimitOrders.getByMarket as any;

export const LiveOpenLimitOrders_insert: (_1:OrderState_UserOpenLimitOrders_t, _2:OrderState_limitOrder) => void = TypeScriptApiJS.LiveOpenLimitOrders.insert as any;

export const LiveOpenLimitOrders_upsert: (_1:OrderState_UserOpenLimitOrders_t, _2:Order_orderUpdate) => void = TypeScriptApiJS.LiveOpenLimitOrders.upsert as any;

export const LiveOpenLimitOrders_remove: (_1:OrderState_UserOpenLimitOrders_t, orderHash:string) => void = TypeScriptApiJS.LiveOpenLimitOrders.remove as any;

export const LiveOpenLimitOrders_clear: (_1:OrderState_UserOpenLimitOrders_t) => void = TypeScriptApiJS.LiveOpenLimitOrders.clear as any;

export const LiveOpenLimitOrders_isEmpty: (_1:OrderState_UserOpenLimitOrders_t) => boolean = TypeScriptApiJS.LiveOpenLimitOrders.isEmpty as any;

export const LiveOpenLimitOrders_limitOrderOfUpdate: (_1:Order_orderUpdate) => OrderState_limitOrder = TypeScriptApiJS.LiveOpenLimitOrders.limitOrderOfUpdate as any;

export const LiveOpenLimitOrders_ofSnapshotOrders: (_1:Order_UserSnapshotOrder_t[]) => [OrderState_UserOpenLimitOrders_t, OrderState_UserTriggerOrders_t] = TypeScriptApiJS.LiveOpenLimitOrders.ofSnapshotOrders as any;

export const LiveTriggerOrders_make: () => OrderState_UserTriggerOrders_t = TypeScriptApiJS.LiveTriggerOrders.make as any;

export const LiveTriggerOrders_get: (_1:OrderState_UserTriggerOrders_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | OrderState_triggerOrder[]) = TypeScriptApiJS.LiveTriggerOrders.get as any;

export const LiveTriggerOrders_getByMarket: (_1:OrderState_UserTriggerOrders_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: OrderState_triggerOrder[]}) = TypeScriptApiJS.LiveTriggerOrders.getByMarket as any;

export const LiveTriggerOrders_all: (_1:OrderState_UserTriggerOrders_t) => OrderState_triggerOrder[] = TypeScriptApiJS.LiveTriggerOrders.all as any;

export const LiveTriggerOrders_getById: (_1:OrderState_UserTriggerOrders_t, triggerOrderId:string) => (undefined | OrderState_triggerOrder) = TypeScriptApiJS.LiveTriggerOrders.getById as any;

export const LiveTriggerOrders_insert: (_1:OrderState_UserTriggerOrders_t, _2:OrderState_triggerOrder) => void = TypeScriptApiJS.LiveTriggerOrders.insert as any;

export const LiveTriggerOrders_remove: (_1:OrderState_UserTriggerOrders_t, triggerOrderId:string) => (undefined | OrderState_triggerOrder) = TypeScriptApiJS.LiveTriggerOrders.remove as any;

export const LiveTriggerOrders_clear: (_1:OrderState_UserTriggerOrders_t) => void = TypeScriptApiJS.LiveTriggerOrders.clear as any;

export const LiveTriggerOrders_isEmpty: (_1:OrderState_UserTriggerOrders_t) => boolean = TypeScriptApiJS.LiveTriggerOrders.isEmpty as any;

export const LiveTriggerOrders_size: (_1:OrderState_UserTriggerOrders_t) => number = TypeScriptApiJS.LiveTriggerOrders.size as any;

export const LiveTriggerOrders_triggerOrderOfUpdate: (_1:Order_triggerOrderUpdate) => OrderState_triggerOrder = TypeScriptApiJS.LiveTriggerOrders.triggerOrderOfUpdate as any;

export const LiveTriggerOrders_limitPrice: (_1:OrderState_triggerOrder) => (undefined | string) = TypeScriptApiJS.LiveTriggerOrders.limitPrice as any;

export const LiveTrades_make: (orderbookId:Shared_orderBookId, maxSize:number) => TradeState_t = TypeScriptApiJS.LiveTrades.make as any;

export const LiveTrades_push: (_1:TradeState_t, _2:Trade_trade) => void = TypeScriptApiJS.LiveTrades.push as any;

export const LiveTrades_replace: (_1:TradeState_t, _2:Trade_trade[]) => void = TypeScriptApiJS.LiveTrades.replace as any;

export const LiveTrades_trades: (_1:TradeState_t) => Trade_trade[] = TypeScriptApiJS.LiveTrades.trades as any;

export const LiveTrades_latest: (_1:TradeState_t) => (undefined | Trade_trade) = TypeScriptApiJS.LiveTrades.latest as any;

export const LiveTrades_clear: (_1:TradeState_t) => void = TypeScriptApiJS.LiveTrades.clear as any;

export const LiveTrades_size: (_1:TradeState_t) => number = TypeScriptApiJS.LiveTrades.size as any;

export const LiveTrades_isEmpty: (_1:TradeState_t) => boolean = TypeScriptApiJS.LiveTrades.isEmpty as any;

export const LiveUserBalances_make: () => Position_UserMarketBalanceIndex_t = TypeScriptApiJS.LiveUserBalances.make as any;

export const LiveUserBalances_get: (_1:Position_UserMarketBalanceIndex_t, marketPubkey:Shared_pubkeyStr) => (undefined | Position_UserMarketBalanceIndex_depositAssetBalanceIndex) = TypeScriptApiJS.LiveUserBalances.get as any;

export const LiveUserBalances_insert: (_1:Position_UserMarketBalanceIndex_t, marketPubkey:Shared_pubkeyStr, _3:Position_UserMarketBalanceIndex_depositAssetBalanceIndex) => void = TypeScriptApiJS.LiveUserBalances.insert as any;

export const LiveUserBalances_remove: (_1:Position_UserMarketBalanceIndex_t, marketPubkey:Shared_pubkeyStr) => void = TypeScriptApiJS.LiveUserBalances.remove as any;

export const LiveUserBalances_extend: (_1:Position_UserMarketBalanceIndex_t, _2:Position_UserMarketBalanceIndex_t) => void = TypeScriptApiJS.LiveUserBalances.extend as any;

export const LiveUserBalances_marketPubkeys: (_1:Position_UserMarketBalanceIndex_t) => Shared_pubkeyStr[] = TypeScriptApiJS.LiveUserBalances.marketPubkeys as any;

export const LiveUserBalances_ofMarketBalance: (_1:Order_userMarketBalance) => (undefined | Position_UserMarketBalanceIndex_t) = TypeScriptApiJS.LiveUserBalances.ofMarketBalance as any;

export const LiveUserBalances_ofMarketBalances: (_1:Order_userMarketBalance[]) => Position_UserMarketBalanceIndex_t = TypeScriptApiJS.LiveUserBalances.ofMarketBalances as any;

export const OrderbookClient: { get: (client:Client_t, orderbookId:string, depth:(undefined | number)) => Promise<Orderbook_orderbookDepthResponse> } = TypeScriptApiJS.OrderbookClient as any;

export const OrderClient: {
  submitLimit: (client:Client_t, market:string, baseMint:string, quoteMint:string, side:number, price:string, size:string, baseDecimals:number, quoteDecimals:number, priceDecimals:number, tickSize:number, orderbookId:string, timeInForce:(undefined | Shared_TimeInForce_t)) => Promise<Order_submitOrderResponse>; 
  forUser: (client:Client_t, limit:(undefined | number), cursor:(undefined | string)) => Promise<Order_userOrdersResponse>; 
  cancel: (client:Client_t, orderHash:string) => Promise<Order_cancelSuccess>; 
  fillsByWallet: (client:Client_t, walletAddress:string, marketPubkey:(undefined | string), limit:(undefined | number), cursor:(undefined | string)) => Promise<Order_userOrderFillsResponse>; 
  fills: (client:Client_t, marketPubkey:(undefined | string), limit:(undefined | number), cursor:(undefined | string)) => Promise<Order_userOrderFillsResponse>; 
  submitTrigger: (client:Client_t, market:string, baseMint:string, quoteMint:string, side:number, price:string, size:string, baseDecimals:number, quoteDecimals:number, priceDecimals:number, tickSize:number, orderbookId:string, triggerPrice:number, triggerType:Shared_TriggerType_t, timeInForce:(undefined | Shared_TimeInForce_t)) => Promise<Order_triggerOrderResponse>; 
  cancelAll: (client:Client_t, orderbookId:string) => Promise<Order_cancelAllSuccess>; 
  cancelTrigger: (client:Client_t, triggerOrderId:string) => Promise<Order_cancelTriggerSuccess>
} = TypeScriptApiJS.OrderClient as any;

export const WsClient: {
  unsubscribe: (connection:WsClient_wsConnection, subscription:Subscriptions_UnsubscribeParams_t) => void; 
  isConnected: (connection:WsClient_wsConnection) => boolean; 
  connect: (client:Client_t, onMessage:((_1:Messages_messageIn) => void), onConnected:(undefined | ((() => void))), onError:(undefined | (((_1:SdkError_t) => void)))) => WsClient_wsConnection; 
  subscribe: (connection:WsClient_wsConnection, subscription:Subscriptions_SubscribeParams_t) => void; 
  readyState: (connection:WsClient_wsConnection) => Ws_readyState; 
  disconnect: (connection:WsClient_wsConnection) => void; 
  clearAuthedSubscriptions: (connection:WsClient_wsConnection) => void
} = TypeScriptApiJS.WsClient as any;

export const PriceHistoryClient: {
  lineData: (client:Client_t, orderbookId:string, resolution:Shared_Resolution_t, fromMs:(undefined | number), toMs:(undefined | number), cursor:(undefined | number), limit:(undefined | number)) => Promise<PriceHistory_lineData[]>; 
  get: (client:Client_t, orderbookId:string, resolution:Shared_Resolution_t, fromMs:(undefined | number), toMs:(undefined | number)) => Promise<PriceHistory_orderbookPriceHistoryResponse>; 
  depositAssetSnapshot: (client:Client_t) => Promise<PriceHistory_depositAssetPricesSnapshotResponse>
} = TypeScriptApiJS.PriceHistoryClient as any;

export const FaucetClient: { claim: (client:Client_t, walletAddress:string) => Promise<Faucet_faucetResponse> } = TypeScriptApiJS.FaucetClient as any;

export const AuthClient: {
  logout: (client:Client_t) => Promise<void>; 
  login: (client:Client_t, useEmbeddedWallet:(undefined | boolean)) => Promise<Auth_sessionResponse>; 
  disconnectX: (client:Client_t) => Promise<void>; 
  registerPrivy: (client:Client_t) => Promise<void>; 
  getNonce: (client:Client_t) => Promise<string>; 
  isAuthenticated: (client:Client_t) => boolean; 
  checkSession: (client:Client_t, cookieHeader:(undefined | string)) => Promise<Auth_sessionResponse>; 
  connectXUrl: (client:Client_t) => string
} = TypeScriptApiJS.AuthClient as any;

export const NotificationClient: { dismiss: (client:Client_t, notificationId:string) => Promise<void>; list: (client:Client_t) => Promise<Notification_notification[]> } = TypeScriptApiJS.NotificationClient as any;

export const ReferralClient: { status: (client:Client_t) => Promise<Referral_referralStatus>; redeem: (client:Client_t, code:string) => Promise<Referral_redeemResult> } = TypeScriptApiJS.ReferralClient as any;

export const LiveUserBalances: {
  extend: (_1:Position_UserMarketBalanceIndex_t, _2:Position_UserMarketBalanceIndex_t) => void; 
  insert: (_1:Position_UserMarketBalanceIndex_t, marketPubkey:Shared_pubkeyStr, _3:Position_UserMarketBalanceIndex_depositAssetBalanceIndex) => void; 
  ofMarketBalances: (_1:Order_userMarketBalance[]) => Position_UserMarketBalanceIndex_t; 
  get: (_1:Position_UserMarketBalanceIndex_t, marketPubkey:Shared_pubkeyStr) => (undefined | Position_UserMarketBalanceIndex_depositAssetBalanceIndex); 
  remove: (_1:Position_UserMarketBalanceIndex_t, marketPubkey:Shared_pubkeyStr) => void; 
  marketPubkeys: (_1:Position_UserMarketBalanceIndex_t) => Shared_pubkeyStr[]; 
  make: () => Position_UserMarketBalanceIndex_t; 
  ofMarketBalance: (_1:Order_userMarketBalance) => (undefined | Position_UserMarketBalanceIndex_t)
} = TypeScriptApiJS.LiveUserBalances as any;

export const TradeClient: { forOrderbook: (client:Client_t, orderbookId:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<Trade_tradesPage>; forMarket: (client:Client_t, marketPubkey:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<Trade_tradesPage> } = TypeScriptApiJS.TradeClient as any;

export const LiveTrades: {
  push: (_1:TradeState_t, _2:Trade_trade) => void; 
  trades: (_1:TradeState_t) => Trade_trade[]; 
  size: (_1:TradeState_t) => number; 
  latest: (_1:TradeState_t) => (undefined | Trade_trade); 
  make: (orderbookId:Shared_orderBookId, maxSize:number) => TradeState_t; 
  clear: (_1:TradeState_t) => void; 
  replace: (_1:TradeState_t, _2:Trade_trade[]) => void; 
  isEmpty: (_1:TradeState_t) => boolean
} = TypeScriptApiJS.LiveTrades as any;

export const MetricsClient: {
  depositTokens: (client:Client_t) => Promise<Metrics_depositTokensMetrics>; 
  userByWallet: (client:Client_t, walletAddress:string) => Promise<Metrics_userMetrics>; 
  user: (client:Client_t) => Promise<Metrics_userMetrics>; 
  categories: (client:Client_t) => Promise<Metrics_categoriesMetrics>; 
  depositTokensVolumeHistory: (client:Client_t, fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics_depositTokenVolumeHistory>; 
  uniqueTradersHistory: (client:Client_t, scope:(undefined | Metrics_UniqueTradersHistoryScope_t), scopeKey:(undefined | string), fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics_uniqueTradersHistory>; 
  markets: (client:Client_t) => Promise<Metrics_marketsMetrics>; 
  orderbookTickers: (client:Client_t, depositAsset:(undefined | string)) => Promise<Metrics_orderbookTickersResponse>; 
  orderbook: (client:Client_t, orderbookId:string) => Promise<Metrics_orderbookVolumeMetrics>; 
  platform: (client:Client_t) => Promise<Metrics_platformMetrics>; 
  leaderboard: (client:Client_t, limit:(undefined | number)) => Promise<Metrics_leaderboard>; 
  openInterestHistory: (client:Client_t, fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics_openInterestHistory>; 
  market: (client:Client_t, marketPubkey:string) => Promise<Metrics_marketDetailMetrics>; 
  history: (client:Client_t, scope:string, scopeKey:string, resolution:(undefined | Shared_Resolution_t), fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<Metrics_metricsHistory>
} = TypeScriptApiJS.MetricsClient as any;

export const MarketClient: {
  globalDepositAssets: (client:Client_t) => Promise<Market_globalDepositAssetsResult>; 
  get: (client:Client_t, cursor:(undefined | number), limit:(undefined | number)) => Promise<Market_marketsResult>; 
  search: (client:Client_t, query:string, limit:(undefined | number)) => Promise<Market_marketSearchResult[]>; 
  featured: (client:Client_t) => Promise<Market_marketSearchResult[]>; 
  getByPubkey: (client:Client_t, pubkey:string) => Promise<Market_market>; 
  getBySlug: (client:Client_t, slug:string) => Promise<Market_market>; 
  depositMints: (client:Client_t, marketPubkey:string) => Promise<Market_depositMintsResponse>
} = TypeScriptApiJS.MarketClient as any;

export const RpcClient: {
  globalDepositTokenPda: (client:Client_t, mint:string) => Promise<string>; 
  nonce: (client:Client_t, userPubkey:string) => Promise<number>; 
  marketPda: (client:Client_t, marketId:bigint) => Promise<string>; 
  position: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<(undefined | Accounts_position)>; 
  exchangePda: (client:Client_t) => Promise<string>; 
  positionPda: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<string>; 
  exchange: (client:Client_t) => Promise<Accounts_exchange>; 
  orderbook: (client:Client_t, baseMint:string, quoteMint:string) => Promise<Accounts_orderbook>; 
  latestBlockhash: (client:Client_t) => Promise<string>; 
  activeRpc: (client:Client_t) => RpcFailover_activeRpc; 
  market: (client:Client_t, marketPubkey:string) => Promise<Accounts_market>
} = TypeScriptApiJS.RpcClient as any;

export const LiveOrderbook: {
  seq: (_1:OrderbookState_t) => number; 
  spread: (_1:OrderbookState_t) => (undefined | string); 
  bestAsk: (_1:OrderbookState_t) => (undefined | string); 
  orderbookId: (_1:OrderbookState_t) => Shared_orderBookId; 
  midPrice: (_1:OrderbookState_t) => (undefined | string); 
  asks: (_1:OrderbookState_t) => Array<[string, string]>; 
  apply: (_1:OrderbookState_t, _2:Orderbook_orderBook) => OrderbookState_applyResult; 
  make: (_1:Shared_orderBookId) => OrderbookState_t; 
  bestBid: (_1:OrderbookState_t) => (undefined | string); 
  clear: (_1:OrderbookState_t) => void; 
  bids: (_1:OrderbookState_t) => Array<[string, string]>; 
  isEmpty: (_1:OrderbookState_t) => boolean
} = TypeScriptApiJS.LiveOrderbook as any;

export const LiveDepositPrice: {
  applyPriceTick: (_1:DepositPriceState_t, depositAsset:Shared_pubkeyStr, price:string, eventTime:number) => void; 
  applyAssetSnapshot: (_1:DepositPriceState_t, depositAsset:Shared_pubkeyStr, price:string) => void; 
  applySnapshot: (_1:DepositPriceState_t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t, candles:PriceHistory_depositPriceCandle[]) => void; 
  applyCandle: (_1:DepositPriceState_t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t, candle:PriceHistory_depositPriceCandle) => void; 
  getCandles: (_1:DepositPriceState_t, depositAsset:Shared_pubkeyStr, resolution:Shared_Resolution_t) => (undefined | PriceHistory_depositPriceCandle[]); 
  make: () => DepositPriceState_t; 
  getLatestPrice: (_1:DepositPriceState_t, depositAsset:Shared_pubkeyStr) => (undefined | DepositPriceState_latestDepositPrice); 
  clear: (_1:DepositPriceState_t) => void
} = TypeScriptApiJS.LiveDepositPrice as any;

export const LiveTriggerOrders: {
  insert: (_1:OrderState_UserTriggerOrders_t, _2:OrderState_triggerOrder) => void; 
  triggerOrderOfUpdate: (_1:Order_triggerOrderUpdate) => OrderState_triggerOrder; 
  size: (_1:OrderState_UserTriggerOrders_t) => number; 
  get: (_1:OrderState_UserTriggerOrders_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | OrderState_triggerOrder[]); 
  remove: (_1:OrderState_UserTriggerOrders_t, triggerOrderId:string) => (undefined | OrderState_triggerOrder); 
  getById: (_1:OrderState_UserTriggerOrders_t, triggerOrderId:string) => (undefined | OrderState_triggerOrder); 
  limitPrice: (_1:OrderState_triggerOrder) => (undefined | string); 
  make: () => OrderState_UserTriggerOrders_t; 
  clear: (_1:OrderState_UserTriggerOrders_t) => void; 
  getByMarket: (_1:OrderState_UserTriggerOrders_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: OrderState_triggerOrder[]}); 
  all: (_1:OrderState_UserTriggerOrders_t) => OrderState_triggerOrder[]; 
  isEmpty: (_1:OrderState_UserTriggerOrders_t) => boolean
} = TypeScriptApiJS.LiveTriggerOrders as any;

export const LivePriceHistory: {
  get: (_1:PriceHistoryState_t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t) => (undefined | PriceHistory_lineData[]); 
  applySnapshot: (_1:PriceHistoryState_t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t, candles:PriceHistory_orderbookPriceCandle[]) => void; 
  applyUpdate: (_1:PriceHistoryState_t, orderbookId:Shared_orderBookId, resolution:Shared_Resolution_t, candle:PriceHistory_orderbookPriceCandle) => void; 
  make: () => PriceHistoryState_t; 
  clear: (_1:PriceHistoryState_t) => void
} = TypeScriptApiJS.LivePriceHistory as any;

export const PositionClient: {
  redeemWinnings: (client:Client_t, market:string, mint:string, amount:bigint, outcomeIndex:number) => Promise<string>; 
  extendPositionTokens: (client:Client_t, user:string, market:string, lookupTable:string, depositMints:string[], numOutcomes:number) => Promise<string>; 
  mine: (client:Client_t) => Promise<Position_positionsResponse>; 
  forUser: (client:Client_t, userPubkey:string) => Promise<Position_positionsResponse>; 
  deposit: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string>; 
  depositTokenBalances: (client:Client_t) => Promise<{[id: string]: Position_depositTokenBalance}>; 
  depositToGlobal: (client:Client_t, mint:string, amount:bigint) => Promise<string>; 
  closePositionTokenAccounts: (client:Client_t, market:string, position:string, depositMints:string[], numOutcomes:number) => Promise<string>; 
  merge: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string>; 
  globalToMarketDeposit: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string>; 
  withdrawFromGlobal: (client:Client_t, mint:string, amount:bigint) => Promise<string>; 
  closePositionAlt: (client:Client_t, position:string, market:string, lookupTable:string) => Promise<string>; 
  withdrawFromPosition: (client:Client_t, market:string, mint:string, amount:bigint, outcomeIndex:number) => Promise<string>; 
  forMarket: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<Position_marketPositionsResponse>
} = TypeScriptApiJS.PositionClient as any;

export const LiveOpenLimitOrders: {
  upsert: (_1:OrderState_UserOpenLimitOrders_t, _2:Order_orderUpdate) => void; 
  insert: (_1:OrderState_UserOpenLimitOrders_t, _2:OrderState_limitOrder) => void; 
  get: (_1:OrderState_UserOpenLimitOrders_t, marketPubkey:Shared_pubkeyStr, orderbookId:Shared_orderBookId) => (undefined | OrderState_limitOrder[]); 
  remove: (_1:OrderState_UserOpenLimitOrders_t, orderHash:string) => void; 
  limitOrderOfUpdate: (_1:Order_orderUpdate) => OrderState_limitOrder; 
  ofSnapshotOrders: (_1:Order_UserSnapshotOrder_t[]) => [OrderState_UserOpenLimitOrders_t, OrderState_UserTriggerOrders_t]; 
  make: () => OrderState_UserOpenLimitOrders_t; 
  clear: (_1:OrderState_UserOpenLimitOrders_t) => void; 
  getByMarket: (_1:OrderState_UserOpenLimitOrders_t, marketPubkey:Shared_pubkeyStr) => (undefined | {[id: string]: OrderState_limitOrder[]}); 
  isEmpty: (_1:OrderState_UserOpenLimitOrders_t) => boolean
} = TypeScriptApiJS.LiveOpenLimitOrders as any;
