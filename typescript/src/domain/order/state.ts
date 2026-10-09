import Decimal from "decimal.js";

import type { OrderBookId, PubkeyStr } from "../../shared";
import { limitOrderFromUpdate } from "./convert";
import { limitOrderIsLive, OrderStatus, type LimitOrder } from "./model";
import { closureScope, ClosureScope, type ClosureUpdate, type OrderUpdate } from "./wire";

/** Result of applying committed order state to {@link UserOpenLimitOrders}. */
export enum ApplyOutcome {
  /** A live order that was not tracked yet. */
  Inserted = "inserted",
  /** A tracked order replaced by newer (or equal) committed state. */
  Updated = "updated",
  /** A tracked order that is no longer live was removed. */
  Removed = "removed",
  /** Older than the tracked state (e.g. a live fact racing a snapshot). */
  Stale = "stale",
  /** Neither live nor tracked; nothing changed. */
  Ignored = "ignored",
}

/**
 * A wallet's live limit orders (resting, or with fills awaiting on-chain
 * confirmation), grouped by market and orderbook.
 *
 * Seed it from the WS `user` snapshot (`convertSnapshotOrders`) or REST pages,
 * then {@link apply} every live `order` fact. Each fact carries the order's
 * complete state, so application is a revision-guarded replace. The container
 * remembers the revision at which each order stopped being live, and every
 * applied closure, so older state (a REST page or snapshot racing live facts)
 * cannot reopen a closed order.
 */
export class UserOpenLimitOrders {
  readonly orders: Map<PubkeyStr, Map<OrderBookId, LimitOrder[]>>;
  /** Committed revision at which each untracked order stopped being live. */
  private readonly retired: Map<string, bigint>;
  /** Applied closures, re-applied to older state inserted later. */
  private readonly closures: Array<[ClosureScope, ClosureUpdate]>;

  constructor() {
    this.orders = new Map();
    this.retired = new Map();
    this.closures = [];
  }

  get(market: PubkeyStr, orderbookId: OrderBookId): LimitOrder[] | undefined {
    return this.orders.get(market)?.get(orderbookId);
  }

  getByMarket(market: PubkeyStr): Map<OrderBookId, LimitOrder[]> | undefined {
    return this.orders.get(market);
  }

  /** Find a tracked order by hash. */
  getByHash(orderHash: string): LimitOrder | undefined {
    for (const order of this.all()) {
      if (order.orderHash === orderHash) return order;
    }
    return undefined;
  }

  /** Every tracked order. */
  *all(): IterableIterator<LimitOrder> {
    for (const byOrderbook of this.orders.values()) {
      for (const orders of byOrderbook.values()) {
        yield* orders;
      }
    }
  }

  /**
   * Apply a live WS `order` fact. Throws (`SdkError` `Validation`), changing
   * nothing, when the fact's signed amounts define no limit price.
   */
  apply(update: OrderUpdate): ApplyOutcome {
    return this.applyOrder(limitOrderFromUpdate(update));
  }

  /**
   * Apply converted committed order state (live fact, snapshot, or REST).
   *
   * State older than the tracked `committedRevision`, or not newer than the
   * revision at which the order stopped being live, is ignored. A closure
   * committed after the state still closes it. A live order replaces the
   * tracked copy; one that is no longer live is removed. The order is copied,
   * never mutated.
   */
  applyOrder(source: LimitOrder): ApplyOutcome {
    const order = { ...source };
    const existing = this.getByHash(order.orderHash);
    if (existing) {
      if (order.committedRevision < existing.committedRevision) {
        return ApplyOutcome.Stale;
      }
    } else {
      const retiredAt = this.retired.get(order.orderHash);
      if (retiredAt !== undefined && order.committedRevision <= retiredAt) {
        return ApplyOutcome.Stale;
      }
    }
    for (const [scope, closure] of this.closures) {
      if (closure.commit.committed_revision > order.committedRevision) {
        closeCovered(order, closure, scope);
      }
    }
    const wasTracked = this.take(order.orderHash) !== undefined;
    if (!limitOrderIsLive(order)) {
      this.retire(order);
      return wasTracked ? ApplyOutcome.Removed : ApplyOutcome.Ignored;
    }
    this.retired.delete(order.orderHash);
    let byOrderbook = this.orders.get(order.marketPubkey);
    if (!byOrderbook) {
      byOrderbook = new Map();
      this.orders.set(order.marketPubkey, byOrderbook);
    }
    const orders = byOrderbook.get(order.orderbookId) ?? [];
    orders.push(order);
    byOrderbook.set(order.orderbookId, orders);
    return wasTracked ? ApplyOutcome.Updated : ApplyOutcome.Inserted;
  }

  /**
   * Apply a committed closure cutoff locally: orders in scope with
   * `acceptedSeq <= closure.accepted_seq` stop resting (their open base
   * becomes cancelled) and are removed unless fills still await confirmation.
   * The container is assumed to hold one wallet's orders.
   *
   * Returns the number of orders closed, or `undefined` when the scope cannot
   * be evaluated from order data (deposit-token, account, settlement-profile,
   * or unknown scopes): refetch the orders in that case. Later per-order facts
   * remain authoritative.
   */
  applyClosure(closure: ClosureUpdate): number | undefined {
    const scope = closureScope(closure);
    if (
      scope === undefined ||
      scope === ClosureScope.DepositToken ||
      scope === ClosureScope.Account ||
      scope === ClosureScope.SettlementProfile
    ) {
      return undefined;
    }
    let closed = 0;
    for (const byOrderbook of this.orders.values()) {
      for (const [orderbookId, orders] of byOrderbook.entries()) {
        for (const order of orders) {
          if (closeCovered(order, closure, scope)) closed += 1;
        }
        byOrderbook.set(
          orderbookId,
          orders.filter((order) => {
            const live = limitOrderIsLive(order);
            if (!live) this.retire(order);
            return live;
          }),
        );
      }
    }
    this.closures.push([scope, closure]);
    return closed;
  }

  /** Stop tracking an order without recording why; older state can add it again. */
  remove(orderHash: string): void {
    this.take(orderHash);
  }

  /** Forget every order, retired revision, and closure (e.g. before reseeding from a new snapshot). */
  clear(): void {
    this.orders.clear();
    this.retired.clear();
    this.closures.length = 0;
  }

  isEmpty(): boolean {
    for (const byOrderbook of this.orders.values()) {
      for (const orders of byOrderbook.values()) {
        if (orders.length > 0) return false;
      }
    }
    return true;
  }

  private take(orderHash: string): LimitOrder | undefined {
    for (const byOrderbook of this.orders.values()) {
      for (const orders of byOrderbook.values()) {
        const index = orders.findIndex((order) => order.orderHash === orderHash);
        if (index !== -1) {
          return orders.splice(index, 1)[0];
        }
      }
    }
    return undefined;
  }

  /** Record the revision at which `order` stopped being live. */
  private retire(order: LimitOrder): void {
    const previous = this.retired.get(order.orderHash);
    if (previous === undefined || order.committedRevision > previous) {
      this.retired.set(order.orderHash, order.committedRevision);
    }
  }
}

/**
 * Close `order` locally when `closure` covers it: in scope, accepted at or
 * before the cutoff, and still resting. Returns whether it was closed.
 */
function closeCovered(order: LimitOrder, closure: ClosureUpdate, scope: ClosureScope): boolean {
  const key = closure.scope_key;
  let inScope: boolean;
  switch (scope) {
    case ClosureScope.Market:
      inScope = order.marketPubkey === key;
      break;
    case ClosureScope.Book:
      inScope = order.orderbookId === key;
      break;
    case ClosureScope.WalletBook: {
      const separator = key.indexOf(":");
      inScope = separator !== -1 && order.orderbookId === key.slice(separator + 1);
      break;
    }
    default:
      inScope = true;
  }
  const remaining = new Decimal(order.remainingSize);
  if (!(order.acceptedSeq <= closure.accepted_seq && inScope && remaining.gt(0))) {
    return false;
  }
  order.cancelledSize = new Decimal(order.cancelledSize).plus(remaining).toFixed();
  order.remainingSize = "0";
  order.closedReason ??= closure.reason;
  order.status = OrderStatus.Closed;
  // Older order facts must not reopen the closed order.
  if (closure.commit.committed_revision > order.committedRevision) {
    order.committedRevision = closure.commit.committed_revision;
  }
  return true;
}
