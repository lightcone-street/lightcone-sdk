import type { OrderBookId } from "../../shared";

export interface TickerData {
  orderbookId: OrderBookId;
  bestBid?: string;
  bestAsk?: string;
  midPrice?: string;
}
