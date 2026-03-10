export {
  asOrderBookId,
  asPubkeyStr,
  DepositSource,
  deriveOrderbookId,
  OrderUpdateType,
  Resolution,
  Side,
  TimeInForce,
  TriggerResultStatus,
  TriggerStatus,
  TriggerType,
  TriggerUpdateType,
  type OrderBookId,
  type PubkeyStr,
  type SubmitOrderRequest,
} from "./shared";

export { SdkError } from "./error";
export { DEFAULT_API_URL, DEFAULT_WS_URL } from "./network";

export {
  LightconeClient,
  LightconeClientBuilder,
} from "./client";

export { Auth } from "./auth";
export { Admin } from "./domain/admin";
export { Markets } from "./domain/market";
export { Orderbooks } from "./domain/orderbook";
export { Orders } from "./domain/order";
export { Positions } from "./domain/position";
export { Trades } from "./domain/trade";
export { PriceHistoryClient } from "./domain/price_history";
export { Notifications } from "./domain/notification";
export { Referrals } from "./domain/referral";

export type {
  Market,
  Status,
  Outcome,
  ConditionalToken,
  DepositAsset,
  Token,
  TokenMetadata,
  ValidatedTokens,
} from "./domain/market";

export type {
  OrderBook,
  OrderBookPair,
  OutcomeImpact,
  TickerData,
  WsBookLevel,
} from "./domain/orderbook";
export { OrderBookValidationError } from "./domain/orderbook";
export { OrderbookSnapshot } from "./domain/orderbook/state";

export type {
  CancelAllBody,
  CancelAllSuccess,
  CancelBody,
  CancelSuccess,
  CancelTriggerBody,
  CancelTriggerSuccess,
  ConditionalBalance,
  FillInfo,
  Order,
  OrderEvent,
  OrderStatus,
  OrderType,
  SubmitOrderResponse,
  TriggerOrder,
  TriggerOrderResponse,
  TriggerOrderUpdate,
  UserSnapshotBalance,
  UserSnapshotOrder,
  UserOrdersResponse,
} from "./domain/order";
export type { UserOpenOrders, UserTriggerOrders } from "./domain/order/state";

export type {
  Portfolio,
  Position,
  PositionOutcome,
  WalletHolding,
  TokenBalance,
  TokenBalanceComputedBase,
  TokenBalanceTokenType,
  DepositAssetMetadata,
  DepositTokenBalance,
} from "./domain/position";

export type { Trade, WsTrade } from "./domain/trade";
export { TradeHistory } from "./domain/trade/state";
export type {
  LineData,
  PriceCandle,
  PriceHistory,
  PriceHistorySnapshot,
  PriceHistoryUpdate,
} from "./domain/price_history";
export { PriceHistoryState } from "./domain/price_history";

export type {
  AuthCredentials,
  ChainType,
  EmbeddedWallet,
  LinkedAccount,
  LinkedAccountType,
  User,
} from "./auth";

export {
  LimitOrderEnvelope,
  TriggerOrderEnvelope,
  type OrderEnvelope,
  type OrderPayload,
} from "./program";

export type {
  ExportWalletRequest,
  ExportWalletResponse,
  PrivyOrderEnvelope,
  SignAndSendOrderRequest,
  SignAndSendTxRequest,
  SignAndSendTxResponse,
} from "./privy";

export type {
  Notification,
  NotificationKind,
  MarketResolvedData,
  OrderFilledData,
  MarketData,
} from "./domain/notification";

export type {
  RedeemResult,
  ReferralCodeInfo,
  ReferralStatus,
} from "./domain/referral";

export {
  RetryPolicy,
  type RetryConfig,
} from "./http";

export type {
  IWsClient,
  Kind,
  MessageIn,
  MessageOut,
  SubscribeParams,
  UnsubscribeParams,
  WsConfig,
  WsEvent,
} from "./ws";

export type AuthClient = import("./auth").Auth;
export type AdminClient = import("./domain/admin").Admin;
export type MarketsClient = import("./domain/market").Markets;
export type MarketsResult = import("./domain/market").MarketsResult;
export type OrderbooksClient = import("./domain/orderbook").Orderbooks;
export type OrdersClient = import("./domain/order").Orders;
export type PositionsClient = import("./domain/position").Positions;
export type TradesClient = import("./domain/trade").Trades;
export type PriceHistorySubClient = import("./domain/price_history").PriceHistoryClient;
export type NotificationsClient = import("./domain/notification").Notifications;
export type ReferralsClient = import("./domain/referral").Referrals;
