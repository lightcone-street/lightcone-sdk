/**
 * Local orderbook state management.
 *
 * Maintains a local copy of the orderbook state, applying deltas from
 * WebSocket updates.
 *
 * Note: Internally uses string keys/values to match the string-based API.
 * For numeric comparisons, parse the strings as needed.
 */

import Decimal from "decimal.js";
import { WebSocketError } from "../error";
import type { BookUpdateData, PriceLevel } from "../types";
import { isZero } from "../../shared/price";

/**
 * Local orderbook state.
 */
export class LocalOrderbook {
  /** Maximum number of price levels to maintain per side */
  private static readonly MAX_LEVELS = 1000;

  /** Orderbook identifier */
  readonly orderbookId: string;
  /** Bid levels (price string -> size string) */
  private bids: Map<string, string> = new Map();
  /** Ask levels (price string -> size string) */
  private asks: Map<string, string> = new Map();
  /** Expected next sequence number */
  private expectedSeq: number = 0;
  /** Whether initial snapshot has been received */
  private _hasSnapshot: boolean = false;
  /** Last update timestamp */
  private _lastTimestamp?: string;

  constructor(orderbookId: string) {
    this.orderbookId = orderbookId;
  }

  /**
   * Apply a snapshot (full orderbook state).
   */
  applySnapshot(update: BookUpdateData): void {
    // Validate sequence number
    if (typeof update.seq !== 'number' || update.seq < 0 || !Number.isFinite(update.seq)) {
      console.warn(`Invalid sequence number in snapshot: ${update.seq}`);
      return;
    }

    // Clear existing state
    this.bids.clear();
    this.asks.clear();

    // Apply all levels (respecting max depth)
    for (const level of update.bids) {
      if (!isZero(level.size)) {
        if (this.bids.size >= LocalOrderbook.MAX_LEVELS && !this.bids.has(level.price)) {
          continue; // Skip if at limit and new level
        }
        this.bids.set(level.price, level.size);
      }
    }

    for (const level of update.asks) {
      if (!isZero(level.size)) {
        if (this.asks.size >= LocalOrderbook.MAX_LEVELS && !this.asks.has(level.price)) {
          continue; // Skip if at limit and new level
        }
        this.asks.set(level.price, level.size);
      }
    }

    this.expectedSeq = update.seq + 1;
    this._hasSnapshot = true;
    this._lastTimestamp = update.timestamp;
  }

  /**
   * Apply a delta update.
   *
   * @throws {WebSocketError} If a sequence gap is detected
   */
  applyDelta(update: BookUpdateData): void {
    // Validate sequence number
    if (typeof update.seq !== 'number' || update.seq < 0 || !Number.isFinite(update.seq)) {
      console.warn(`Invalid sequence number in delta: ${update.seq}`);
      return;
    }

    // Check sequence number
    if (update.seq !== this.expectedSeq) {
      throw WebSocketError.sequenceGap(this.expectedSeq, update.seq);
    }

    // Apply bid updates (respecting max depth)
    for (const level of update.bids) {
      if (isZero(level.size)) {
        this.bids.delete(level.price);
      } else {
        if (this.bids.size >= LocalOrderbook.MAX_LEVELS && !this.bids.has(level.price)) {
          continue; // Skip if at limit and new level
        }
        this.bids.set(level.price, level.size);
      }
    }

    // Apply ask updates (respecting max depth)
    for (const level of update.asks) {
      if (isZero(level.size)) {
        this.asks.delete(level.price);
      } else {
        if (this.asks.size >= LocalOrderbook.MAX_LEVELS && !this.asks.has(level.price)) {
          continue; // Skip if at limit and new level
        }
        this.asks.set(level.price, level.size);
      }
    }

    this.expectedSeq = update.seq + 1;
    this._lastTimestamp = update.timestamp;
  }

  /**
   * Apply an update (snapshot or delta).
   *
   * @throws {WebSocketError} If a sequence gap is detected
   */
  applyUpdate(update: BookUpdateData): void {
    if (update.is_snapshot) {
      this.applySnapshot(update);
    } else {
      this.applyDelta(update);
    }
  }

  /**
   * Get all bid levels sorted by price (descending).
   */
  getBids(): PriceLevel[] {
    const entries = Array.from(this.bids.entries());
    // Sort by price descending (higher prices first)
    entries.sort((a, b) => parseFloat(b[0]) - parseFloat(a[0]));
    return entries.map(([price, size]) => ({
      side: "bid",
      price,
      size,
    }));
  }

  /**
   * Get all ask levels sorted by price (ascending).
   */
  getAsks(): PriceLevel[] {
    const entries = Array.from(this.asks.entries());
    // Sort by price ascending (lower prices first)
    entries.sort((a, b) => parseFloat(a[0]) - parseFloat(b[0]));
    return entries.map(([price, size]) => ({
      side: "ask",
      price,
      size,
    }));
  }

  /**
   * Get top N bid levels.
   */
  getTopBids(n: number): PriceLevel[] {
    return this.getBids().slice(0, n);
  }

  /**
   * Get top N ask levels.
   */
  getTopAsks(n: number): PriceLevel[] {
    return this.getAsks().slice(0, n);
  }

  /**
   * Get the best bid (highest bid price) as [price, size].
   */
  bestBid(): [string, string] | undefined {
    const bids = this.getBids();
    if (bids.length === 0) return undefined;
    return [bids[0].price, bids[0].size];
  }

  /**
   * Get the best ask (lowest ask price) as [price, size].
   */
  bestAsk(): [string, string] | undefined {
    const asks = this.getAsks();
    if (asks.length === 0) return undefined;
    return [asks[0].price, asks[0].size];
  }

  /**
   * Get the spread as a string (best_ask - best_bid).
   */
  spread(): string | undefined {
    const bid = this.bestBid();
    const ask = this.bestAsk();
    if (!bid || !ask) return undefined;

    try {
      const bidPrice = new Decimal(bid[0]);
      const askPrice = new Decimal(ask[0]);
      const spread = askPrice.greaterThan(bidPrice)
        ? askPrice.minus(bidPrice)
        : new Decimal(0);
      return spread.toFixed(6);
    } catch {
      return undefined;
    }
  }

  /**
   * Get the midpoint price as a string.
   */
  midpoint(): string | undefined {
    const bid = this.bestBid();
    const ask = this.bestAsk();
    if (!bid || !ask) return undefined;

    try {
      const bidPrice = new Decimal(bid[0]);
      const askPrice = new Decimal(ask[0]);
      return bidPrice.plus(askPrice).dividedBy(2).toFixed(6);
    } catch {
      return undefined;
    }
  }

  /**
   * Get size at a specific bid price.
   */
  bidSizeAt(price: string): string | undefined {
    return this.bids.get(price);
  }

  /**
   * Get size at a specific ask price.
   */
  askSizeAt(price: string): string | undefined {
    return this.asks.get(price);
  }

  /**
   * Get total bid depth (sum of all bid sizes).
   */
  totalBidDepth(): number {
    let total = new Decimal(0);
    for (const size of this.bids.values()) {
      try {
        total = total.plus(new Decimal(size));
      } catch {
        // skip unparseable values
      }
    }
    return total.toNumber();
  }

  /**
   * Get total ask depth (sum of all ask sizes).
   */
  totalAskDepth(): number {
    let total = new Decimal(0);
    for (const size of this.asks.values()) {
      try {
        total = total.plus(new Decimal(size));
      } catch {
        // skip unparseable values
      }
    }
    return total.toNumber();
  }

  /** Number of bid levels */
  bidCount(): number {
    return this.bids.size;
  }

  /** Number of ask levels */
  askCount(): number {
    return this.asks.size;
  }

  /** Whether the orderbook has received its initial snapshot */
  hasSnapshot(): boolean {
    return this._hasSnapshot;
  }

  /** Current expected sequence number */
  expectedSequence(): number {
    return this.expectedSeq;
  }

  /** Last update timestamp */
  lastTimestamp(): string | undefined {
    return this._lastTimestamp;
  }

  /** Clear the orderbook state (for resync) */
  clear(): void {
    this.bids.clear();
    this.asks.clear();
    this.expectedSeq = 0;
    this._hasSnapshot = false;
    this._lastTimestamp = undefined;
  }
}
