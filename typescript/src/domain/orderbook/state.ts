import Decimal from "decimal.js";
import type { OrderBookId } from "../../shared";
import type { OrderBook } from "./wire";

export type OrderbookApplyResult =
  | { kind: "applied" }
  | { kind: "stale" }
  | { kind: "gap_detected"; expected: number; got: number };

export class OrderbookState {
  readonly orderbookId: OrderBookId;
  seq: number;
  private readonly bidsMap: Map<string, string>;
  private readonly asksMap: Map<string, string>;
  private cachedBestBid: string | undefined | null;
  private cachedBestAsk: string | undefined | null;
  private hasSnapshot: boolean;

  constructor(orderbookId: OrderBookId) {
    this.orderbookId = orderbookId;
    this.seq = 0;
    this.bidsMap = new Map();
    this.asksMap = new Map();
    this.cachedBestBid = null;
    this.cachedBestAsk = null;
    this.hasSnapshot = false;
  }

  /**
   * Apply a WS orderbook message (snapshot replaces, delta merges).
   *
   * Snapshots are always applied. Deltas with a `seq` at or below the
   * current value are silently dropped to prevent stale updates. Deltas
   * that skip one or more expected sequence values are rejected so callers
   * can refresh from a fresh snapshot instead of mutating a corrupted book.
   * Server resync messages leave the book unchanged and return `stale`.
   */
  apply(book: OrderBook): OrderbookApplyResult {
    if (book.resync) {
      return { kind: "stale" };
    }

    const seq = book.seq ?? 0;

    if (book.is_snapshot) {
      this.bidsMap.clear();
      this.asksMap.clear();
      this.hasSnapshot = true;
    } else {
      // The backend sends snapshots with seq=0 and starts delta seq at 1.
      // A delta with seq=0 means it has no valid sequence, so drop it.
      if (seq <= 0) {
        return { kind: "stale" };
      }

      if (!this.hasSnapshot) {
        return { kind: "gap_detected", expected: 0, got: seq };
      }

      if (seq <= this.seq) {
        return { kind: "stale" };
      }

      if (seq !== this.seq + 1) {
        return { kind: "gap_detected", expected: this.seq + 1, got: seq };
      }
    }

    this.seq = seq;

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

    return { kind: "applied" };
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
    this.hasSnapshot = false;
  }
}
