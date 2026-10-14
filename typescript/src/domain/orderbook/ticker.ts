import type { OrderBookId } from "../../shared";

export interface TickerData {
  orderbookId: OrderBookId;
  bestBid?: string;
  bestAsk?: string;
  /** Engine-authoritative; may use one-sided-book or last-trade fallback. */
  midPrice?: string;
}
