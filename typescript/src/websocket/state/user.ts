/**
 * User state management.
 *
 * Maintains local state for user orders and balances.
 */

import type {
  Order,
  Balance,
  BalanceEntry,
  UserEventData,
} from "../types";

/**
 * User state tracking orders and balances.
 */
export class UserState {
  /** User public key */
  readonly user: string;
  /** Open orders by order hash */
  orders: Map<string, Order> = new Map();
  /** Balances by market_pubkey:deposit_mint key */
  balances: Map<string, BalanceEntry> = new Map();
  /** Whether initial snapshot has been received */
  private _hasSnapshot: boolean = false;
  /** Last update timestamp */
  private _lastTimestamp?: string;

  constructor(user: string) {
    this.user = user;
  }

  /**
   * Apply a snapshot (full user state).
   */
  applySnapshot(data: UserEventData): void {
    // Clear existing state
    this.orders.clear();
    this.balances.clear();

    // Apply orders
    for (const order of data.orders) {
      this.orders.set(order.order_hash, order);
    }

    // Apply balances
    for (const [key, balance] of Object.entries(data.balances)) {
      this.balances.set(key, balance);
    }

    this._hasSnapshot = true;
    this._lastTimestamp = data.timestamp;
  }

  /**
   * Apply an order update.
   */
  applyOrderUpdate(data: UserEventData): void {
    if (!data.order) return;

    const update = data.order;
    const orderHash = update.order_hash;

    // If remaining is 0, the order is fully filled or cancelled - remove it
    if (parseFloat(update.remaining) === 0) {
      this.orders.delete(orderHash);
    } else {
      const existing = this.orders.get(orderHash);
      if (existing) {
        // Update existing order
        existing.remaining = update.remaining;
        existing.filled = update.filled;
      } else {
        // New order - construct it from the update
        if (data.market_pubkey && data.orderbook_id) {
          const order: Order = {
            order_hash: orderHash,
            market_pubkey: data.market_pubkey,
            orderbook_id: data.orderbook_id,
            side: update.side,
            maker_amount: update.remaining, // Approximate
            taker_amount: "0", // Unknown
            remaining: update.remaining,
            filled: update.filled,
            price: update.price,
            created_at: update.created_at,
            expiration: 0,
          };
          this.orders.set(orderHash, order);
        }
      }
    }

    // Apply balance updates if present
    if (update.balance) {
      this.applyBalanceFromOrder(data, update.balance);
    }

    this._lastTimestamp = data.timestamp;
  }

  /**
   * Apply a balance update.
   */
  applyBalanceUpdate(data: UserEventData): void {
    if (!data.market_pubkey || !data.deposit_mint || !data.balance) return;

    const key = `${data.market_pubkey}:${data.deposit_mint}`;
    const entry: BalanceEntry = {
      market_pubkey: data.market_pubkey,
      deposit_mint: data.deposit_mint,
      outcomes: data.balance.outcomes,
    };
    this.balances.set(key, entry);

    this._lastTimestamp = data.timestamp;
  }

  /**
   * Apply balance from order update.
   */
  private applyBalanceFromOrder(data: UserEventData, balance: Balance): void {
    if (data.market_pubkey && data.deposit_mint) {
      const key = `${data.market_pubkey}:${data.deposit_mint}`;
      const entry: BalanceEntry = {
        market_pubkey: data.market_pubkey,
        deposit_mint: data.deposit_mint,
        outcomes: balance.outcomes,
      };
      this.balances.set(key, entry);
    } else if (data.market_pubkey) {
      // If no deposit_mint, update existing entry with matching market
      for (const [key, entry] of this.balances.entries()) {
        if (key.startsWith(data.market_pubkey)) {
          entry.outcomes = balance.outcomes;
          break;
        }
      }
    }
  }

  /**
   * Apply any user event.
   */
  applyEvent(data: UserEventData): void {
    switch (data.event_type) {
      case "snapshot":
        this.applySnapshot(data);
        break;
      case "order_update":
        this.applyOrderUpdate(data);
        break;
      case "balance_update":
        this.applyBalanceUpdate(data);
        break;
      default:
        console.warn(`Unknown user event type: ${data.event_type}`);
    }
  }

  /**
   * Get an order by hash.
   */
  getOrder(orderHash: string): Order | undefined {
    return this.orders.get(orderHash);
  }

  /**
   * Get all open orders.
   */
  openOrders(): Order[] {
    return Array.from(this.orders.values());
  }

  /**
   * Get orders for a specific market.
   */
  ordersForMarket(marketPubkey: string): Order[] {
    return this.openOrders().filter(
      (order) => order.market_pubkey === marketPubkey
    );
  }

  /**
   * Get orders for a specific orderbook.
   */
  ordersForOrderbook(orderbookId: string): Order[] {
    return this.openOrders().filter(
      (order) => order.orderbook_id === orderbookId
    );
  }

  /**
   * Get balance for a market/deposit_mint pair.
   */
  getBalance(marketPubkey: string, depositMint: string): BalanceEntry | undefined {
    const key = `${marketPubkey}:${depositMint}`;
    return this.balances.get(key);
  }

  /**
   * Get all balances.
   */
  allBalances(): BalanceEntry[] {
    return Array.from(this.balances.values());
  }

  /**
   * Get total idle balance for a specific outcome as a string.
   */
  idleBalanceForOutcome(
    marketPubkey: string,
    depositMint: string,
    outcomeIndex: number
  ): string | undefined {
    const balance = this.getBalance(marketPubkey, depositMint);
    if (!balance) return undefined;
    const outcome = balance.outcomes.find(
      (o) => o.outcome_index === outcomeIndex
    );
    return outcome?.idle;
  }

  /**
   * Get total on-book balance for a specific outcome as a string.
   */
  onBookBalanceForOutcome(
    marketPubkey: string,
    depositMint: string,
    outcomeIndex: number
  ): string | undefined {
    const balance = this.getBalance(marketPubkey, depositMint);
    if (!balance) return undefined;
    const outcome = balance.outcomes.find(
      (o) => o.outcome_index === outcomeIndex
    );
    return outcome?.on_book;
  }

  /** Number of open orders */
  orderCount(): number {
    return this.orders.size;
  }

  /** Whether the user state has received its initial snapshot */
  hasSnapshot(): boolean {
    return this._hasSnapshot;
  }

  /** Last update timestamp */
  lastTimestamp(): string | undefined {
    return this._lastTimestamp;
  }

  /** Clear the user state (for disconnect/resync) */
  clear(): void {
    this.orders.clear();
    this.balances.clear();
    this._hasSnapshot = false;
    this._lastTimestamp = undefined;
  }
}
