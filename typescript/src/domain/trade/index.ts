import type { OrderBookId, Side } from "../../shared";

export * from "./client";
export * from "./wire";
export * from "./state";
export { tradeFromResponse, tradeFromWs } from "./convert";

export interface Trade {
  orderbookId: OrderBookId;
  tradeId: string;
  timestamp: Date;
  price: string;
  size: string;
  side: Side;
}

export interface TradesPage {
  trades: Trade[];
  nextCursor?: number;
  hasMore: boolean;
}
