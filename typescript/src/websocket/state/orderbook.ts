/**
 * Local orderbook state management.
 *
 * Maintains a local copy of the orderbook state, applying deltas from
 * WebSocket updates.
 *
 * Note: Internally uses string keys/values to match the string-based API.
 * For numeric comparisons, parse the strings as needed.
 */

import { WebSocketError } from "../error";
import type { BookUpdateData, PriceLevel } from "../types";

/**
 * Local orderbook state.
 */
export class LocalOrderbook {
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
    // Clear existing state
    this.bids.clear();
    this.asks.clear();

    // Apply all levels
    for (const level of update.bids) {
      if (parseFloat(level.size) !== 0) {
        this.bids.set(level.price, level.size);
      }
    }

    for (const level of update.asks) {
      if (parseFloat(level.size) !== 0) {
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
    // Check sequence number
    if (update.seq !== this.expectedSeq) {
      throw WebSocketError.sequenceGap(this.expectedSeq, update.seq);
    }

    // Apply bid updates
    for (const level of update.bids) {
      if (parseFloat(level.size) === 0) {
        this.bids.delete(level.price);
      } else {
        this.bids.set(level.price, level.size);
      }
    }

    // Apply ask updates
    for (const level of update.asks) {
      if (parseFloat(level.size) === 0) {
        this.asks.delete(level.price);
      } else {
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
   * Note: This parses as number for the calculation.
   */
  spread(): string | undefined {
    const bid = this.bestBid();
    const ask = this.bestAsk();
    if (!bid || !ask) return undefined;

    const bidPrice = parseFloat(bid[0]);
    const askPrice = parseFloat(ask[0]);
    if (isNaN(bidPrice) || isNaN(askPrice)) return undefined;

    const spread = askPrice > bidPrice ? askPrice - bidPrice : 0;
    return spread.toFixed(6);
  }

  /**
   * Get the midpoint price as a string.
   * Note: This parses as number for the calculation.
   */
  midpoint(): string | undefined {
    const bid = this.bestBid();
    const ask = this.bestAsk();
    if (!bid || !ask) return undefined;

    const bidPrice = parseFloat(bid[0]);
    const askPrice = parseFloat(ask[0]);
    if (isNaN(bidPrice) || isNaN(askPrice)) return undefined;

    return ((bidPrice + askPrice) / 2).toFixed(6);
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
   * Note: This parses as number for the calculation.
   */
  totalBidDepth(): number {
    let total = 0;
    for (const size of this.bids.values()) {
      const parsed = parseFloat(size);
      if (!isNaN(parsed)) total += parsed;
    }
    return total;
  }

  /**
   * Get total ask depth (sum of all ask sizes).
   * Note: This parses as number for the calculation.
   */
  totalAskDepth(): number {
    let total = 0;
    for (const size of this.asks.values()) {
      const parsed = parseFloat(size);
      if (!isNaN(parsed)) total += parsed;
    }
    return total;
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
