import Decimal from "decimal.js";
import type { OrderBookId } from "../../shared";
import type { OrderBook } from "./wire";

/**
 * The `book_update` stream is snapshot-only: every data frame carries the
 * full top-20 levels per side and replaces the previous book wholesale
 * (last-write-wins). Consumers holding multiple aggregation views of one
 * orderbook on the same connection key their `OrderbookState` instances by
 * `(orderbook_id, aggregation)` using
 * `aggregationFromFrame(book.n_sig_figs, book.mantissa)`.
 */
export type OrderbookApplyResult =
  | { kind: "applied" }
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
  /**
   * Projection version of the last applied frame. Strictly increasing but
   * non-contiguous server-side (conflation skips versions), and the initial
   * snapshot after every (re)subscribe is `seq: 0` — informational only,
   * never used to gate frames.
   */
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
   * Apply a WS orderbook frame (snapshot-only stream, last-write-wins).
   *
   * `resync` frames take precedence and leave the book untouched — the
   * caller must re-subscribe with the same parameters. Every other data
   * frame is a full snapshot by contract and replaces the book wholesale
   * (the `is_snapshot` flag is not consulted), including the `seq: 0`
   * initial snapshot delivered after every (re)subscribe: gating on `seq`
   * would freeze the book after a resync or aggregation change, so `seq` is
   * stored as informational only.
   */
  apply(book: OrderBook): OrderbookApplyResult {
    if (book.resync) {
      return {
        kind: "refresh_required",
        reason: { kind: "server_resync" },
      };
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
    this.seq = book.seq ?? 0;
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
  }
}
