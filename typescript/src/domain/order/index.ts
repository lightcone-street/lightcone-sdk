import Decimal from "decimal.js";
import { Side, type OrderBookId, type PubkeyStr, type TimeInForce, type TriggerType } from "../../shared";

export * from "./client";
export * from "./wire";
export * from "./state";
export { limitSnapshotToOrder, splitSnapshotOrders, triggerSnapshotToOrder, orderFromUpdate, triggerOrderFromUpdate } from "./convert";

export enum OrderType {
  Limit = "Limit",
  Market = "Market",
  Deposit = "Deposit",
  Withdraw = "Withdraw",
  StopMarket = "StopMarket",
  StopLimit = "StopLimit",
  TakeProfitMarket = "TakeProfitMarket",
  TakeProfitLimit = "TakeProfitLimit",
}

export enum OrderStatus {
  Open = "OPEN",
  Matching = "MATCHING",
  Cancelled = "CANCELLED",
  Filled = "FILLED",
  Pending = "PENDING",
}

export interface Order {
  marketPubkey: PubkeyStr;
  orderbookId: OrderBookId;
  txSignature?: string;
  baseMint: PubkeyStr;
  quoteMint: PubkeyStr;
  orderHash: string;
  side: Side;
  size: string;
  price: string;
  filledSize: string;
  remainingSize: string;
  createdAt: Date;
  status: OrderStatus;
  outcomeIndex: number;
}

export interface TriggerOrder {
  triggerOrderId: string;
  orderHash: string;
  marketPubkey: PubkeyStr;
  orderbookId: OrderBookId;
  triggerPrice: string;
  triggerType: TriggerType;
  side: Side;
  amountIn: string;
  amountOut: string;
  timeInForce: TimeInForce;
  createdAt: Date;
}

export function triggerOrderLimitPrice(order: TriggerOrder): Decimal | undefined {
  const amountIn = new Decimal(order.amountIn);
  const amountOut = new Decimal(order.amountOut);

  if (order.side === Side.Ask && amountIn.greaterThan(0)) {
    return amountOut.div(amountIn);
  }
  if (order.side === Side.Bid && amountOut.greaterThan(0)) {
    return amountIn.div(amountOut);
  }
  return undefined;
}
