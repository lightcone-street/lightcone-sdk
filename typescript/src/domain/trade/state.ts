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
    if (this.buffer.length >= this.maxSize) {
      this.buffer.pop();
    }
    if (trade.sequence === 0) {
      this.buffer.unshift(trade);
      return;
    }
    const position = this.buffer.findIndex(
      (existing) => existing.sequence < trade.sequence,
    );
    if (position === -1) {
      this.buffer.push(trade);
    } else {
      this.buffer.splice(position, 0, trade);
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
