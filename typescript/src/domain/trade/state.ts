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

  push(trade: Trade): void {
    this.buffer.unshift(trade);
    if (this.buffer.length > this.maxSize) {
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
