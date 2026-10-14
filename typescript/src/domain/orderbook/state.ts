import Decimal from "decimal.js";
import type { OrderBookId } from "../../shared";
import type { OrderBook } from "./wire";
import {
  aggregationFromFrame,
  aggregationsEqual,
  normalizeAggregation,
  type BookAggregation,
} from "./aggregation";

/**
 * The `book_update` stream is snapshot-only: every data frame carries the
 * full top-20 levels per side and replaces the previous book wholesale.
 * Equal and older revisions are discarded within one generation.
 * Consumers holding multiple aggregation views of one orderbook on the same
 * connection key their `OrderbookState` instances by
 * `(orderbook_id, aggregation)` using
 * `aggregationFromFrame(book.n_sig_figs, book.mantissa)`.
 */
export type OrderbookApplyResult =
  | { kind: "applied" }
  | { kind: "discarded_stale" }
  | { kind: "subscription_mismatch" }
  | { kind: "refresh_required"; reason: OrderbookRefreshReason };

export type OrderbookRefreshReason =
  /**
   * The backend explicitly requested a resync: unsubscribe and re-subscribe
   * with the same parameters (including aggregation) to receive a fresh
   * snapshot.
   */
  { kind: "server_resync" };

export class OrderbookState {
  readonly orderbookId: OrderBookId;
  readonly aggregation: BookAggregation;
  /**
   * Last accepted engine depth revision. Forward gaps are expected.
   */
  seq: bigint;
  private lastSeq: bigint | undefined;
  bidsTruncated: boolean;
  asksTruncated: boolean;
  private readonly bidsMap: Map<string, string>;
  private readonly asksMap: Map<string, string>;
  private cachedBestBid: string | undefined | null;
  private cachedBestAsk: string | undefined | null;

  constructor(orderbookId: OrderBookId, aggregation: BookAggregation = {}) {
    this.orderbookId = orderbookId;
    this.aggregation = normalizeAggregation(aggregation);
    this.seq = 0n;
    this.lastSeq = undefined;
    this.bidsTruncated = false;
    this.asksTruncated = false;
    this.bidsMap = new Map();
    this.asksMap = new Map();
    this.cachedBestBid = null;
    this.cachedBestAsk = null;
  }

  /**
   * Apply a full snapshot when its revision is newer in this generation.
   *
   * `resync` frames take precedence and leave the book untouched — the
   * caller must re-subscribe with the same parameters. Every other data
  * accepted frame replaces the book wholesale. Revision gaps are normal.
  */
  apply(book: OrderBook): OrderbookApplyResult {
    if (book.seq < 0n) throw new Error("book_update seq must be non-negative");
    if (
      book.orderbook_id !== this.orderbookId ||
      !aggregationsEqual(
        aggregationFromFrame(book.n_sig_figs, book.mantissa),
        this.aggregation
      )
    ) {
      return { kind: "subscription_mismatch" };
    }
    if (book.resync) {
      return {
        kind: "refresh_required",
        reason: { kind: "server_resync" },
      };
    }
    if (this.lastSeq !== undefined && book.seq <= this.lastSeq) {
      return { kind: "discarded_stale" };
    }

    this.bidsMap.clear();
    this.asksMap.clear();
    for (const level of book.bids) {
      if (!new Decimal(level.size).isZero()) {
        this.bidsMap.set(level.price, level.size);
      }
    }
    for (const level of book.asks) {
      if (!new Decimal(level.size).isZero()) {
        this.asksMap.set(level.price, level.size);
      }
    }
    this.seq = book.seq;
    this.lastSeq = book.seq;
    this.bidsTruncated = book.bids_truncated ?? false;
    this.asksTruncated = book.asks_truncated ?? false;
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

  /** Reset the revision gate for reconnect/resubscribe without hiding levels. */
  beginGeneration(): void {
    this.lastSeq = undefined;
  }

  clear(): void {
    this.bidsMap.clear();
    this.asksMap.clear();
    this.seq = 0n;
    this.lastSeq = undefined;
    this.bidsTruncated = false;
    this.asksTruncated = false;
    this.cachedBestBid = null;
    this.cachedBestAsk = null;
  }
}
