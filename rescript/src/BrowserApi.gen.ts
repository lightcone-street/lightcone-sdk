/* TypeScript file generated from BrowserApi.res by genType. */

/* eslint-disable */
/* tslint:disable */

import * as BrowserApiJS from './BrowserApi.res.mjs';

import type {Accounts_Exchange_t as LightconeSdk_Accounts_Exchange_t} from './LightconeSdk.gen.ts';

import type {Accounts_Market_t as LightconeSdk_Accounts_Market_t} from './LightconeSdk.gen.ts';

import type {Accounts_Orderbook_t as LightconeSdk_Accounts_Orderbook_t} from './LightconeSdk.gen.ts';

import type {Accounts_Position_t as LightconeSdk_Accounts_Position_t} from './LightconeSdk.gen.ts';

import type {Auth__Model_Credentials_t as LightconeSdk_Auth__Model_Credentials_t} from './LightconeSdk.gen.ts';

import type {Auth__Model_Method_t as LightconeSdk_Auth__Model_Method_t} from './LightconeSdk.gen.ts';

import type {Auth__Model_RegisterPrivyConflict_t as LightconeSdk_Auth__Model_RegisterPrivyConflict_t} from './LightconeSdk.gen.ts';

import type {Auth__Model_RegisterPrivyRequest_t as LightconeSdk_Auth__Model_RegisterPrivyRequest_t} from './LightconeSdk.gen.ts';

import type {Auth__Model_Session_t as LightconeSdk_Auth__Model_Session_t} from './LightconeSdk.gen.ts';

import type {Auth__Model_User_t as LightconeSdk_Auth__Model_User_t} from './LightconeSdk.gen.ts';

import type {Client_t as LightconeSdk_Client_t} from './LightconeSdk.gen.ts';

import type {Env_t as LightconeSdk_Env_t} from './LightconeSdk.gen.ts';

import type {Faucet__Raw_Response_t as LightconeSdk_Faucet__Raw_Response_t} from './LightconeSdk.gen.ts';

import type {Market__Model_FavoriteMarketUpdate_t as LightconeSdk_Market__Model_FavoriteMarketUpdate_t} from './LightconeSdk.gen.ts';

import type {Market__Model_FavoriteMarkets_t as LightconeSdk_Market__Model_FavoriteMarkets_t} from './LightconeSdk.gen.ts';

import type {Market__Model_GlobalDepositAssetsResult_t as LightconeSdk_Market__Model_GlobalDepositAssetsResult_t} from './LightconeSdk.gen.ts';

import type {Market__Model_MarketsResult_t as LightconeSdk_Market__Model_MarketsResult_t} from './LightconeSdk.gen.ts';

import type {Market__Model_t as LightconeSdk_Market__Model_t} from './LightconeSdk.gen.ts';

import type {Market__Raw_DepositMintsResponse_t as LightconeSdk_Market__Raw_DepositMintsResponse_t} from './LightconeSdk.gen.ts';

import type {Market__Raw_MarketSearchResult_t as LightconeSdk_Market__Raw_MarketSearchResult_t} from './LightconeSdk.gen.ts';

import type {Messages_t as LightconeSdk_Messages_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_Categories_t as LightconeSdk_Metrics__Raw_Categories_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_DepositTokenVolumeHistory_t as LightconeSdk_Metrics__Raw_DepositTokenVolumeHistory_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_DepositTokens_t as LightconeSdk_Metrics__Raw_DepositTokens_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_History_t as LightconeSdk_Metrics__Raw_History_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_Leaderboard_t as LightconeSdk_Metrics__Raw_Leaderboard_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_MarketDetail_t as LightconeSdk_Metrics__Raw_MarketDetail_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_Markets_t as LightconeSdk_Metrics__Raw_Markets_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_OpenInterestHistory_t as LightconeSdk_Metrics__Raw_OpenInterestHistory_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_OrderbookTickersResponse_t as LightconeSdk_Metrics__Raw_OrderbookTickersResponse_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_OrderbookVolume_t as LightconeSdk_Metrics__Raw_OrderbookVolume_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_Platform_t as LightconeSdk_Metrics__Raw_Platform_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_UniqueTradersHistoryScope_t as LightconeSdk_Metrics__Raw_UniqueTradersHistoryScope_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_UniqueTradersHistory_t as LightconeSdk_Metrics__Raw_UniqueTradersHistory_t} from './LightconeSdk.gen.ts';

import type {Metrics__Raw_User_t as LightconeSdk_Metrics__Raw_User_t} from './LightconeSdk.gen.ts';

import type {Notification__Model_t as LightconeSdk_Notification__Model_t} from './LightconeSdk.gen.ts';

import type {Order__Model_Limit_t as LightconeSdk_Order__Model_Limit_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_CancelAllSuccess_t as LightconeSdk_Order__Raw_CancelAllSuccess_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_CancelSuccess_t as LightconeSdk_Order__Raw_CancelSuccess_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_SnapshotOrder_t as LightconeSdk_Order__Raw_SnapshotOrder_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_SubmitResponse_t as LightconeSdk_Order__Raw_SubmitResponse_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_Update_t as LightconeSdk_Order__Raw_Update_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_UserFillsResponse_t as LightconeSdk_Order__Raw_UserFillsResponse_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_UserMarketBalance_t as LightconeSdk_Order__Raw_UserMarketBalance_t} from './LightconeSdk.gen.ts';

import type {Order__Raw_UserOrdersResponse_t as LightconeSdk_Order__Raw_UserOrdersResponse_t} from './LightconeSdk.gen.ts';

import type {Order__State_Limits_t as LightconeSdk_Order__State_Limits_t} from './LightconeSdk.gen.ts';

import type {Orderbook__Model_Aggregation_t as LightconeSdk_Orderbook__Model_Aggregation_t} from './LightconeSdk.gen.ts';

import type {Orderbook__Raw_Book_t as LightconeSdk_Orderbook__Raw_Book_t} from './LightconeSdk.gen.ts';

import type {Orderbook__Raw_DepthResponse_t as LightconeSdk_Orderbook__Raw_DepthResponse_t} from './LightconeSdk.gen.ts';

import type {Orderbook__Raw_WsLevel_t as LightconeSdk_Orderbook__Raw_WsLevel_t} from './LightconeSdk.gen.ts';

import type {Orderbook__State_applyResult as LightconeSdk_Orderbook__State_applyResult} from './LightconeSdk.gen.ts';

import type {Orderbook__State_t as LightconeSdk_Orderbook__State_t} from './LightconeSdk.gen.ts';

import type {Position__Raw_DepositTokenBalance_t as LightconeSdk_Position__Raw_DepositTokenBalance_t} from './LightconeSdk.gen.ts';

import type {Position__Raw_DepositTokenBalancesSnapshot_t as LightconeSdk_Position__Raw_DepositTokenBalancesSnapshot_t} from './LightconeSdk.gen.ts';

import type {Position__Raw_MarketPositionsResponse_t as LightconeSdk_Position__Raw_MarketPositionsResponse_t} from './LightconeSdk.gen.ts';

import type {Position__Raw_PositionsResponse_t as LightconeSdk_Position__Raw_PositionsResponse_t} from './LightconeSdk.gen.ts';

import type {Position__Raw_WalletDepositBalancesEvent_t as LightconeSdk_Position__Raw_WalletDepositBalancesEvent_t} from './LightconeSdk.gen.ts';

import type {Position__Sol_Availability_t as LightconeSdk_Position__Sol_Availability_t} from './LightconeSdk.gen.ts';

import type {Position__Sol_CanonicalAccount_t as LightconeSdk_Position__Sol_CanonicalAccount_t} from './LightconeSdk.gen.ts';

import type {Position__Sol_Costs_t as LightconeSdk_Position__Sol_Costs_t} from './LightconeSdk.gen.ts';

import type {Position__Sol_Plan_t as LightconeSdk_Position__Sol_Plan_t} from './LightconeSdk.gen.ts';

import type {Position__State_depositAssetBalanceIndex as LightconeSdk_Position__State_depositAssetBalanceIndex} from './LightconeSdk.gen.ts';

import type {Position__State_t as LightconeSdk_Position__State_t} from './LightconeSdk.gen.ts';

import type {Position__WalletState_applyResult as LightconeSdk_Position__WalletState_applyResult} from './LightconeSdk.gen.ts';

import type {Position__WalletState_solBalanceBreakdown as LightconeSdk_Position__WalletState_solBalanceBreakdown} from './LightconeSdk.gen.ts';

import type {Position__WalletState_t as LightconeSdk_Position__WalletState_t} from './LightconeSdk.gen.ts';

import type {PriceHistory__DepositState_latestPrice as LightconeSdk_PriceHistory__DepositState_latestPrice} from './LightconeSdk.gen.ts';

import type {PriceHistory__DepositState_t as LightconeSdk_PriceHistory__DepositState_t} from './LightconeSdk.gen.ts';

import type {PriceHistory__Model_LineData_t as LightconeSdk_PriceHistory__Model_LineData_t} from './LightconeSdk.gen.ts';

import type {PriceHistory__Raw_DepositCandle_t as LightconeSdk_PriceHistory__Raw_DepositCandle_t} from './LightconeSdk.gen.ts';

import type {PriceHistory__Raw_DepositPricesSnapshotResponse_t as LightconeSdk_PriceHistory__Raw_DepositPricesSnapshotResponse_t} from './LightconeSdk.gen.ts';

import type {PriceHistory__Raw_OrderbookCandle_t as LightconeSdk_PriceHistory__Raw_OrderbookCandle_t} from './LightconeSdk.gen.ts';

import type {PriceHistory__Raw_OrderbookResponse_t as LightconeSdk_PriceHistory__Raw_OrderbookResponse_t} from './LightconeSdk.gen.ts';

import type {PriceHistory__State_t as LightconeSdk_PriceHistory__State_t} from './LightconeSdk.gen.ts';

import type {Referral__Model_RedeemResult_t as LightconeSdk_Referral__Model_RedeemResult_t} from './LightconeSdk.gen.ts';

import type {Referral__Model_Status_t as LightconeSdk_Referral__Model_Status_t} from './LightconeSdk.gen.ts';

import type {RpcFailover_Active_t as LightconeSdk_RpcFailover_Active_t} from './LightconeSdk.gen.ts';

import type {Scaling_OrderbookRules_t as LightconeSdk_Scaling_OrderbookRules_t} from './LightconeSdk.gen.ts';

import type {SdkError_t as LightconeSdk_SdkError_t} from './LightconeSdk.gen.ts';

import type {Shared_DepositSource_t as LightconeSdk_Shared_DepositSource_t} from './LightconeSdk.gen.ts';

import type {Shared_Resolution_t as LightconeSdk_Shared_Resolution_t} from './LightconeSdk.gen.ts';

import type {Shared_TimeInForce_t as LightconeSdk_Shared_TimeInForce_t} from './LightconeSdk.gen.ts';

import type {Shared_orderBookId as LightconeSdk_Shared_orderBookId} from './LightconeSdk.gen.ts';

import type {Shared_pubkeyStr as LightconeSdk_Shared_pubkeyStr} from './LightconeSdk.gen.ts';

import type {SolanaKit_address as LightconeSdk_SolanaKit_address} from './LightconeSdk.gen.ts';

import type {SolanaKit_instruction as LightconeSdk_SolanaKit_instruction} from './LightconeSdk.gen.ts';

import type {Subscriptions_SubscribeParams_t as LightconeSdk_Subscriptions_SubscribeParams_t} from './LightconeSdk.gen.ts';

import type {Subscriptions_UnsubscribeParams_t as LightconeSdk_Subscriptions_UnsubscribeParams_t} from './LightconeSdk.gen.ts';

import type {Trade__Model_Page_t as LightconeSdk_Trade__Model_Page_t} from './LightconeSdk.gen.ts';

import type {Trade__Model_t as LightconeSdk_Trade__Model_t} from './LightconeSdk.gen.ts';

import type {Trade__State_t as LightconeSdk_Trade__State_t} from './LightconeSdk.gen.ts';

import type {Transactions_Confirmed_t as LightconeSdk_Transactions_Confirmed_t} from './LightconeSdk.gen.ts';

import type {Transactions_Simulation_t as LightconeSdk_Transactions_Simulation_t} from './LightconeSdk.gen.ts';

import type {V1Transaction_Context_t as LightconeSdk_V1Transaction_Context_t} from './LightconeSdk.gen.ts';

import type {V1Transaction_Resources_t as LightconeSdk_V1Transaction_Resources_t} from './LightconeSdk.gen.ts';

import type {V1Transaction_t as LightconeSdk_V1Transaction_t} from './LightconeSdk.gen.ts';

import type {Ws_ReadyState_t as LightconeSdk_Ws_ReadyState_t} from './LightconeSdk.gen.ts';

import type {Ws_t as LightconeSdk_Ws_t} from './LightconeSdk.gen.ts';

export type WsClient_wsConnection = LightconeSdk_Ws_t;

export type WsClient_wsMessage = LightconeSdk_Messages_t;

export type WsClient_wsSubscription = LightconeSdk_Subscriptions_SubscribeParams_t;

export type WsClient_wsUnsubscription = LightconeSdk_Subscriptions_UnsubscribeParams_t;

export const make: (env:(undefined | LightconeSdk_Env_t), baseUrl:(undefined | string), wsUrl:(undefined | string), rpcUrl:(undefined | string), backupRpcUrl:(undefined | string), programId:(undefined | string), depositSource:(undefined | LightconeSdk_Shared_DepositSource_t), transactionResources:(undefined | LightconeSdk_V1Transaction_Resources_t), sponsorshipEnabled:(undefined | boolean), _10:void) => LightconeSdk_Client_t = BrowserApiJS.make as any;

export const makeForEnv: (_1:LightconeSdk_Env_t) => LightconeSdk_Client_t = BrowserApiJS.makeForEnv as any;

export const useExternalSigner: (_1:LightconeSdk_Client_t, address:string, signMessage:((_1:Uint8Array) => Promise<Uint8Array>), signTransaction:((_1:Uint8Array) => Promise<Uint8Array>)) => void = BrowserApiJS.useExternalSigner as any;

export const clearSigningStrategy: (_1:LightconeSdk_Client_t) => void = BrowserApiJS.clearSigningStrategy as any;

export const clearOrderNonce: (_1:LightconeSdk_Client_t) => void = BrowserApiJS.clearOrderNonce as any;

export const signerAddress: (_1:LightconeSdk_Client_t) => (undefined | string) = BrowserApiJS.signerAddress as any;

export const setOrderNonce: (_1:LightconeSdk_Client_t, _2:bigint) => void = BrowserApiJS.setOrderNonce as any;

export const unwrap: <T1>(_1:Promise<
    { TAG: "Ok"; _0: T1 }
  | { TAG: "Error"; _0: LightconeSdk_SdkError_t }>) => Promise<T1> = BrowserApiJS.unwrap as any;

export const MarketClient_favoriteMarkets: (_1:LightconeSdk_Client_t, limit:(undefined | number), cursor:(undefined | bigint), cookieHeader:(undefined | string)) => Promise<LightconeSdk_Market__Model_FavoriteMarkets_t> = BrowserApiJS.MarketClient.favoriteMarkets as any;

export const MarketClient_addFavoriteMarket: (_1:LightconeSdk_Client_t, marketPubkey:string, cookieHeader:(undefined | string)) => Promise<LightconeSdk_Market__Model_FavoriteMarketUpdate_t> = BrowserApiJS.MarketClient.addFavoriteMarket as any;

export const MarketClient_removeFavoriteMarket: (_1:LightconeSdk_Client_t, marketPubkey:string, cookieHeader:(undefined | string)) => Promise<LightconeSdk_Market__Model_FavoriteMarketUpdate_t> = BrowserApiJS.MarketClient.removeFavoriteMarket as any;

export const MarketClient_depositSymbol: (mint:string, symbol:string) => string = BrowserApiJS.MarketClient.depositSymbol as any;

export const MarketClient_get: (_1:LightconeSdk_Client_t, cursor:(undefined | number), limit:(undefined | number)) => Promise<LightconeSdk_Market__Model_MarketsResult_t> = BrowserApiJS.MarketClient.get as any;

export const MarketClient_featured: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Market__Raw_MarketSearchResult_t[]> = BrowserApiJS.MarketClient.featured as any;

export const MarketClient_getBySlug: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Market__Model_t> = BrowserApiJS.MarketClient.getBySlug as any;

export const MarketClient_getByPubkey: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Market__Model_t> = BrowserApiJS.MarketClient.getByPubkey as any;

export const MarketClient_search: (_1:LightconeSdk_Client_t, _2:string, limit:(undefined | number)) => Promise<LightconeSdk_Market__Raw_MarketSearchResult_t[]> = BrowserApiJS.MarketClient.search as any;

export const MarketClient_globalDepositAssets: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Market__Model_GlobalDepositAssetsResult_t> = BrowserApiJS.MarketClient.globalDepositAssets as any;

export const MarketClient_depositMints: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Market__Raw_DepositMintsResponse_t> = BrowserApiJS.MarketClient.depositMints as any;

export const OrderbookClient_decimals: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Scaling_OrderbookRules_t> = BrowserApiJS.OrderbookClient.decimals as any;

export const OrderbookClient_invalidateDecimals: (_1:LightconeSdk_Client_t, _2:string) => void = BrowserApiJS.OrderbookClient.invalidateDecimals as any;

export const OrderbookClient_clearDecimalsCache: (_1:LightconeSdk_Client_t) => void = BrowserApiJS.OrderbookClient.clearDecimalsCache as any;

export const OrderbookClient_getWithAggregation: (_1:LightconeSdk_Client_t, _2:string, _3:LightconeSdk_Orderbook__Model_Aggregation_t, depth:(undefined | number), cookieHeader:(undefined | string)) => Promise<LightconeSdk_Orderbook__Raw_DepthResponse_t> = BrowserApiJS.OrderbookClient.getWithAggregation as any;

export const OrderbookClient_get: (_1:LightconeSdk_Client_t, _2:string, depth:(undefined | number)) => Promise<LightconeSdk_Orderbook__Raw_DepthResponse_t> = BrowserApiJS.OrderbookClient.get as any;

export const TradeClient_forOrderbook: (_1:LightconeSdk_Client_t, _2:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<LightconeSdk_Trade__Model_Page_t> = BrowserApiJS.TradeClient.forOrderbook as any;

export const TradeClient_forMarket: (_1:LightconeSdk_Client_t, _2:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<LightconeSdk_Trade__Model_Page_t> = BrowserApiJS.TradeClient.forMarket as any;

export const OrderClient_forUser: (_1:LightconeSdk_Client_t, limit:(undefined | number), cursor:(undefined | string)) => Promise<LightconeSdk_Order__Raw_UserOrdersResponse_t> = BrowserApiJS.OrderClient.forUser as any;

export const OrderClient_submitLimit: (_1:LightconeSdk_Client_t, market:string, baseMint:string, quoteMint:string, side:number, price:string, size:string, orderbookId:string, timeInForce:(undefined | LightconeSdk_Shared_TimeInForce_t)) => Promise<LightconeSdk_Order__Raw_SubmitResponse_t> = BrowserApiJS.OrderClient.submitLimit as any;

export const OrderClient_cancel: (_1:LightconeSdk_Client_t, orderHash:string) => Promise<LightconeSdk_Order__Raw_CancelSuccess_t> = BrowserApiJS.OrderClient.cancel as any;

export const OrderClient_cancelAll: (_1:LightconeSdk_Client_t, orderbookId:string) => Promise<LightconeSdk_Order__Raw_CancelAllSuccess_t> = BrowserApiJS.OrderClient.cancelAll as any;

export const OrderClient_fills: (_1:LightconeSdk_Client_t, marketPubkey:(undefined | string), limit:(undefined | number), cursor:(undefined | string)) => Promise<LightconeSdk_Order__Raw_UserFillsResponse_t> = BrowserApiJS.OrderClient.fills as any;

export const OrderClient_fillsByWallet: (_1:LightconeSdk_Client_t, walletAddress:string, marketPubkey:(undefined | string), limit:(undefined | number), cursor:(undefined | string)) => Promise<LightconeSdk_Order__Raw_UserFillsResponse_t> = BrowserApiJS.OrderClient.fillsByWallet as any;

export const PositionClient_initPositionTokens: (_1:LightconeSdk_Client_t, payer:string, user:string, market:string, depositMints:string[]) => Promise<string> = BrowserApiJS.PositionClient.initPositionTokens as any;

export const PositionClient_forUser: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Position__Raw_PositionsResponse_t> = BrowserApiJS.PositionClient.forUser as any;

export const PositionClient_forMarket: (_1:LightconeSdk_Client_t, userPubkey:string, marketPubkey:string) => Promise<LightconeSdk_Position__Raw_MarketPositionsResponse_t> = BrowserApiJS.PositionClient.forMarket as any;

export const PositionClient_mine: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Position__Raw_PositionsResponse_t> = BrowserApiJS.PositionClient.mine as any;

export const PositionClient_depositTokenBalances: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Position__Raw_DepositTokenBalancesSnapshot_t> = BrowserApiJS.PositionClient.depositTokenBalances as any;

export const PositionClient_depositToGlobal: (_1:LightconeSdk_Client_t, mint:string, amount:bigint) => Promise<string> = BrowserApiJS.PositionClient.depositToGlobal as any;

export const PositionClient_withdrawFromGlobal: (_1:LightconeSdk_Client_t, mint:string, amount:bigint) => Promise<string> = BrowserApiJS.PositionClient.withdrawFromGlobal as any;

export const PositionClient_globalToMarketDeposit: (_1:LightconeSdk_Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string> = BrowserApiJS.PositionClient.globalToMarketDeposit as any;

export const PositionClient_merge: (_1:LightconeSdk_Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string> = BrowserApiJS.PositionClient.merge as any;

export const PositionClient_redeemWinnings: (_1:LightconeSdk_Client_t, market:string, mint:string, amount:bigint, outcomeIndex:number) => Promise<string> = BrowserApiJS.PositionClient.redeemWinnings as any;

export const PositionClient_deposit: (_1:LightconeSdk_Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string> = BrowserApiJS.PositionClient.deposit as any;

export const PositionClient_withdrawFromPosition: (_1:LightconeSdk_Client_t, market:string, mint:string, amount:bigint, outcomeIndex:number) => Promise<string> = BrowserApiJS.PositionClient.withdrawFromPosition as any;

export const PositionClient_closePositionTokenAccounts: (_1:LightconeSdk_Client_t, market:string, position:string, depositMints:string[], numOutcomes:number) => Promise<string> = BrowserApiJS.PositionClient.closePositionTokenAccounts as any;

export const MetricsClient_platform: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Metrics__Raw_Platform_t> = BrowserApiJS.MetricsClient.platform as any;

export const MetricsClient_markets: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Metrics__Raw_Markets_t> = BrowserApiJS.MetricsClient.markets as any;

export const MetricsClient_market: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Metrics__Raw_MarketDetail_t> = BrowserApiJS.MetricsClient.market as any;

export const MetricsClient_orderbookTickers: (_1:LightconeSdk_Client_t, depositAsset:(undefined | string)) => Promise<LightconeSdk_Metrics__Raw_OrderbookTickersResponse_t> = BrowserApiJS.MetricsClient.orderbookTickers as any;

export const MetricsClient_categories: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Metrics__Raw_Categories_t> = BrowserApiJS.MetricsClient.categories as any;

export const MetricsClient_depositTokens: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Metrics__Raw_DepositTokens_t> = BrowserApiJS.MetricsClient.depositTokens as any;

export const MetricsClient_leaderboard: (_1:LightconeSdk_Client_t, limit:(undefined | number)) => Promise<LightconeSdk_Metrics__Raw_Leaderboard_t> = BrowserApiJS.MetricsClient.leaderboard as any;

export const MetricsClient_orderbook: (_1:LightconeSdk_Client_t, orderbookId:string) => Promise<LightconeSdk_Metrics__Raw_OrderbookVolume_t> = BrowserApiJS.MetricsClient.orderbook as any;

export const MetricsClient_depositTokensVolumeHistory: (_1:LightconeSdk_Client_t, fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<LightconeSdk_Metrics__Raw_DepositTokenVolumeHistory_t> = BrowserApiJS.MetricsClient.depositTokensVolumeHistory as any;

export const MetricsClient_openInterestHistory: (_1:LightconeSdk_Client_t, fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<LightconeSdk_Metrics__Raw_OpenInterestHistory_t> = BrowserApiJS.MetricsClient.openInterestHistory as any;

export const MetricsClient_uniqueTradersHistory: (_1:LightconeSdk_Client_t, scope:(undefined | LightconeSdk_Metrics__Raw_UniqueTradersHistoryScope_t), scopeKey:(undefined | string), fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<LightconeSdk_Metrics__Raw_UniqueTradersHistory_t> = BrowserApiJS.MetricsClient.uniqueTradersHistory as any;

export const MetricsClient_history: (_1:LightconeSdk_Client_t, scope:string, scopeKey:string, resolution:(undefined | LightconeSdk_Shared_Resolution_t), fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<LightconeSdk_Metrics__Raw_History_t> = BrowserApiJS.MetricsClient.history as any;

export const MetricsClient_user: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Metrics__Raw_User_t> = BrowserApiJS.MetricsClient.user as any;

export const MetricsClient_userByWallet: (_1:LightconeSdk_Client_t, walletAddress:string) => Promise<LightconeSdk_Metrics__Raw_User_t> = BrowserApiJS.MetricsClient.userByWallet as any;

export const PriceHistoryClient_get: (_1:LightconeSdk_Client_t, orderbookId:string, resolution:LightconeSdk_Shared_Resolution_t, fromMs:(undefined | number), toMs:(undefined | number)) => Promise<LightconeSdk_PriceHistory__Raw_OrderbookResponse_t> = BrowserApiJS.PriceHistoryClient.get as any;

export const PriceHistoryClient_lineData: (_1:LightconeSdk_Client_t, orderbookId:string, resolution:LightconeSdk_Shared_Resolution_t, fromMs:(undefined | number), toMs:(undefined | number), cursor:(undefined | number), limit:(undefined | number)) => Promise<LightconeSdk_PriceHistory__Model_LineData_t[]> = BrowserApiJS.PriceHistoryClient.lineData as any;

export const PriceHistoryClient_depositAssetSnapshot: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_PriceHistory__Raw_DepositPricesSnapshotResponse_t> = BrowserApiJS.PriceHistoryClient.depositAssetSnapshot as any;

export const NotificationClient_list: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Notification__Model_t[]> = BrowserApiJS.NotificationClient.list as any;

export const NotificationClient_dismiss: (_1:LightconeSdk_Client_t, notificationId:string) => Promise<void> = BrowserApiJS.NotificationClient.dismiss as any;

export const ReferralClient_status: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Referral__Model_Status_t> = BrowserApiJS.ReferralClient.status as any;

export const ReferralClient_redeem: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Referral__Model_RedeemResult_t> = BrowserApiJS.ReferralClient.redeem as any;

export const FaucetClient_claim: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Faucet__Raw_Response_t> = BrowserApiJS.FaucetClient.claim as any;

export const RpcClient_activeRpc: (_1:LightconeSdk_Client_t) => LightconeSdk_RpcFailover_Active_t = BrowserApiJS.RpcClient.activeRpc as any;

export const RpcClient_latestBlockhash: (_1:LightconeSdk_Client_t) => Promise<string> = BrowserApiJS.RpcClient.latestBlockhash as any;

export const RpcClient_exchange: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Accounts_Exchange_t> = BrowserApiJS.RpcClient.exchange as any;

export const RpcClient_market: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Accounts_Market_t> = BrowserApiJS.RpcClient.market as any;

export const RpcClient_orderbook: (_1:LightconeSdk_Client_t, _2:string, _3:string) => Promise<LightconeSdk_Accounts_Orderbook_t> = BrowserApiJS.RpcClient.orderbook as any;

export const RpcClient_position: (_1:LightconeSdk_Client_t, _2:string, _3:string) => Promise<(undefined | LightconeSdk_Accounts_Position_t)> = BrowserApiJS.RpcClient.position as any;

export const RpcClient_nonce: (_1:LightconeSdk_Client_t, _2:string) => Promise<number> = BrowserApiJS.RpcClient.nonce as any;

export const RpcClient_exchangePda: (_1:LightconeSdk_Client_t) => Promise<string> = BrowserApiJS.RpcClient.exchangePda as any;

export const RpcClient_marketPda: (_1:LightconeSdk_Client_t, _2:bigint) => Promise<string> = BrowserApiJS.RpcClient.marketPda as any;

export const RpcClient_positionPda: (_1:LightconeSdk_Client_t, _2:string, _3:string) => Promise<string> = BrowserApiJS.RpcClient.positionPda as any;

export const RpcClient_globalDepositTokenPda: (_1:LightconeSdk_Client_t, _2:string) => Promise<string> = BrowserApiJS.RpcClient.globalDepositTokenPda as any;

export const WsClient_connect: (_1:LightconeSdk_Client_t, onMessage:((_1:LightconeSdk_Messages_t) => void), onConnected:(undefined | ((() => void))), onError:(undefined | (((_1:LightconeSdk_SdkError_t) => void)))) => WsClient_wsConnection = BrowserApiJS.WsClient.connect as any;

export const WsClient_subscribe: (_1:WsClient_wsConnection, _2:LightconeSdk_Subscriptions_SubscribeParams_t) => void = BrowserApiJS.WsClient.subscribe as any;

export const WsClient_unsubscribe: (_1:WsClient_wsConnection, _2:LightconeSdk_Subscriptions_UnsubscribeParams_t) => void = BrowserApiJS.WsClient.unsubscribe as any;

export const WsClient_stop: (_1:LightconeSdk_Ws_t) => void = BrowserApiJS.WsClient.stop as any;

export const WsClient_restartConnection: (_1:LightconeSdk_Ws_t) => void = BrowserApiJS.WsClient.restartConnection as any;

export const WsClient_isStopped: (_1:LightconeSdk_Ws_t) => boolean = BrowserApiJS.WsClient.isStopped as any;

export const WsClient_disconnect: (_1:WsClient_wsConnection) => void = BrowserApiJS.WsClient.disconnect as any;

export const WsClient_isConnected: (_1:WsClient_wsConnection) => boolean = BrowserApiJS.WsClient.isConnected as any;

export const WsClient_readyState: (_1:WsClient_wsConnection) => LightconeSdk_Ws_ReadyState_t = BrowserApiJS.WsClient.readyState as any;

export const WsClient_clearAuthedSubscriptions: (_1:WsClient_wsConnection) => void = BrowserApiJS.WsClient.clearAuthedSubscriptions as any;

export const LiveOrderbook_withAggregation: (_1:LightconeSdk_Shared_orderBookId, _2:LightconeSdk_Orderbook__Model_Aggregation_t) => LightconeSdk_Orderbook__State_t = BrowserApiJS.LiveOrderbook.withAggregation as any;

export const LiveOrderbook_beginGeneration: (_1:LightconeSdk_Orderbook__State_t) => void = BrowserApiJS.LiveOrderbook.beginGeneration as any;

export const LiveOrderbook_bidsRaw: (_1:LightconeSdk_Orderbook__State_t) => LightconeSdk_Orderbook__Raw_WsLevel_t[] = BrowserApiJS.LiveOrderbook.bidsRaw as any;

export const LiveOrderbook_asksRaw: (_1:LightconeSdk_Orderbook__State_t) => LightconeSdk_Orderbook__Raw_WsLevel_t[] = BrowserApiJS.LiveOrderbook.asksRaw as any;

export const LiveOrderbook_make: (_1:LightconeSdk_Shared_orderBookId) => LightconeSdk_Orderbook__State_t = BrowserApiJS.LiveOrderbook.make as any;

export const LiveOrderbook_apply: (_1:LightconeSdk_Orderbook__State_t, _2:LightconeSdk_Orderbook__Raw_Book_t) => LightconeSdk_Orderbook__State_applyResult = BrowserApiJS.LiveOrderbook.apply as any;

export const LiveOrderbook_bestBid: (_1:LightconeSdk_Orderbook__State_t) => (undefined | string) = BrowserApiJS.LiveOrderbook.bestBid as any;

export const LiveOrderbook_bestAsk: (_1:LightconeSdk_Orderbook__State_t) => (undefined | string) = BrowserApiJS.LiveOrderbook.bestAsk as any;

export const LiveOrderbook_midPrice: (_1:LightconeSdk_Orderbook__State_t) => (undefined | string) = BrowserApiJS.LiveOrderbook.midPrice as any;

export const LiveOrderbook_spread: (_1:LightconeSdk_Orderbook__State_t) => (undefined | string) = BrowserApiJS.LiveOrderbook.spread as any;

export const LiveOrderbook_bids: (_1:LightconeSdk_Orderbook__State_t) => Array<[string, string]> = BrowserApiJS.LiveOrderbook.bids as any;

export const LiveOrderbook_asks: (_1:LightconeSdk_Orderbook__State_t) => Array<[string, string]> = BrowserApiJS.LiveOrderbook.asks as any;

export const LiveOrderbook_isEmpty: (_1:LightconeSdk_Orderbook__State_t) => boolean = BrowserApiJS.LiveOrderbook.isEmpty as any;

export const LiveOrderbook_seq: (_1:LightconeSdk_Orderbook__State_t) => bigint = BrowserApiJS.LiveOrderbook.seq as any;

export const LiveOrderbook_orderbookId: (_1:LightconeSdk_Orderbook__State_t) => LightconeSdk_Shared_orderBookId = BrowserApiJS.LiveOrderbook.orderbookId as any;

export const LiveOrderbook_clear: (_1:LightconeSdk_Orderbook__State_t) => void = BrowserApiJS.LiveOrderbook.clear as any;

export const LivePriceHistory_make: () => LightconeSdk_PriceHistory__State_t = BrowserApiJS.LivePriceHistory.make as any;

export const LivePriceHistory_applySnapshot: (_1:LightconeSdk_PriceHistory__State_t, orderbookId:LightconeSdk_Shared_orderBookId, resolution:LightconeSdk_Shared_Resolution_t, candles:LightconeSdk_PriceHistory__Raw_OrderbookCandle_t[]) => void = BrowserApiJS.LivePriceHistory.applySnapshot as any;

export const LivePriceHistory_applyUpdate: (_1:LightconeSdk_PriceHistory__State_t, orderbookId:LightconeSdk_Shared_orderBookId, resolution:LightconeSdk_Shared_Resolution_t, candle:LightconeSdk_PriceHistory__Raw_OrderbookCandle_t) => void = BrowserApiJS.LivePriceHistory.applyUpdate as any;

export const LivePriceHistory_get: (_1:LightconeSdk_PriceHistory__State_t, orderbookId:LightconeSdk_Shared_orderBookId, resolution:LightconeSdk_Shared_Resolution_t) => (undefined | LightconeSdk_PriceHistory__Model_LineData_t[]) = BrowserApiJS.LivePriceHistory.get as any;

export const LivePriceHistory_clear: (_1:LightconeSdk_PriceHistory__State_t) => void = BrowserApiJS.LivePriceHistory.clear as any;

export const LiveDepositPrice_make: () => LightconeSdk_PriceHistory__DepositState_t = BrowserApiJS.LiveDepositPrice.make as any;

export const LiveDepositPrice_applySnapshot: (_1:LightconeSdk_PriceHistory__DepositState_t, depositAsset:LightconeSdk_Shared_pubkeyStr, resolution:LightconeSdk_Shared_Resolution_t, candles:LightconeSdk_PriceHistory__Raw_DepositCandle_t[]) => void = BrowserApiJS.LiveDepositPrice.applySnapshot as any;

export const LiveDepositPrice_applyCandle: (_1:LightconeSdk_PriceHistory__DepositState_t, depositAsset:LightconeSdk_Shared_pubkeyStr, resolution:LightconeSdk_Shared_Resolution_t, candle:LightconeSdk_PriceHistory__Raw_DepositCandle_t) => void = BrowserApiJS.LiveDepositPrice.applyCandle as any;

export const LiveDepositPrice_applyPriceTick: (_1:LightconeSdk_PriceHistory__DepositState_t, depositAsset:LightconeSdk_Shared_pubkeyStr, price:string, eventTime:number) => void = BrowserApiJS.LiveDepositPrice.applyPriceTick as any;

export const LiveDepositPrice_applyAssetSnapshot: (_1:LightconeSdk_PriceHistory__DepositState_t, depositAsset:LightconeSdk_Shared_pubkeyStr, price:string) => void = BrowserApiJS.LiveDepositPrice.applyAssetSnapshot as any;

export const LiveDepositPrice_getCandles: (_1:LightconeSdk_PriceHistory__DepositState_t, depositAsset:LightconeSdk_Shared_pubkeyStr, resolution:LightconeSdk_Shared_Resolution_t) => (undefined | LightconeSdk_PriceHistory__Raw_DepositCandle_t[]) = BrowserApiJS.LiveDepositPrice.getCandles as any;

export const LiveDepositPrice_getLatestPrice: (_1:LightconeSdk_PriceHistory__DepositState_t, depositAsset:LightconeSdk_Shared_pubkeyStr) => (undefined | LightconeSdk_PriceHistory__DepositState_latestPrice) = BrowserApiJS.LiveDepositPrice.getLatestPrice as any;

export const LiveDepositPrice_clear: (_1:LightconeSdk_PriceHistory__DepositState_t) => void = BrowserApiJS.LiveDepositPrice.clear as any;

export const LiveOpenLimitOrders_make: () => LightconeSdk_Order__State_Limits_t = BrowserApiJS.LiveOpenLimitOrders.make as any;

export const LiveOpenLimitOrders_get: (_1:LightconeSdk_Order__State_Limits_t, marketPubkey:LightconeSdk_Shared_pubkeyStr, orderbookId:LightconeSdk_Shared_orderBookId) => (undefined | LightconeSdk_Order__Model_Limit_t[]) = BrowserApiJS.LiveOpenLimitOrders.get as any;

export const LiveOpenLimitOrders_getByMarket: (_1:LightconeSdk_Order__State_Limits_t, marketPubkey:LightconeSdk_Shared_pubkeyStr) => (undefined | {[id: string]: LightconeSdk_Order__Model_Limit_t[]}) = BrowserApiJS.LiveOpenLimitOrders.getByMarket as any;

export const LiveOpenLimitOrders_insert: (_1:LightconeSdk_Order__State_Limits_t, _2:LightconeSdk_Order__Model_Limit_t) => void = BrowserApiJS.LiveOpenLimitOrders.insert as any;

export const LiveOpenLimitOrders_upsert: (_1:LightconeSdk_Order__State_Limits_t, _2:LightconeSdk_Order__Raw_Update_t) => void = BrowserApiJS.LiveOpenLimitOrders.upsert as any;

export const LiveOpenLimitOrders_remove: (_1:LightconeSdk_Order__State_Limits_t, orderHash:string) => void = BrowserApiJS.LiveOpenLimitOrders.remove as any;

export const LiveOpenLimitOrders_clear: (_1:LightconeSdk_Order__State_Limits_t) => void = BrowserApiJS.LiveOpenLimitOrders.clear as any;

export const LiveOpenLimitOrders_isEmpty: (_1:LightconeSdk_Order__State_Limits_t) => boolean = BrowserApiJS.LiveOpenLimitOrders.isEmpty as any;

export const LiveOpenLimitOrders_limitOrderOfUpdate: (_1:LightconeSdk_Order__Raw_Update_t) => LightconeSdk_Order__Model_Limit_t = BrowserApiJS.LiveOpenLimitOrders.limitOrderOfUpdate as any;

export const LiveOpenLimitOrders_ofSnapshotOrders: (_1:LightconeSdk_Order__Raw_SnapshotOrder_t[]) => LightconeSdk_Order__State_Limits_t = BrowserApiJS.LiveOpenLimitOrders.ofSnapshotOrders as any;

export const LiveTrades_make: (orderbookId:LightconeSdk_Shared_orderBookId, maxSize:number) => LightconeSdk_Trade__State_t = BrowserApiJS.LiveTrades.make as any;

export const LiveTrades_push: (_1:LightconeSdk_Trade__State_t, _2:LightconeSdk_Trade__Model_t) => void = BrowserApiJS.LiveTrades.push as any;

export const LiveTrades_replace: (_1:LightconeSdk_Trade__State_t, _2:LightconeSdk_Trade__Model_t[]) => void = BrowserApiJS.LiveTrades.replace as any;

export const LiveTrades_trades: (_1:LightconeSdk_Trade__State_t) => LightconeSdk_Trade__Model_t[] = BrowserApiJS.LiveTrades.trades as any;

export const LiveTrades_latest: (_1:LightconeSdk_Trade__State_t) => (undefined | LightconeSdk_Trade__Model_t) = BrowserApiJS.LiveTrades.latest as any;

export const LiveTrades_clear: (_1:LightconeSdk_Trade__State_t) => void = BrowserApiJS.LiveTrades.clear as any;

export const LiveTrades_size: (_1:LightconeSdk_Trade__State_t) => number = BrowserApiJS.LiveTrades.size as any;

export const LiveTrades_isEmpty: (_1:LightconeSdk_Trade__State_t) => boolean = BrowserApiJS.LiveTrades.isEmpty as any;

export const LiveUserBalances_make: () => LightconeSdk_Position__State_t = BrowserApiJS.LiveUserBalances.make as any;

export const LiveUserBalances_get: (_1:LightconeSdk_Position__State_t, marketPubkey:LightconeSdk_Shared_pubkeyStr) => (undefined | LightconeSdk_Position__State_depositAssetBalanceIndex) = BrowserApiJS.LiveUserBalances.get as any;

export const LiveUserBalances_insert: (_1:LightconeSdk_Position__State_t, marketPubkey:LightconeSdk_Shared_pubkeyStr, _3:LightconeSdk_Position__State_depositAssetBalanceIndex) => void = BrowserApiJS.LiveUserBalances.insert as any;

export const LiveUserBalances_remove: (_1:LightconeSdk_Position__State_t, marketPubkey:LightconeSdk_Shared_pubkeyStr) => void = BrowserApiJS.LiveUserBalances.remove as any;

export const LiveUserBalances_extend: (_1:LightconeSdk_Position__State_t, _2:LightconeSdk_Position__State_t) => void = BrowserApiJS.LiveUserBalances.extend as any;

export const LiveUserBalances_marketPubkeys: (_1:LightconeSdk_Position__State_t) => LightconeSdk_Shared_pubkeyStr[] = BrowserApiJS.LiveUserBalances.marketPubkeys as any;

export const LiveUserBalances_ofMarketBalance: (_1:LightconeSdk_Order__Raw_UserMarketBalance_t) => (undefined | LightconeSdk_Position__State_t) = BrowserApiJS.LiveUserBalances.ofMarketBalance as any;

export const LiveUserBalances_ofMarketBalances: (_1:LightconeSdk_Order__Raw_UserMarketBalance_t[]) => LightconeSdk_Position__State_t = BrowserApiJS.LiveUserBalances.ofMarketBalances as any;

export const LiveWalletBalances_make: () => LightconeSdk_Position__WalletState_t = BrowserApiJS.LiveWalletBalances.make as any;

export const LiveWalletBalances_applyRestSnapshot: (_1:LightconeSdk_Position__WalletState_t, walletAddress:string, _3:LightconeSdk_Position__Raw_DepositTokenBalancesSnapshot_t, minimumSnapshotSlot:(undefined | bigint)) => LightconeSdk_Position__WalletState_applyResult = BrowserApiJS.LiveWalletBalances.applyRestSnapshot as any;

export const LiveWalletBalances_applyEvent: (_1:LightconeSdk_Position__WalletState_t, _2:LightconeSdk_Position__Raw_WalletDepositBalancesEvent_t, minimumSnapshotSlot:(undefined | bigint)) => LightconeSdk_Position__WalletState_applyResult = BrowserApiJS.LiveWalletBalances.applyEvent as any;

export const LiveWalletBalances_balances: (_1:LightconeSdk_Position__WalletState_t) => {[id: string]: LightconeSdk_Position__Raw_DepositTokenBalance_t} = BrowserApiJS.LiveWalletBalances.balances as any;

export const LiveWalletBalances_contextSlot: (_1:LightconeSdk_Position__WalletState_t) => (undefined | bigint) = BrowserApiJS.LiveWalletBalances.contextSlot as any;

export const LiveWalletBalances_walletAddress: (_1:LightconeSdk_Position__WalletState_t) => (undefined | string) = BrowserApiJS.LiveWalletBalances.walletAddress as any;

export const LiveWalletBalances_nativeSolBalance: (_1:LightconeSdk_Position__WalletState_t) => (undefined | string) = BrowserApiJS.LiveWalletBalances.nativeSolBalance as any;

export const LiveWalletBalances_combinedSolBalance: (_1:LightconeSdk_Position__WalletState_t) =>
    { TAG: "Ok"; _0: string }
  | { TAG: "Error"; _0: LightconeSdk_SdkError_t } = BrowserApiJS.LiveWalletBalances.combinedSolBalance as any;

export const LiveWalletBalances_solBalanceBreakdown: (_1:LightconeSdk_Position__WalletState_t) =>
    { TAG: "Ok"; _0: LightconeSdk_Position__WalletState_solBalanceBreakdown }
  | { TAG: "Error"; _0: LightconeSdk_SdkError_t } = BrowserApiJS.LiveWalletBalances.solBalanceBreakdown as any;

export const LiveWalletBalances_clear: (_1:LightconeSdk_Position__WalletState_t) => void = BrowserApiJS.LiveWalletBalances.clear as any;

export const TransactionClient_setResources: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_Resources_t) => void = BrowserApiJS.TransactionClient.setResources as any;

export const TransactionClient_context: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_V1Transaction_Context_t> = BrowserApiJS.TransactionClient.context as any;

export const TransactionClient_build: (_1:LightconeSdk_Client_t, payer:LightconeSdk_SolanaKit_address, instructions:LightconeSdk_SolanaKit_instruction[]) => Promise<LightconeSdk_V1Transaction_t> = BrowserApiJS.TransactionClient.build as any;

export const TransactionClient_importBytes: (_1:Uint8Array, _2:LightconeSdk_V1Transaction_Context_t) => Promise<LightconeSdk_V1Transaction_t> = BrowserApiJS.TransactionClient.importBytes as any;

export const TransactionClient_toWireBytes: (_1:LightconeSdk_V1Transaction_t) => Uint8Array = BrowserApiJS.TransactionClient.toWireBytes as any;

export const TransactionClient_messageBytes: (_1:LightconeSdk_V1Transaction_t) => Uint8Array = BrowserApiJS.TransactionClient.messageBytes as any;

export const TransactionClient_requiredSigners: (_1:LightconeSdk_V1Transaction_t) => LightconeSdk_SolanaKit_address[] = BrowserApiJS.TransactionClient.requiredSigners as any;

export const TransactionClient_estimateFee: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_t) => Promise<bigint> = BrowserApiJS.TransactionClient.estimateFee as any;

export const TransactionClient_simulate: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_t) => Promise<LightconeSdk_Transactions_Simulation_t> = BrowserApiJS.TransactionClient.simulate as any;

export const TransactionClient_sign: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_t) => Promise<LightconeSdk_V1Transaction_t> = BrowserApiJS.TransactionClient.sign as any;

export const TransactionClient_submitSigned: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_t) => Promise<string> = BrowserApiJS.TransactionClient.submitSigned as any;

export const TransactionClient_signAndSubmit: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_t) => Promise<string> = BrowserApiJS.TransactionClient.signAndSubmit as any;

export const TransactionClient_signAndSubmitConfirmed: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_t) => Promise<LightconeSdk_Transactions_Confirmed_t> = BrowserApiJS.TransactionClient.signAndSubmitConfirmed as any;

export const TransactionClient_confirm: (_1:LightconeSdk_Client_t, _2:string, lastValidBlockHeight:(undefined | bigint)) => Promise<LightconeSdk_Transactions_Confirmed_t> = BrowserApiJS.TransactionClient.confirm as any;

export const SolClient_availability: (_1:LightconeSdk_Position__WalletState_solBalanceBreakdown, _2:LightconeSdk_Position__Sol_Costs_t, unwrapAll:(undefined | boolean)) =>
    { TAG: "Ok"; _0: LightconeSdk_Position__Sol_Availability_t }
  | { TAG: "Error"; _0: LightconeSdk_SdkError_t } = BrowserApiJS.SolClient.availability as any;

export const SolClient_canonicalAccount: (_1:LightconeSdk_Client_t, _2:string) => Promise<(undefined | LightconeSdk_Position__Sol_CanonicalAccount_t)> = BrowserApiJS.SolClient.canonicalAccount as any;

export const SolClient_planWrap: (_1:LightconeSdk_Client_t, _2:bigint, _3:LightconeSdk_Position__WalletState_t) => Promise<LightconeSdk_Position__Sol_Plan_t> = BrowserApiJS.SolClient.planWrap as any;

export const SolClient_planUnwrapAll: (_1:LightconeSdk_Client_t, _2:LightconeSdk_Position__WalletState_t) => Promise<LightconeSdk_Position__Sol_Plan_t> = BrowserApiJS.SolClient.planUnwrapAll as any;

export const SolClient_planSplit: (_1:LightconeSdk_Client_t, _2:string, _3:number, _4:bigint, _5:LightconeSdk_Position__WalletState_t) => Promise<LightconeSdk_Position__Sol_Plan_t> = BrowserApiJS.SolClient.planSplit as any;

export const SolClient_planMerge: (_1:LightconeSdk_Client_t, _2:string, _3:number, _4:bigint, _5:LightconeSdk_Position__WalletState_t) => Promise<LightconeSdk_Position__Sol_Plan_t> = BrowserApiJS.SolClient.planMerge as any;

export const SolClient_planRedeem: (_1:LightconeSdk_Client_t, _2:string, _3:number, _4:number, _5:bigint, _6:LightconeSdk_Position__WalletState_t) => Promise<LightconeSdk_Position__Sol_Plan_t> = BrowserApiJS.SolClient.planRedeem as any;

export const SolClient_planNativeWithdrawal: (_1:LightconeSdk_Client_t, _2:string, _3:bigint, _4:LightconeSdk_Position__WalletState_t) => Promise<LightconeSdk_Position__Sol_Plan_t> = BrowserApiJS.SolClient.planNativeWithdrawal as any;

export const SolClient_submit: (_1:LightconeSdk_Client_t, _2:LightconeSdk_Position__Sol_Plan_t) => Promise<LightconeSdk_Transactions_Confirmed_t> = BrowserApiJS.SolClient.submit as any;

export const AuthClient_getNonce: (_1:LightconeSdk_Client_t) => Promise<string> = BrowserApiJS.AuthClient.getNonce as any;

export const AuthClient_checkSession: (_1:LightconeSdk_Client_t, cookieHeader:(undefined | string)) => Promise<LightconeSdk_Auth__Model_Session_t> = BrowserApiJS.AuthClient.checkSession as any;

export const AuthClient_logout: (_1:LightconeSdk_Client_t) => Promise<void> = BrowserApiJS.AuthClient.logout as any;

export const AuthClient_isAuthenticated: (_1:LightconeSdk_Client_t) => boolean = BrowserApiJS.AuthClient.isAuthenticated as any;

export const AuthClient_registerPrivy: (_1:LightconeSdk_Client_t, _2:LightconeSdk_Auth__Model_RegisterPrivyRequest_t) => Promise<LightconeSdk_Auth__Model_Session_t> = BrowserApiJS.AuthClient.registerPrivy as any;

export const AuthClient_disconnectX: (_1:LightconeSdk_Client_t) => Promise<void> = BrowserApiJS.AuthClient.disconnectX as any;

export const AuthClient_connectXUrl: (_1:LightconeSdk_Client_t) => string = BrowserApiJS.AuthClient.connectXUrl as any;

export const AuthClient_credentials: (_1:LightconeSdk_Client_t) => (undefined | LightconeSdk_Auth__Model_Credentials_t) = BrowserApiJS.AuthClient.credentials as any;

export const AuthClient_updateMaxSlippagePreference: (_1:LightconeSdk_Client_t, _2:string) => Promise<string> = BrowserApiJS.AuthClient.updateMaxSlippagePreference as any;

export const AuthClient_setCredentialRestorer: (_1:LightconeSdk_Client_t, _2:(undefined | ((() => Promise<boolean>)))) => void = BrowserApiJS.AuthClient.setCredentialRestorer as any;

export const AuthClient_clearAuth: (_1:LightconeSdk_Client_t) => void = BrowserApiJS.AuthClient.clearAuth as any;

export const AuthClient_tradingWallet: (_1:LightconeSdk_Auth__Model_User_t, _2:LightconeSdk_Auth__Model_Method_t) => string = BrowserApiJS.AuthClient.tradingWallet as any;

export const AuthClient_displayName: (_1:LightconeSdk_Auth__Model_User_t) => string = BrowserApiJS.AuthClient.displayName as any;

export const AuthClient_walletDisplayName: (_1:LightconeSdk_Auth__Model_User_t, _2:LightconeSdk_Auth__Model_Method_t) => string = BrowserApiJS.AuthClient.walletDisplayName as any;

export const AuthClient_avatarUrl: (_1:LightconeSdk_Auth__Model_User_t) => (undefined | string) = BrowserApiJS.AuthClient.avatarUrl as any;

export const AuthClient_classifyRegisterPrivyConflict: (_1:LightconeSdk_SdkError_t) => (undefined | LightconeSdk_Auth__Model_RegisterPrivyConflict_t) = BrowserApiJS.AuthClient.classifyRegisterPrivyConflict as any;

export const OrderbookClient: {
  getWithAggregation: (_1:LightconeSdk_Client_t, _2:string, _3:LightconeSdk_Orderbook__Model_Aggregation_t, depth:(undefined | number), cookieHeader:(undefined | string)) => Promise<LightconeSdk_Orderbook__Raw_DepthResponse_t>;
  clearDecimalsCache: (_1:LightconeSdk_Client_t) => void;
  get: (_1:LightconeSdk_Client_t, _2:string, depth:(undefined | number)) => Promise<LightconeSdk_Orderbook__Raw_DepthResponse_t>;
  invalidateDecimals: (_1:LightconeSdk_Client_t, _2:string) => void;
  decimals: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Scaling_OrderbookRules_t>
} = BrowserApiJS.OrderbookClient as any;

export const OrderClient: {
  submitLimit: (_1:LightconeSdk_Client_t, market:string, baseMint:string, quoteMint:string, side:number, price:string, size:string, orderbookId:string, timeInForce:(undefined | LightconeSdk_Shared_TimeInForce_t)) => Promise<LightconeSdk_Order__Raw_SubmitResponse_t>;
  forUser: (_1:LightconeSdk_Client_t, limit:(undefined | number), cursor:(undefined | string)) => Promise<LightconeSdk_Order__Raw_UserOrdersResponse_t>;
  cancel: (_1:LightconeSdk_Client_t, orderHash:string) => Promise<LightconeSdk_Order__Raw_CancelSuccess_t>;
  fillsByWallet: (_1:LightconeSdk_Client_t, walletAddress:string, marketPubkey:(undefined | string), limit:(undefined | number), cursor:(undefined | string)) => Promise<LightconeSdk_Order__Raw_UserFillsResponse_t>;
  fills: (_1:LightconeSdk_Client_t, marketPubkey:(undefined | string), limit:(undefined | number), cursor:(undefined | string)) => Promise<LightconeSdk_Order__Raw_UserFillsResponse_t>;
  cancelAll: (_1:LightconeSdk_Client_t, orderbookId:string) => Promise<LightconeSdk_Order__Raw_CancelAllSuccess_t>
} = BrowserApiJS.OrderClient as any;

export const WsClient: {
  unsubscribe: (_1:WsClient_wsConnection, _2:LightconeSdk_Subscriptions_UnsubscribeParams_t) => void;
  isConnected: (_1:WsClient_wsConnection) => boolean;
  connect: (_1:LightconeSdk_Client_t, onMessage:((_1:LightconeSdk_Messages_t) => void), onConnected:(undefined | ((() => void))), onError:(undefined | (((_1:LightconeSdk_SdkError_t) => void)))) => WsClient_wsConnection;
  stop: (_1:LightconeSdk_Ws_t) => void;
  subscribe: (_1:WsClient_wsConnection, _2:LightconeSdk_Subscriptions_SubscribeParams_t) => void;
  isStopped: (_1:LightconeSdk_Ws_t) => boolean;
  readyState: (_1:WsClient_wsConnection) => LightconeSdk_Ws_ReadyState_t;
  restartConnection: (_1:LightconeSdk_Ws_t) => void;
  disconnect: (_1:WsClient_wsConnection) => void;
  clearAuthedSubscriptions: (_1:WsClient_wsConnection) => void
} = BrowserApiJS.WsClient as any;

export const PriceHistoryClient: {
  lineData: (_1:LightconeSdk_Client_t, orderbookId:string, resolution:LightconeSdk_Shared_Resolution_t, fromMs:(undefined | number), toMs:(undefined | number), cursor:(undefined | number), limit:(undefined | number)) => Promise<LightconeSdk_PriceHistory__Model_LineData_t[]>;
  get: (_1:LightconeSdk_Client_t, orderbookId:string, resolution:LightconeSdk_Shared_Resolution_t, fromMs:(undefined | number), toMs:(undefined | number)) => Promise<LightconeSdk_PriceHistory__Raw_OrderbookResponse_t>;
  depositAssetSnapshot: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_PriceHistory__Raw_DepositPricesSnapshotResponse_t>
} = BrowserApiJS.PriceHistoryClient as any;

export const FaucetClient: { claim: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Faucet__Raw_Response_t> } = BrowserApiJS.FaucetClient as any;

export const NotificationClient: { dismiss: (_1:LightconeSdk_Client_t, notificationId:string) => Promise<void>; list: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Notification__Model_t[]> } = BrowserApiJS.NotificationClient as any;

export const SolClient: {
  planWrap: (_1:LightconeSdk_Client_t, _2:bigint, _3:LightconeSdk_Position__WalletState_t) => Promise<LightconeSdk_Position__Sol_Plan_t>;
  submit: (_1:LightconeSdk_Client_t, _2:LightconeSdk_Position__Sol_Plan_t) => Promise<LightconeSdk_Transactions_Confirmed_t>;
  planRedeem: (_1:LightconeSdk_Client_t, _2:string, _3:number, _4:number, _5:bigint, _6:LightconeSdk_Position__WalletState_t) => Promise<LightconeSdk_Position__Sol_Plan_t>;
  availability: (_1:LightconeSdk_Position__WalletState_solBalanceBreakdown, _2:LightconeSdk_Position__Sol_Costs_t, unwrapAll:(undefined | boolean)) =>
    {
    TAG: "Ok";
    _0: LightconeSdk_Position__Sol_Availability_t
  }
  | {
    TAG: "Error";
    _0: LightconeSdk_SdkError_t
  };
  planMerge: (_1:LightconeSdk_Client_t, _2:string, _3:number, _4:bigint, _5:LightconeSdk_Position__WalletState_t) => Promise<LightconeSdk_Position__Sol_Plan_t>;
  canonicalAccount: (_1:LightconeSdk_Client_t, _2:string) => Promise<(undefined | LightconeSdk_Position__Sol_CanonicalAccount_t)>;
  planUnwrapAll: (_1:LightconeSdk_Client_t, _2:LightconeSdk_Position__WalletState_t) => Promise<LightconeSdk_Position__Sol_Plan_t>;
  planSplit: (_1:LightconeSdk_Client_t, _2:string, _3:number, _4:bigint, _5:LightconeSdk_Position__WalletState_t) => Promise<LightconeSdk_Position__Sol_Plan_t>;
  planNativeWithdrawal: (_1:LightconeSdk_Client_t, _2:string, _3:bigint, _4:LightconeSdk_Position__WalletState_t) => Promise<LightconeSdk_Position__Sol_Plan_t>
} = BrowserApiJS.SolClient as any;

export const AuthClient: {
  displayName: (_1:LightconeSdk_Auth__Model_User_t) => string;
  logout: (_1:LightconeSdk_Client_t) => Promise<void>;
  credentials: (_1:LightconeSdk_Client_t) => (undefined | LightconeSdk_Auth__Model_Credentials_t);
  avatarUrl: (_1:LightconeSdk_Auth__Model_User_t) => (undefined | string);
  disconnectX: (_1:LightconeSdk_Client_t) => Promise<void>;
  classifyRegisterPrivyConflict: (_1:LightconeSdk_SdkError_t) => (undefined | LightconeSdk_Auth__Model_RegisterPrivyConflict_t);
  registerPrivy: (_1:LightconeSdk_Client_t, _2:LightconeSdk_Auth__Model_RegisterPrivyRequest_t) => Promise<LightconeSdk_Auth__Model_Session_t>;
  walletDisplayName: (_1:LightconeSdk_Auth__Model_User_t, _2:LightconeSdk_Auth__Model_Method_t) => string;
  clearAuth: (_1:LightconeSdk_Client_t) => void;
  getNonce: (_1:LightconeSdk_Client_t) => Promise<string>;
  tradingWallet: (_1:LightconeSdk_Auth__Model_User_t, _2:LightconeSdk_Auth__Model_Method_t) => string;
  isAuthenticated: (_1:LightconeSdk_Client_t) => boolean;
  updateMaxSlippagePreference: (_1:LightconeSdk_Client_t, _2:string) => Promise<string>;
  setCredentialRestorer: (_1:LightconeSdk_Client_t, _2:(undefined | ((() => Promise<boolean>)))) => void;
  checkSession: (_1:LightconeSdk_Client_t, cookieHeader:(undefined | string)) => Promise<LightconeSdk_Auth__Model_Session_t>;
  connectXUrl: (_1:LightconeSdk_Client_t) => string
} = BrowserApiJS.AuthClient as any;

export const ReferralClient: { status: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Referral__Model_Status_t>; redeem: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Referral__Model_RedeemResult_t> } = BrowserApiJS.ReferralClient as any;

export const LiveUserBalances: {
  extend: (_1:LightconeSdk_Position__State_t, _2:LightconeSdk_Position__State_t) => void;
  insert: (_1:LightconeSdk_Position__State_t, marketPubkey:LightconeSdk_Shared_pubkeyStr, _3:LightconeSdk_Position__State_depositAssetBalanceIndex) => void;
  ofMarketBalances: (_1:LightconeSdk_Order__Raw_UserMarketBalance_t[]) => LightconeSdk_Position__State_t;
  get: (_1:LightconeSdk_Position__State_t, marketPubkey:LightconeSdk_Shared_pubkeyStr) => (undefined | LightconeSdk_Position__State_depositAssetBalanceIndex);
  remove: (_1:LightconeSdk_Position__State_t, marketPubkey:LightconeSdk_Shared_pubkeyStr) => void;
  marketPubkeys: (_1:LightconeSdk_Position__State_t) => LightconeSdk_Shared_pubkeyStr[];
  make: () => LightconeSdk_Position__State_t;
  ofMarketBalance: (_1:LightconeSdk_Order__Raw_UserMarketBalance_t) => (undefined | LightconeSdk_Position__State_t)
} = BrowserApiJS.LiveUserBalances as any;

export const TradeClient: { forOrderbook: (_1:LightconeSdk_Client_t, _2:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<LightconeSdk_Trade__Model_Page_t>; forMarket: (_1:LightconeSdk_Client_t, _2:string, limit:(undefined | number), cursor:(undefined | number)) => Promise<LightconeSdk_Trade__Model_Page_t> } = BrowserApiJS.TradeClient as any;

export const LiveTrades: {
  push: (_1:LightconeSdk_Trade__State_t, _2:LightconeSdk_Trade__Model_t) => void;
  trades: (_1:LightconeSdk_Trade__State_t) => LightconeSdk_Trade__Model_t[];
  size: (_1:LightconeSdk_Trade__State_t) => number;
  latest: (_1:LightconeSdk_Trade__State_t) => (undefined | LightconeSdk_Trade__Model_t);
  make: (orderbookId:LightconeSdk_Shared_orderBookId, maxSize:number) => LightconeSdk_Trade__State_t;
  clear: (_1:LightconeSdk_Trade__State_t) => void;
  replace: (_1:LightconeSdk_Trade__State_t, _2:LightconeSdk_Trade__Model_t[]) => void;
  isEmpty: (_1:LightconeSdk_Trade__State_t) => boolean
} = BrowserApiJS.LiveTrades as any;

export const MetricsClient: {
  depositTokens: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Metrics__Raw_DepositTokens_t>;
  userByWallet: (_1:LightconeSdk_Client_t, walletAddress:string) => Promise<LightconeSdk_Metrics__Raw_User_t>;
  user: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Metrics__Raw_User_t>;
  categories: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Metrics__Raw_Categories_t>;
  depositTokensVolumeHistory: (_1:LightconeSdk_Client_t, fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<LightconeSdk_Metrics__Raw_DepositTokenVolumeHistory_t>;
  uniqueTradersHistory: (_1:LightconeSdk_Client_t, scope:(undefined | LightconeSdk_Metrics__Raw_UniqueTradersHistoryScope_t), scopeKey:(undefined | string), fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<LightconeSdk_Metrics__Raw_UniqueTradersHistory_t>;
  markets: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Metrics__Raw_Markets_t>;
  orderbookTickers: (_1:LightconeSdk_Client_t, depositAsset:(undefined | string)) => Promise<LightconeSdk_Metrics__Raw_OrderbookTickersResponse_t>;
  orderbook: (_1:LightconeSdk_Client_t, orderbookId:string) => Promise<LightconeSdk_Metrics__Raw_OrderbookVolume_t>;
  platform: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Metrics__Raw_Platform_t>;
  leaderboard: (_1:LightconeSdk_Client_t, limit:(undefined | number)) => Promise<LightconeSdk_Metrics__Raw_Leaderboard_t>;
  openInterestHistory: (_1:LightconeSdk_Client_t, fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<LightconeSdk_Metrics__Raw_OpenInterestHistory_t>;
  market: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Metrics__Raw_MarketDetail_t>;
  history: (_1:LightconeSdk_Client_t, scope:string, scopeKey:string, resolution:(undefined | LightconeSdk_Shared_Resolution_t), fromMs:(undefined | number), toMs:(undefined | number), limit:(undefined | number)) => Promise<LightconeSdk_Metrics__Raw_History_t>
} = BrowserApiJS.MetricsClient as any;

export const MarketClient: {
  globalDepositAssets: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Market__Model_GlobalDepositAssetsResult_t>;
  favoriteMarkets: (_1:LightconeSdk_Client_t, limit:(undefined | number), cursor:(undefined | bigint), cookieHeader:(undefined | string)) => Promise<LightconeSdk_Market__Model_FavoriteMarkets_t>;
  depositSymbol: (mint:string, symbol:string) => string;
  get: (_1:LightconeSdk_Client_t, cursor:(undefined | number), limit:(undefined | number)) => Promise<LightconeSdk_Market__Model_MarketsResult_t>;
  search: (_1:LightconeSdk_Client_t, _2:string, limit:(undefined | number)) => Promise<LightconeSdk_Market__Raw_MarketSearchResult_t[]>;
  featured: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Market__Raw_MarketSearchResult_t[]>;
  getByPubkey: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Market__Model_t>;
  removeFavoriteMarket: (_1:LightconeSdk_Client_t, marketPubkey:string, cookieHeader:(undefined | string)) => Promise<LightconeSdk_Market__Model_FavoriteMarketUpdate_t>;
  addFavoriteMarket: (_1:LightconeSdk_Client_t, marketPubkey:string, cookieHeader:(undefined | string)) => Promise<LightconeSdk_Market__Model_FavoriteMarketUpdate_t>;
  getBySlug: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Market__Model_t>;
  depositMints: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Market__Raw_DepositMintsResponse_t>
} = BrowserApiJS.MarketClient as any;

export const RpcClient: {
  globalDepositTokenPda: (_1:LightconeSdk_Client_t, _2:string) => Promise<string>;
  nonce: (_1:LightconeSdk_Client_t, _2:string) => Promise<number>;
  marketPda: (_1:LightconeSdk_Client_t, _2:bigint) => Promise<string>;
  position: (_1:LightconeSdk_Client_t, _2:string, _3:string) => Promise<(undefined | LightconeSdk_Accounts_Position_t)>;
  exchangePda: (_1:LightconeSdk_Client_t) => Promise<string>;
  positionPda: (_1:LightconeSdk_Client_t, _2:string, _3:string) => Promise<string>;
  exchange: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Accounts_Exchange_t>;
  orderbook: (_1:LightconeSdk_Client_t, _2:string, _3:string) => Promise<LightconeSdk_Accounts_Orderbook_t>;
  latestBlockhash: (_1:LightconeSdk_Client_t) => Promise<string>;
  activeRpc: (_1:LightconeSdk_Client_t) => LightconeSdk_RpcFailover_Active_t;
  market: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Accounts_Market_t>
} = BrowserApiJS.RpcClient as any;

export const LiveOrderbook: {
  beginGeneration: (_1:LightconeSdk_Orderbook__State_t) => void;
  seq: (_1:LightconeSdk_Orderbook__State_t) => bigint;
  spread: (_1:LightconeSdk_Orderbook__State_t) => (undefined | string);
  asksRaw: (_1:LightconeSdk_Orderbook__State_t) => LightconeSdk_Orderbook__Raw_WsLevel_t[];
  bestAsk: (_1:LightconeSdk_Orderbook__State_t) => (undefined | string);
  orderbookId: (_1:LightconeSdk_Orderbook__State_t) => LightconeSdk_Shared_orderBookId;
  withAggregation: (_1:LightconeSdk_Shared_orderBookId, _2:LightconeSdk_Orderbook__Model_Aggregation_t) => LightconeSdk_Orderbook__State_t;
  midPrice: (_1:LightconeSdk_Orderbook__State_t) => (undefined | string);
  bidsRaw: (_1:LightconeSdk_Orderbook__State_t) => LightconeSdk_Orderbook__Raw_WsLevel_t[];
  asks: (_1:LightconeSdk_Orderbook__State_t) => Array<[string, string]>;
  apply: (_1:LightconeSdk_Orderbook__State_t, _2:LightconeSdk_Orderbook__Raw_Book_t) => LightconeSdk_Orderbook__State_applyResult;
  make: (_1:LightconeSdk_Shared_orderBookId) => LightconeSdk_Orderbook__State_t;
  bestBid: (_1:LightconeSdk_Orderbook__State_t) => (undefined | string);
  clear: (_1:LightconeSdk_Orderbook__State_t) => void;
  bids: (_1:LightconeSdk_Orderbook__State_t) => Array<[string, string]>;
  isEmpty: (_1:LightconeSdk_Orderbook__State_t) => boolean
} = BrowserApiJS.LiveOrderbook as any;

export const LiveDepositPrice: {
  applyPriceTick: (_1:LightconeSdk_PriceHistory__DepositState_t, depositAsset:LightconeSdk_Shared_pubkeyStr, price:string, eventTime:number) => void;
  applyAssetSnapshot: (_1:LightconeSdk_PriceHistory__DepositState_t, depositAsset:LightconeSdk_Shared_pubkeyStr, price:string) => void;
  applySnapshot: (_1:LightconeSdk_PriceHistory__DepositState_t, depositAsset:LightconeSdk_Shared_pubkeyStr, resolution:LightconeSdk_Shared_Resolution_t, candles:LightconeSdk_PriceHistory__Raw_DepositCandle_t[]) => void;
  applyCandle: (_1:LightconeSdk_PriceHistory__DepositState_t, depositAsset:LightconeSdk_Shared_pubkeyStr, resolution:LightconeSdk_Shared_Resolution_t, candle:LightconeSdk_PriceHistory__Raw_DepositCandle_t) => void;
  getCandles: (_1:LightconeSdk_PriceHistory__DepositState_t, depositAsset:LightconeSdk_Shared_pubkeyStr, resolution:LightconeSdk_Shared_Resolution_t) => (undefined | LightconeSdk_PriceHistory__Raw_DepositCandle_t[]);
  make: () => LightconeSdk_PriceHistory__DepositState_t;
  getLatestPrice: (_1:LightconeSdk_PriceHistory__DepositState_t, depositAsset:LightconeSdk_Shared_pubkeyStr) => (undefined | LightconeSdk_PriceHistory__DepositState_latestPrice);
  clear: (_1:LightconeSdk_PriceHistory__DepositState_t) => void
} = BrowserApiJS.LiveDepositPrice as any;

export const LiveWalletBalances: {
  combinedSolBalance: (_1:LightconeSdk_Position__WalletState_t) =>
    {
    TAG: "Ok";
    _0: string
  }
  | {
    TAG: "Error";
    _0: LightconeSdk_SdkError_t
  };
  applyEvent: (_1:LightconeSdk_Position__WalletState_t, _2:LightconeSdk_Position__Raw_WalletDepositBalancesEvent_t, minimumSnapshotSlot:(undefined | bigint)) => LightconeSdk_Position__WalletState_applyResult;
  solBalanceBreakdown: (_1:LightconeSdk_Position__WalletState_t) =>
    {
    TAG: "Ok";
    _0: LightconeSdk_Position__WalletState_solBalanceBreakdown
  }
  | {
    TAG: "Error";
    _0: LightconeSdk_SdkError_t
  };
  contextSlot: (_1:LightconeSdk_Position__WalletState_t) => (undefined | bigint);
  walletAddress: (_1:LightconeSdk_Position__WalletState_t) => (undefined | string);
  nativeSolBalance: (_1:LightconeSdk_Position__WalletState_t) => (undefined | string);
  balances: (_1:LightconeSdk_Position__WalletState_t) => {[id: string]: LightconeSdk_Position__Raw_DepositTokenBalance_t};
  make: () => LightconeSdk_Position__WalletState_t;
  clear: (_1:LightconeSdk_Position__WalletState_t) => void;
  applyRestSnapshot: (_1:LightconeSdk_Position__WalletState_t, walletAddress:string, _3:LightconeSdk_Position__Raw_DepositTokenBalancesSnapshot_t, minimumSnapshotSlot:(undefined | bigint)) => LightconeSdk_Position__WalletState_applyResult
} = BrowserApiJS.LiveWalletBalances as any;

export const TransactionClient: {
  toWireBytes: (_1:LightconeSdk_V1Transaction_t) => Uint8Array;
  requiredSigners: (_1:LightconeSdk_V1Transaction_t) => LightconeSdk_SolanaKit_address[];
  estimateFee: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_t) => Promise<bigint>;
  submitSigned: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_t) => Promise<string>;
  context: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_V1Transaction_Context_t>;
  signAndSubmitConfirmed: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_t) => Promise<LightconeSdk_Transactions_Confirmed_t>;
  confirm: (_1:LightconeSdk_Client_t, _2:string, lastValidBlockHeight:(undefined | bigint)) => Promise<LightconeSdk_Transactions_Confirmed_t>;
  build: (_1:LightconeSdk_Client_t, payer:LightconeSdk_SolanaKit_address, instructions:LightconeSdk_SolanaKit_instruction[]) => Promise<LightconeSdk_V1Transaction_t>;
  setResources: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_Resources_t) => void;
  importBytes: (_1:Uint8Array, _2:LightconeSdk_V1Transaction_Context_t) => Promise<LightconeSdk_V1Transaction_t>;
  signAndSubmit: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_t) => Promise<string>;
  simulate: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_t) => Promise<LightconeSdk_Transactions_Simulation_t>;
  messageBytes: (_1:LightconeSdk_V1Transaction_t) => Uint8Array;
  sign: (_1:LightconeSdk_Client_t, _2:LightconeSdk_V1Transaction_t) => Promise<LightconeSdk_V1Transaction_t>
} = BrowserApiJS.TransactionClient as any;

export const LivePriceHistory: {
  get: (_1:LightconeSdk_PriceHistory__State_t, orderbookId:LightconeSdk_Shared_orderBookId, resolution:LightconeSdk_Shared_Resolution_t) => (undefined | LightconeSdk_PriceHistory__Model_LineData_t[]);
  applySnapshot: (_1:LightconeSdk_PriceHistory__State_t, orderbookId:LightconeSdk_Shared_orderBookId, resolution:LightconeSdk_Shared_Resolution_t, candles:LightconeSdk_PriceHistory__Raw_OrderbookCandle_t[]) => void;
  applyUpdate: (_1:LightconeSdk_PriceHistory__State_t, orderbookId:LightconeSdk_Shared_orderBookId, resolution:LightconeSdk_Shared_Resolution_t, candle:LightconeSdk_PriceHistory__Raw_OrderbookCandle_t) => void;
  make: () => LightconeSdk_PriceHistory__State_t;
  clear: (_1:LightconeSdk_PriceHistory__State_t) => void
} = BrowserApiJS.LivePriceHistory as any;

export const PositionClient: {
  redeemWinnings: (_1:LightconeSdk_Client_t, market:string, mint:string, amount:bigint, outcomeIndex:number) => Promise<string>;
  mine: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Position__Raw_PositionsResponse_t>;
  forUser: (_1:LightconeSdk_Client_t, _2:string) => Promise<LightconeSdk_Position__Raw_PositionsResponse_t>;
  initPositionTokens: (_1:LightconeSdk_Client_t, payer:string, user:string, market:string, depositMints:string[]) => Promise<string>;
  deposit: (_1:LightconeSdk_Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string>;
  depositTokenBalances: (_1:LightconeSdk_Client_t) => Promise<LightconeSdk_Position__Raw_DepositTokenBalancesSnapshot_t>;
  depositToGlobal: (_1:LightconeSdk_Client_t, mint:string, amount:bigint) => Promise<string>;
  closePositionTokenAccounts: (_1:LightconeSdk_Client_t, market:string, position:string, depositMints:string[], numOutcomes:number) => Promise<string>;
  merge: (_1:LightconeSdk_Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string>;
  globalToMarketDeposit: (_1:LightconeSdk_Client_t, market:string, mint:string, amount:bigint, numOutcomes:number) => Promise<string>;
  withdrawFromGlobal: (_1:LightconeSdk_Client_t, mint:string, amount:bigint) => Promise<string>;
  withdrawFromPosition: (_1:LightconeSdk_Client_t, market:string, mint:string, amount:bigint, outcomeIndex:number) => Promise<string>;
  forMarket: (_1:LightconeSdk_Client_t, userPubkey:string, marketPubkey:string) => Promise<LightconeSdk_Position__Raw_MarketPositionsResponse_t>
} = BrowserApiJS.PositionClient as any;

export const LiveOpenLimitOrders: {
  upsert: (_1:LightconeSdk_Order__State_Limits_t, _2:LightconeSdk_Order__Raw_Update_t) => void;
  insert: (_1:LightconeSdk_Order__State_Limits_t, _2:LightconeSdk_Order__Model_Limit_t) => void;
  get: (_1:LightconeSdk_Order__State_Limits_t, marketPubkey:LightconeSdk_Shared_pubkeyStr, orderbookId:LightconeSdk_Shared_orderBookId) => (undefined | LightconeSdk_Order__Model_Limit_t[]);
  remove: (_1:LightconeSdk_Order__State_Limits_t, orderHash:string) => void;
  limitOrderOfUpdate: (_1:LightconeSdk_Order__Raw_Update_t) => LightconeSdk_Order__Model_Limit_t;
  ofSnapshotOrders: (_1:LightconeSdk_Order__Raw_SnapshotOrder_t[]) => LightconeSdk_Order__State_Limits_t;
  make: () => LightconeSdk_Order__State_Limits_t;
  clear: (_1:LightconeSdk_Order__State_Limits_t) => void;
  getByMarket: (_1:LightconeSdk_Order__State_Limits_t, marketPubkey:LightconeSdk_Shared_pubkeyStr) => (undefined | {[id: string]: LightconeSdk_Order__Model_Limit_t[]});
  isEmpty: (_1:LightconeSdk_Order__State_Limits_t) => boolean
} = BrowserApiJS.LiveOpenLimitOrders as any;
