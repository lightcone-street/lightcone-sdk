/**
 * Price history state management.
 *
 * Maintains local state for price history candles.
 */

import { Resolution } from "../../shared";
import type { Candle, PriceHistoryData } from "../types";
import { toCandle } from "../types";

/**
 * Key for price history subscriptions.
 */
export class PriceHistoryKey {
  readonly orderbookId: string;
  readonly resolution: string;

  constructor(orderbookId: string, resolution: string) {
    this.orderbookId = orderbookId;
    this.resolution = resolution;
  }

  toString(): string {
    return `${this.orderbookId}:${this.resolution}`;
  }

  equals(other: PriceHistoryKey): boolean {
    return (
      this.orderbookId === other.orderbookId &&
      this.resolution === other.resolution
    );
  }
}

/**
 * Price history state for a single orderbook/resolution pair.
 */
export class PriceHistory {
  /** Maximum number of candles to maintain */
  private static readonly MAX_CANDLES = 1000;

  /** Orderbook identifier */
  readonly orderbookId: string;
  /** Resolution (1m, 5m, 15m, 1h, 4h, 1d) */
  readonly resolution: string;
  /** Whether OHLCV data is included */
  includeOhlcv: boolean;
  /** Candles sorted by timestamp (newest first) */
  private _candles: Candle[] = [];
  /** Index by timestamp for fast lookup */
  private candleIndex: Map<number, number> = new Map();
  /** Last server timestamp */
  private _lastTimestamp?: number;
  /** Server time from last message */
  private _serverTime?: number;
  /** Whether initial snapshot has been received */
  private _hasSnapshot: boolean = false;

  constructor(orderbookId: string, resolution: string, includeOhlcv: boolean) {
    this.orderbookId = orderbookId;
    this.resolution = resolution;
    this.includeOhlcv = includeOhlcv;
  }

  /**
   * Apply a snapshot (historical candles).
   */
  applySnapshot(data: PriceHistoryData): void {
    // Clear existing state
    this._candles = [];
    this.candleIndex.clear();

    // Apply candles (they come newest-first from server)
    for (const candle of data.prices) {
      const idx = this._candles.length;
      this.candleIndex.set(candle.t, idx);
      this._candles.push(candle);
    }

    this._lastTimestamp = data.last_timestamp;
    this._serverTime = data.server_time;
    this._hasSnapshot = true;

    // Update include_ohlcv if provided
    if (data.include_ohlcv !== undefined) {
      this.includeOhlcv = data.include_ohlcv;
    }
  }

  /**
   * Apply an update (new or updated candle).
   */
  applyUpdate(data: PriceHistoryData): void {
    const candle = toCandle(data);
    if (candle) {
      this.updateOrAppendCandle(candle);
    }
  }

  /**
   * Update an existing candle or append a new one.
   */
  private updateOrAppendCandle(candle: Candle): void {
    const existingIdx = this.candleIndex.get(candle.t);
    if (existingIdx !== undefined) {
      // Update existing candle
      this._candles[existingIdx] = candle;
    } else {
      // New candle - insert at the correct position (newest first)
      const insertPos = this._candles.findIndex((c) => c.t < candle.t);
      const pos = insertPos === -1 ? this._candles.length : insertPos;

      // Update indices for candles that will shift
      for (const [ts, idx] of this.candleIndex.entries()) {
        if (idx >= pos) {
          this.candleIndex.set(ts, idx + 1);
        }
      }

      this.candleIndex.set(candle.t, pos);
      this._candles.splice(pos, 0, candle);

      // Trim to max candles limit
      while (this._candles.length > PriceHistory.MAX_CANDLES) {
        const removed = this._candles.pop();
        if (removed) {
          this.candleIndex.delete(removed.t);
        }
      }
    }

    // Update last timestamp
    if (this._candles.length > 0) {
      this._lastTimestamp = this._candles[0].t;
    }
  }

  /**
   * Handle heartbeat (update server time).
   */
  applyHeartbeat(data: PriceHistoryData): void {
    this._serverTime = data.server_time;
  }

  /**
   * Apply any price history event.
   */
  applyEvent(data: PriceHistoryData): void {
    switch (data.event_type) {
      case "snapshot":
        this.applySnapshot(data);
        break;
      case "update":
        this.applyUpdate(data);
        break;
      case "heartbeat":
        this.applyHeartbeat(data);
        break;
      default:
        console.warn(`Unknown price history event type: ${data.event_type}`);
    }
  }

  /**
   * Get all candles (newest first).
   */
  candles(): Candle[] {
    return this._candles;
  }

  /**
   * Get the N most recent candles.
   */
  recentCandles(n: number): Candle[] {
    return this._candles.slice(0, Math.min(n, this._candles.length));
  }

  /**
   * Get a candle by timestamp.
   */
  getCandle(timestamp: number): Candle | undefined {
    const idx = this.candleIndex.get(timestamp);
    if (idx === undefined) return undefined;
    return this._candles[idx];
  }

  /**
   * Get the most recent candle.
   */
  latestCandle(): Candle | undefined {
    return this._candles[0];
  }

  /**
   * Get the oldest candle.
   */
  oldestCandle(): Candle | undefined {
    return this._candles[this._candles.length - 1];
  }

  /**
   * Get current midpoint price (from most recent candle).
   */
  currentMidpoint(): string | undefined {
    return this._candles[0]?.m;
  }

  /**
   * Get current best bid (from most recent candle).
   */
  currentBestBid(): string | undefined {
    return this._candles[0]?.bb;
  }

  /**
   * Get current best ask (from most recent candle).
   */
  currentBestAsk(): string | undefined {
    return this._candles[0]?.ba;
  }

  /** Number of candles */
  candleCount(): number {
    return this._candles.length;
  }

  /** Whether the price history has received its initial snapshot */
  hasSnapshot(): boolean {
    return this._hasSnapshot;
  }

  /** Last candle timestamp */
  lastTimestamp(): number | undefined {
    return this._lastTimestamp;
  }

  /** Server time from last message */
  serverTime(): number | undefined {
    return this._serverTime;
  }

  /**
   * Get resolution as enum.
   */
  resolutionEnum(): Resolution | undefined {
    switch (this.resolution) {
      case "1m":
        return Resolution.OneMinute;
      case "5m":
        return Resolution.FiveMinutes;
      case "15m":
        return Resolution.FifteenMinutes;
      case "1h":
        return Resolution.OneHour;
      case "4h":
        return Resolution.FourHours;
      case "1d":
        return Resolution.OneDay;
      default:
        return undefined;
    }
  }

  /** Clear the price history (for disconnect/resync) */
  clear(): void {
    this._candles = [];
    this.candleIndex.clear();
    this._lastTimestamp = undefined;
    this._serverTime = undefined;
    this._hasSnapshot = false;
  }
}
