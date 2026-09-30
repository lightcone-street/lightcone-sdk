import { Side, type OrderBookId, type PubkeyStr } from "../../shared";

export * from "./client";
export * from "./wire";
export * from "./state";
export { limitSnapshotToOrder, convertSnapshotOrders, orderFromUpdate } from "./convert";

/** Supported resting-order kind. */
export enum OrderType {
  Limit = "limit",
}

export function orderTypeLabel(orderType: OrderType): string {
  switch (orderType) {
    case OrderType.Limit:
      return "Limit";
  }
}

export enum OrderStatus {
  Open = "OPEN",
  Matching = "MATCHING",
  Cancelled = "CANCELLED",
  Filled = "FILLED",
  Pending = "PENDING",
}

export interface LimitOrder {
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
