import Decimal from "decimal.js";
import type { OrderBookId } from "../../shared";
import type { OrderBook } from "./wire";

export type OrderbookApplyResult =
  | { kind: "applied" }
  | { kind: "ignored"; reason: OrderbookIgnoreReason }
  | { kind: "refresh_required"; reason: OrderbookRefreshReason };

export type OrderbookIgnoreReason =
  | { kind: "invalid_delta_sequence"; got: number }
  | { kind: "stale_delta"; current: number; got: number }
  | { kind: "already_awaiting_snapshot"; got: number };

export type OrderbookRefreshReason =
  | { kind: "missing_snapshot"; got: number }
  | { kind: "sequence_gap"; expected: number; got: number }
  | { kind: "server_resync"; got: number };

export class OrderbookState {
  readonly orderbookId: OrderBookId;
  seq: number;
  private readonly bidsMap: Map<string, string>;
  private readonly asksMap: Map<string, string>;
  private cachedBestBid: string | undefined | null;
  private cachedBestAsk: string | undefined | null;
  private hasSnapshot: boolean;
  private awaitingSnapshot: boolean;

  constructor(orderbookId: OrderBookId) {
    this.orderbookId = orderbookId;
    this.seq = 0;
    this.bidsMap = new Map();
    this.asksMap = new Map();
    this.cachedBestBid = null;
    this.cachedBestAsk = null;
    this.hasSnapshot = false;
    this.awaitingSnapshot = false;
  }

  /**
   * Apply a WS orderbook message (snapshot replaces, delta merges).
   *
   * Server resync messages take precedence and return `refresh_required`.
   * Otherwise, snapshots are applied and deltas with a `seq` at or below the
   * current value are ignored to prevent stale updates. Deltas that skip one
   * or more expected sequence values are rejected so callers can refresh from
   * a fresh snapshot instead of mutating a corrupted book.
   */
  apply(book: OrderBook): OrderbookApplyResult {
    const seq = book.seq ?? 0;

    if (book.resync) {
      this.awaitingSnapshot = true;
      return {
        kind: "refresh_required",
        reason: { kind: "server_resync", got: seq },
      };
    }

    if (book.is_snapshot) {
      this.bidsMap.clear();
      this.asksMap.clear();
      this.hasSnapshot = true;
      this.awaitingSnapshot = false;
    } else {
      if (this.awaitingSnapshot) {
        return {
          kind: "ignored",
          reason: { kind: "already_awaiting_snapshot", got: seq },
        };
      }

      // The backend sends snapshots with seq=0 and starts delta seq at 1.
      // A delta with seq=0 means it has no valid sequence, so drop it.
      if (seq <= 0) {
        return {
          kind: "ignored",
          reason: { kind: "invalid_delta_sequence", got: seq },
        };
      }

      if (!this.hasSnapshot) {
        this.awaitingSnapshot = true;
        return {
          kind: "refresh_required",
          reason: { kind: "missing_snapshot", got: seq },
        };
      }

      if (seq <= this.seq) {
        return {
          kind: "ignored",
          reason: { kind: "stale_delta", current: this.seq, got: seq },
        };
      }

      const expected = this.seq + 1;
      if (seq !== expected) {
        this.awaitingSnapshot = true;
        return {
          kind: "refresh_required",
          reason: { kind: "sequence_gap", expected, got: seq },
        };
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
    this.awaitingSnapshot = false;
  }
}
