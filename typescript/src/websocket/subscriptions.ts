/**
 * Subscription management for WebSocket channels.
 *
 * Tracks active subscriptions and supports re-subscribing after reconnect.
 */

import type { SubscribeParams } from "./types";
import {
  bookUpdateParams,
  tradesParams,
  userParams,
  priceHistoryParams,
  marketParams,
} from "./types";

/**
 * Represents a subscription to a specific channel.
 */
export type Subscription =
  | { type: "BookUpdate"; orderbookIds: string[] }
  | { type: "Trades"; orderbookIds: string[] }
  | { type: "User"; user: string }
  | {
      type: "PriceHistory";
      orderbookId: string;
      resolution: string;
      includeOhlcv: boolean;
    }
  | { type: "Market"; marketPubkey: string };

/**
 * Convert subscription to SubscribeParams for sending.
 */
export function subscriptionToParams(sub: Subscription): SubscribeParams {
  switch (sub.type) {
    case "BookUpdate":
      return bookUpdateParams(sub.orderbookIds);
    case "Trades":
      return tradesParams(sub.orderbookIds);
    case "User":
      return userParams(sub.user);
    case "PriceHistory":
      return priceHistoryParams(
        sub.orderbookId,
        sub.resolution,
        sub.includeOhlcv
      );
    case "Market":
      return marketParams(sub.marketPubkey);
  }
}

/**
 * Get the subscription type as a string.
 */
export function subscriptionType(sub: Subscription): string {
  return sub.type.toLowerCase();
}

/**
 * Manages active subscriptions.
 */
export class SubscriptionManager {
  /** Book update subscriptions (orderbook_id set) */
  private bookUpdates: Set<string> = new Set();
  /** Trades subscriptions (orderbook_id set) */
  private trades: Set<string> = new Set();
  /** User subscriptions (user pubkey set) */
  private users: Set<string> = new Set();
  /** Price history subscriptions (key -> params) */
  private priceHistory: Map<string, [string, string, boolean]> = new Map();
  /** Market subscriptions (market_pubkey set) */
  private markets: Set<string> = new Set();

  /**
   * Add a book update subscription.
   */
  addBookUpdate(orderbookIds: string[]): void {
    for (const id of orderbookIds) {
      this.bookUpdates.add(id);
    }
  }

  /**
   * Remove a book update subscription.
   */
  removeBookUpdate(orderbookIds: string[]): void {
    for (const id of orderbookIds) {
      this.bookUpdates.delete(id);
    }
  }

  /**
   * Check if subscribed to book updates for an orderbook.
   */
  isSubscribedBookUpdate(orderbookId: string): boolean {
    return this.bookUpdates.has(orderbookId);
  }

  /**
   * Add a trades subscription.
   */
  addTrades(orderbookIds: string[]): void {
    for (const id of orderbookIds) {
      this.trades.add(id);
    }
  }

  /**
   * Remove a trades subscription.
   */
  removeTrades(orderbookIds: string[]): void {
    for (const id of orderbookIds) {
      this.trades.delete(id);
    }
  }

  /**
   * Check if subscribed to trades for an orderbook.
   */
  isSubscribedTrades(orderbookId: string): boolean {
    return this.trades.has(orderbookId);
  }

  /**
   * Add a user subscription.
   */
  addUser(user: string): void {
    this.users.add(user);
  }

  /**
   * Remove a user subscription.
   */
  removeUser(user: string): void {
    this.users.delete(user);
  }

  /**
   * Check if subscribed to a user.
   */
  isSubscribedUser(user: string): boolean {
    return this.users.has(user);
  }

  /**
   * Add a price history subscription.
   */
  addPriceHistory(
    orderbookId: string,
    resolution: string,
    includeOhlcv: boolean
  ): void {
    const key = `${orderbookId}:${resolution}`;
    this.priceHistory.set(key, [orderbookId, resolution, includeOhlcv]);
  }

  /**
   * Remove a price history subscription.
   */
  removePriceHistory(orderbookId: string, resolution: string): void {
    const key = `${orderbookId}:${resolution}`;
    this.priceHistory.delete(key);
  }

  /**
   * Check if subscribed to price history for an orderbook/resolution.
   */
  isSubscribedPriceHistory(orderbookId: string, resolution: string): boolean {
    const key = `${orderbookId}:${resolution}`;
    return this.priceHistory.has(key);
  }

  /**
   * Add a market subscription.
   */
  addMarket(marketPubkey: string): void {
    this.markets.add(marketPubkey);
  }

  /**
   * Remove a market subscription.
   */
  removeMarket(marketPubkey: string): void {
    this.markets.delete(marketPubkey);
  }

  /**
   * Check if subscribed to market events.
   */
  isSubscribedMarket(marketPubkey: string): boolean {
    return this.markets.has(marketPubkey) || this.markets.has("all");
  }

  /**
   * Get all subscriptions for re-subscribing after reconnect.
   */
  getAllSubscriptions(): Subscription[] {
    const subs: Subscription[] = [];

    // Group book updates
    if (this.bookUpdates.size > 0) {
      subs.push({
        type: "BookUpdate",
        orderbookIds: Array.from(this.bookUpdates),
      });
    }

    // Group trades
    if (this.trades.size > 0) {
      subs.push({
        type: "Trades",
        orderbookIds: Array.from(this.trades),
      });
    }

    // Users
    for (const user of this.users) {
      subs.push({ type: "User", user });
    }

    // Price history
    for (const [orderbookId, resolution, includeOhlcv] of this.priceHistory.values()) {
      subs.push({
        type: "PriceHistory",
        orderbookId,
        resolution,
        includeOhlcv,
      });
    }

    // Markets
    for (const marketPubkey of this.markets) {
      subs.push({ type: "Market", marketPubkey });
    }

    return subs;
  }

  /**
   * Clear all subscriptions.
   */
  clear(): void {
    this.bookUpdates.clear();
    this.trades.clear();
    this.users.clear();
    this.priceHistory.clear();
    this.markets.clear();
  }

  /**
   * Check if there are any active subscriptions.
   */
  hasSubscriptions(): boolean {
    return (
      this.bookUpdates.size > 0 ||
      this.trades.size > 0 ||
      this.users.size > 0 ||
      this.priceHistory.size > 0 ||
      this.markets.size > 0
    );
  }

  /**
   * Get count of active subscriptions.
   */
  subscriptionCount(): number {
    return (
      this.bookUpdates.size +
      this.trades.size +
      this.users.size +
      this.priceHistory.size +
      this.markets.size
    );
  }

  /**
   * Get all subscribed orderbook IDs (for book updates).
   */
  bookUpdateOrderbooks(): string[] {
    return Array.from(this.bookUpdates);
  }

  /**
   * Get all subscribed orderbook IDs (for trades).
   */
  tradeOrderbooks(): string[] {
    return Array.from(this.trades);
  }

  /**
   * Get all subscribed users.
   */
  subscribedUsers(): string[] {
    return Array.from(this.users);
  }

  /**
   * Get all subscribed markets.
   */
  subscribedMarkets(): string[] {
    return Array.from(this.markets);
  }
}
