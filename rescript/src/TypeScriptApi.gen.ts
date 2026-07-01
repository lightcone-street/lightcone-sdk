/* TypeScript file generated from TypeScriptApi.res by genType. */

/* eslint-disable */
/* tslint:disable */

import * as TypeScriptApiJS from './TypeScriptApi.res.mjs';

import type {DepositSource_t as Shared_DepositSource_t} from './Shared.gen.ts';

import type {Resolution_t as Shared_Resolution_t} from './Shared.gen.ts';

import type {SubscribeParams_t as Subscriptions_SubscribeParams_t} from '../src/ws/Subscriptions.gen.ts';

import type {TimeInForce_t as Shared_TimeInForce_t} from './Shared.gen.ts';

import type {UniqueTradersHistoryScope_t as Metrics_UniqueTradersHistoryScope_t} from '../src/domain/Metrics.gen.ts';

import type {UnsubscribeParams_t as Subscriptions_UnsubscribeParams_t} from '../src/ws/Subscriptions.gen.ts';

import type {activeRpc as RpcFailover_activeRpc} from './RpcFailover.gen.ts';

import type {cancelAllSuccess as Order_cancelAllSuccess} from '../src/domain/Order.gen.ts';

import type {cancelSuccess as Order_cancelSuccess} from '../src/domain/Order.gen.ts';

import type {categoriesMetrics as Metrics_categoriesMetrics} from '../src/domain/Metrics.gen.ts';

import type {depositAssetPricesSnapshotResponse as PriceHistory_depositAssetPricesSnapshotResponse} from '../src/domain/PriceHistory.gen.ts';

import type {depositMintsResponse as Market_depositMintsResponse} from '../src/domain/Market.gen.ts';

import type {depositTokenBalance as Position_depositTokenBalance} from '../src/domain/Position.gen.ts';

import type {depositTokenVolumeHistory as Metrics_depositTokenVolumeHistory} from '../src/domain/Metrics.gen.ts';

import type {depositTokensMetrics as Metrics_depositTokensMetrics} from '../src/domain/Metrics.gen.ts';

import type {exchange as Accounts_exchange} from '../src/program/Accounts.gen.ts';

import type {faucetResponse as Faucet_faucetResponse} from '../src/domain/Faucet.gen.ts';

import type {globalDepositAssetsResult as Market_globalDepositAssetsResult} from '../src/domain/Market.gen.ts';

import type {leaderboard as Metrics_leaderboard} from '../src/domain/Metrics.gen.ts';

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

import type {orderbookDepthResponse as Orderbook_orderbookDepthResponse} from '../src/domain/Orderbook.gen.ts';

import type {orderbookPriceHistoryResponse as PriceHistory_orderbookPriceHistoryResponse} from '../src/domain/PriceHistory.gen.ts';

import type {orderbookTickersResponse as Metrics_orderbookTickersResponse} from '../src/domain/Metrics.gen.ts';

import type {orderbookVolumeMetrics as Metrics_orderbookVolumeMetrics} from '../src/domain/Metrics.gen.ts';

import type {orderbook as Accounts_orderbook} from '../src/program/Accounts.gen.ts';

import type {platformMetrics as Metrics_platformMetrics} from '../src/domain/Metrics.gen.ts';

import type {position as Accounts_position} from '../src/program/Accounts.gen.ts';

import type {positionsResponse as Position_positionsResponse} from '../src/domain/Position.gen.ts';

import type {redeemResult as Referral_redeemResult} from '../src/domain/Referral.gen.ts';

import type {referralStatus as Referral_referralStatus} from '../src/domain/Referral.gen.ts';

import type {sessionResponse as Auth_sessionResponse} from './Auth.gen.ts';

import type {submitOrderResponse as Order_submitOrderResponse} from '../src/domain/Order.gen.ts';

import type {t as Client_t} from './Client.gen.ts';

import type {t as Env_t} from './Env.gen.ts';

import type {t as SdkError_t} from './SdkError.gen.ts';

import type {tradesPage as Trade_tradesPage} from '../src/domain/Trade.gen.ts';

import type {uniqueTradersHistory as Metrics_uniqueTradersHistory} from '../src/domain/Metrics.gen.ts';

import type {userMetrics as Metrics_userMetrics} from '../src/domain/Metrics.gen.ts';

import type {userOrdersResponse as Order_userOrdersResponse} from '../src/domain/Order.gen.ts';

export abstract class WsClient_wsConnection { protected opaque!: any }; /* simulate opaque types */

export type WsClient_wsMessage = Messages_messageIn;

export type WsClient_wsSubscription = Subscriptions_SubscribeParams_t;

export type WsClient_wsUnsubscription = Subscriptions_UnsubscribeParams_t;

export const make: (env:(undefined | Env_t), baseUrl:(undefined | string), wsUrl:(undefined | string), rpcUrl:(undefined | string), backupRpcUrl:(undefined | string), programId:(undefined | string), depositSource:(undefined | Shared_DepositSource_t), _8:void) => Client_t = TypeScriptApiJS.make as any;

export const makeForEnv: (env:Env_t) => Client_t = TypeScriptApiJS.makeForEnv as any;

export const useNativeSigner: (client:Client_t, secretKey:Uint8Array) => Promise<void> = TypeScriptApiJS.useNativeSigner as any;

export const signerAddress: (client:Client_t) => (undefined | string) = TypeScriptApiJS.signerAddress as any;

export const unwrap: <T1>(_1:Promise<
    { TAG: "Ok"; _0: T1 }
  | { TAG: "Error"; _0: SdkError_t }>) => Promise<T1> = TypeScriptApiJS.unwrap as any;

export const AuthClient_getNonce: (client:Client_t) => Promise<string> = TypeScriptApiJS.AuthClient.getNonce as any;

export const AuthClient_login: (client:Client_t, useEmbeddedWallet:(undefined | boolean)) => Promise<Auth_sessionResponse> = TypeScriptApiJS.AuthClient.login as any;

export const AuthClient_checkSession: (client:Client_t, cookieHeader:(undefined | string)) => Promise<Auth_sessionResponse> = TypeScriptApiJS.AuthClient.checkSession as any;

export const AuthClient_logout: (client:Client_t) => Promise<void> = TypeScriptApiJS.AuthClient.logout as any;

export const AuthClient_isAuthenticated: (client:Client_t) => boolean = TypeScriptApiJS.AuthClient.isAuthenticated as any;

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

export const OrderClient_cancel: (client:Client_t, orderHash:string) => Promise<Order_cancelSuccess> = TypeScriptApiJS.OrderClient.cancel as any;

export const OrderClient_cancelAll: (client:Client_t, orderbookId:string) => Promise<Order_cancelAllSuccess> = TypeScriptApiJS.OrderClient.cancelAll as any;

export const PositionClient_forUser: (client:Client_t, userPubkey:string) => Promise<Position_positionsResponse> = TypeScriptApiJS.PositionClient.forUser as any;

export const PositionClient_forMarket: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<Position_marketPositionsResponse> = TypeScriptApiJS.PositionClient.forMarket as any;

export const PositionClient_mine: (client:Client_t) => Promise<Position_positionsResponse> = TypeScriptApiJS.PositionClient.mine as any;

export const PositionClient_depositTokenBalances: (client:Client_t) => Promise<{[id: string]: Position_depositTokenBalance}> = TypeScriptApiJS.PositionClient.depositTokenBalances as any;

export const PositionClient_depositToGlobal: (client:Client_t, mint:string, amount:bigint) => Promise<string> = TypeScriptApiJS.PositionClient.depositToGlobal as any;

export const PositionClient_withdrawFromGlobal: (client:Client_t, mint:string, amount:bigint) => Promise<string> = TypeScriptApiJS.PositionClient.withdrawFromGlobal as any;

export const PositionClient_globalToMarketDeposit: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string> = TypeScriptApiJS.PositionClient.globalToMarketDeposit as any;

export const PositionClient_merge: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string> = TypeScriptApiJS.PositionClient.merge as any;

export const PositionClient_redeemWinnings: (client:Client_t, market:string, mint:string, amount:bigint, outcomeIndex:number) => Promise<string> = TypeScriptApiJS.PositionClient.redeemWinnings as any;

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

export const OrderbookClient: { get: (client:Client_t, orderbookId:string, depth:(undefined | number)) => Promise<Orderbook_orderbookDepthResponse> } = TypeScriptApiJS.OrderbookClient as any;

export const OrderClient: {
  submitLimit: (client:Client_t, market:string, baseMint:string, quoteMint:string, side:number, price:string, size:string, baseDecimals:number, quoteDecimals:number, priceDecimals:number, tickSize:number, orderbookId:string, timeInForce:(undefined | Shared_TimeInForce_t)) => Promise<Order_submitOrderResponse>; 
  forUser: (client:Client_t, limit:(undefined | number), cursor:(undefined | string)) => Promise<Order_userOrdersResponse>; 
  cancel: (client:Client_t, orderHash:string) => Promise<Order_cancelSuccess>; 
  cancelAll: (client:Client_t, orderbookId:string) => Promise<Order_cancelAllSuccess>
} = TypeScriptApiJS.OrderClient as any;

export const WsClient: {
  unsubscribe: (connection:WsClient_wsConnection, subscription:Subscriptions_UnsubscribeParams_t) => void; 
  isConnected: (connection:WsClient_wsConnection) => boolean; 
  connect: (client:Client_t, onMessage:((_1:Messages_messageIn) => void), onConnected:(undefined | ((() => void))), onError:(undefined | (((_1:SdkError_t) => void)))) => WsClient_wsConnection; 
  subscribe: (connection:WsClient_wsConnection, subscription:Subscriptions_SubscribeParams_t) => void; 
  disconnect: (connection:WsClient_wsConnection) => void
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
  getNonce: (client:Client_t) => Promise<string>; 
  isAuthenticated: (client:Client_t) => boolean; 
  checkSession: (client:Client_t, cookieHeader:(undefined | string)) => Promise<Auth_sessionResponse>
} = TypeScriptApiJS.AuthClient as any;

export const NotificationClient: { dismiss: (client:Client_t, notificationId:string) => Promise<void>; list: (client:Client_t) => Promise<Notification_notification[]> } = TypeScriptApiJS.NotificationClient as any;

export const ReferralClient: { status: (client:Client_t) => Promise<Referral_referralStatus>; redeem: (client:Client_t, code:string) => Promise<Referral_redeemResult> } = TypeScriptApiJS.ReferralClient as any;

export const TradeClient: { forOrderbook: (client:Client_t, orderbookId:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<Trade_tradesPage>; forMarket: (client:Client_t, marketPubkey:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<Trade_tradesPage> } = TypeScriptApiJS.TradeClient as any;

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

export const PositionClient: {
  redeemWinnings: (client:Client_t, market:string, mint:string, amount:bigint, outcomeIndex:number) => Promise<string>; 
  mine: (client:Client_t) => Promise<Position_positionsResponse>; 
  forUser: (client:Client_t, userPubkey:string) => Promise<Position_positionsResponse>; 
  depositTokenBalances: (client:Client_t) => Promise<{[id: string]: Position_depositTokenBalance}>; 
  depositToGlobal: (client:Client_t, mint:string, amount:bigint) => Promise<string>; 
  merge: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string>; 
  globalToMarketDeposit: (client:Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string>; 
  withdrawFromGlobal: (client:Client_t, mint:string, amount:bigint) => Promise<string>; 
  forMarket: (client:Client_t, userPubkey:string, marketPubkey:string) => Promise<Position_marketPositionsResponse>
} = TypeScriptApiJS.PositionClient as any;
