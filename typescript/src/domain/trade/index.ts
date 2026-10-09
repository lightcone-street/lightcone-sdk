import type { OrderBookId, Side } from "../../shared";

export * from "./client";
export * from "./wire";
export * from "./state";
export { tradeFromResponse, tradeFromWs } from "./convert";

/** A trade execution record. */
export interface Trade {
  orderbookId: OrderBookId;
  /**
   * `"<execution_id>:<leg_index>:<projection_generation>"`, identical for REST
   * and WS trades, so it deduplicates merged history.
   */
  tradeId: string;
  /** Numeric REST row id (i64) used for cursor pagination. Undefined on WS trades. */
  cursorId?: bigint;
  timestamp: Date;
  /**
   * Price in quote units per base unit. REST reports the maker price truncated
   * to the book's price decimals; WS trades derive the exact
   * `quote_amount / base_amount`.
   */
  price: string;
  /** Size in base-token units. */
  size: string;
  /** Taker side. */
  side: Side;
}

/** A page of trades with cursor-based pagination metadata. */
export interface TradesPage {
  trades: Trade[];
  nextCursor?: bigint;
  hasMore: boolean;
}
