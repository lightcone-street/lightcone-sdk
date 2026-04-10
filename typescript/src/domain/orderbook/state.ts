import Decimal from "decimal.js";
import type { OrderBookId } from "../../shared";
import type { OrderBook } from "./wire";

export class OrderbookSnapshot {
  readonly orderbookId: OrderBookId;
  seq: number;
  private readonly bidsMap: Map<string, string>;
  private readonly asksMap: Map<string, string>;
  private cachedBestBid: string | undefined | null;
  private cachedBestAsk: string | undefined | null;

  constructor(orderbookId: OrderBookId) {
    this.orderbookId = orderbookId;
    this.seq = 0;
    this.bidsMap = new Map();
    this.asksMap = new Map();
    this.cachedBestBid = null;
    this.cachedBestAsk = null;
  }

  /**
   * Apply a WS orderbook message (snapshot replaces, delta merges).
   *
   * Snapshots are always applied. Deltas with a `seq` at or below the
   * current value are silently dropped to prevent stale updates.
   */
  apply(book: OrderBook): void {
    if (book.is_snapshot) {
      this.bidsMap.clear();
      this.asksMap.clear();
    } else if (book.seq && book.seq > 0 && book.seq <= this.seq) {
      return;
    }

    this.seq = book.seq ?? this.seq;

    for (const level of book.bids) {
      if (new Decimal(level.size).isZero()) {
        this.bidsMap.delete(level.price);
      } else {
        this.bidsMap.set(level.price, level.size);
      }
    }

    for (const level of book.asks) {
      if (new Decimal(level.size).isZero()) {
        this.asksMap.delete(level.price);
      } else {
        this.asksMap.set(level.price, level.size);
      }
    }

    this.cachedBestBid = null;
    this.cachedBestAsk = null;
  }

  bids(): ReadonlyMap<string, string> {
    return this.bidsMap;
  }

  asks(): ReadonlyMap<string, string> {
    return this.asksMap;
  }

  bestBid(): string | undefined {
    if (this.cachedBestBid !== null) {
      return this.cachedBestBid;
    }
    if (this.bidsMap.size === 0) {
      this.cachedBestBid = undefined;
      return undefined;
    }
    const result = Array.from(this.bidsMap.keys())
      .sort((a, b) => new Decimal(a).cmp(new Decimal(b)))
      .at(-1);
    this.cachedBestBid = result;
    return result;
  }

  bestAsk(): string | undefined {
    if (this.cachedBestAsk !== null) {
      return this.cachedBestAsk;
    }
    if (this.asksMap.size === 0) {
      this.cachedBestAsk = undefined;
      return undefined;
    }
    const result = Array.from(this.asksMap.keys())
      .sort((a, b) => new Decimal(a).cmp(new Decimal(b)))[0];
    this.cachedBestAsk = result;
    return result;
  }

  midPrice(): string | undefined {
    const bid = this.bestBid();
    const ask = this.bestAsk();
    if (!bid || !ask) {
      return undefined;
    }

    return new Decimal(bid).plus(ask).div(2).toString();
  }

  spread(): string | undefined {
    const bid = this.bestBid();
    const ask = this.bestAsk();
    if (!bid || !ask) {
      return undefined;
    }

    return new Decimal(ask).minus(bid).toString();
  }

  isEmpty(): boolean {
    return this.bidsMap.size === 0 && this.asksMap.size === 0;
  }

  clear(): void {
    this.bidsMap.clear();
    this.asksMap.clear();
    this.seq = 0;
    this.cachedBestBid = null;
    this.cachedBestAsk = null;
  }
}
