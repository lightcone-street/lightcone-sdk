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
  /** Monotonic sequence number per orderbook for ordering guarantees. 0 for REST trades. */
  sequence: number;
}

export interface TradesPage {
  trades: Trade[];
  nextCursor?: number;
  hasMore: boolean;
}
