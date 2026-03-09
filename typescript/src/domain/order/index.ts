import type { OrderBookId, PubkeyStr, Side, TimeInForce, TriggerType } from "../../shared";

export * from "./client";
export * from "./wire";
export * from "./state";
export { limitSnapshotToOrder, splitSnapshotOrders, triggerSnapshotToOrder, orderFromUpdate } from "./convert";

export enum OrderType {
  Limit = "Limit",
  Market = "Market",
  Deposit = "Deposit",
  Withdraw = "Withdraw",
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
