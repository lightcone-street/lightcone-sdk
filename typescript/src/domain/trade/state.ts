import type { OrderBookId } from "../../shared";
import type { Trade } from "./index";

/**
 * Rolling trade history buffer for an orderbook.
 *
 * The app owns instances of this type. The SDK provides update methods.
 */
export class TradeHistory {
  readonly orderbookId: OrderBookId;
  private readonly maxSize: number;
  private readonly buffer: Trade[];

  constructor(orderbookId: OrderBookId, maxSize: number) {
    this.orderbookId = orderbookId;
    this.maxSize = maxSize;
    this.buffer = [];
  }

  /**
   * Push a trade, keeping the buffer newest-first by execution time.
   *
   * Trades are deduplicated by `tradeId`, which REST and WS share, so live
   * trades can be merged into REST history (a duplicate only fills in a
   * missing REST `cursorId`). Equal timestamps keep arrival order, newest
   * first. When full, a trade older than everything retained is dropped.
   * The history stores a copy and never changes a pushed `Trade`.
   */
  push(trade: Trade): void {
    // Treat a zero-capacity history as disabled.
    if (this.maxSize === 0) {
      return;
    }
    const index = this.buffer.findIndex((retained) => retained.tradeId === trade.tradeId);
    if (index !== -1) {
      const existing = this.buffer[index];
      if (existing.cursorId === undefined && trade.cursorId !== undefined) {
        this.buffer[index] = { ...existing, cursorId: trade.cursorId };
      }
      return;
    }
    let position = this.buffer.findIndex(
      (retained) => retained.timestamp.getTime() <= trade.timestamp.getTime(),
    );
    if (position === -1) position = this.buffer.length;
    if (this.buffer.length >= this.maxSize && position === this.buffer.length) {
      return;
    }
    this.buffer.splice(position, 0, { ...trade });
    if (this.buffer.length > this.maxSize) {
      this.buffer.pop();
    }
  }

  /** Replace all trades (e.g. from a REST fetch). */
  replace(trades: Trade[]): void {
    this.buffer.length = 0;
    this.buffer.push(...trades.slice(0, this.maxSize));
  }

  trades(): readonly Trade[] {
    return this.buffer;
  }

  latest(): Trade | undefined {
    return this.buffer[0];
  }

  clear(): void {
    this.buffer.length = 0;
  }

  len(): number {
    return this.buffer.length;
  }

  isEmpty(): boolean {
    return this.buffer.length === 0;
  }
}
