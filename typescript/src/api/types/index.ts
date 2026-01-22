/**
 * Re-export all API types.
 */

// Market types
export type {
  ApiMarketStatus,
  Outcome,
  OrderbookSummary,
  ConditionalToken,
  DepositAsset,
  Market,
  MarketsResponse,
  MarketInfoResponse,
  DepositAssetsResponse,
} from "./market";

// Order types
export type {
  ApiOrderSide,
  OrderStatusValue,
  Fill,
  SubmitOrderRequest,
  OrderResponse,
  CancelOrderRequest,
  CancelResponse,
  CancelAllOrdersRequest,
  CancelAllResponse,
  UserOrder,
  GetUserOrdersRequest,
  UserOrderOutcomeBalance,
  UserBalance,
  UserOrdersResponse,
} from "./order";

// Orderbook types
export type { PriceLevel, OrderbookResponse } from "./orderbook";

// Position types
export type {
  OutcomeBalance,
  Position,
  PositionsResponse,
  MarketPositionsResponse,
} from "./position";

// Price history types
export type {
  PricePoint,
  PriceHistoryParams,
  PriceHistoryResponse,
} from "./price_history";
export { createPriceHistoryParams } from "./price_history";

// Trade types
export type { TradeSide, Trade, TradesParams, TradesResponse } from "./trade";
export { createTradesParams } from "./trade";

// Admin types
export type {
  AdminResponse,
  CreateOrderbookRequest,
  CreateOrderbookResponse,
} from "./admin";
export { createOrderbookRequest } from "./admin";
