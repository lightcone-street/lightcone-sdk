import type { OrderBookId } from "../../shared";
import type { Trade } from "./index";

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
   * Insert a trade in descending sequence order.
   *
   * Trades with `sequence > 0` are placed at the correct position so the
   * buffer stays sorted newest-first. Trades with `sequence === 0` (REST)
   * are prepended.
   */
  push(trade: Trade): void {
    // Treat a zero-capacity history as disabled.
    if (this.maxSize === 0) {
      return;
    }
    if (trade.sequence === 0) {
      // REST trades do not carry ordering metadata, so preserve the legacy
      // "latest first" behavior and let normal capacity eviction apply.
      this.buffer.unshift(trade);
      if (this.buffer.length > this.maxSize) {
        this.buffer.pop();
      }
      return;
    }
    // Find the first retained trade that is older than the incoming sequence.
    // Inserting there keeps the buffer sorted newest-first.
    const position = this.buffer.findIndex(
      (existing) => existing.sequence < trade.sequence,
    );
    if (this.buffer.length >= this.maxSize && position === -1) {
      // The buffer is full and the new trade is older than everything we
      // already keep, so dropping it preserves the newest retained window.
      return;
    }
    if (position === -1) {
      this.buffer.push(trade);
    } else {
      this.buffer.splice(position, 0, trade);
    }
    if (this.buffer.length > this.maxSize) {
      // The new trade landed inside the retained window, so evict the oldest
      // trade at the tail to restore capacity.
      this.buffer.pop();
    }
  }

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
