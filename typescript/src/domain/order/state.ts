import type { OrderBookId, PubkeyStr } from "../../shared";
import type { Order, TriggerOrder } from "./index";
import type { OrderUpdate } from "./wire";
import { orderFromUpdate } from "./convert";

export class UserOpenOrders {
  readonly orders: Map<PubkeyStr, Order[]>;

  constructor() {
    this.orders = new Map();
  }

  get(market: PubkeyStr): Order[] | undefined {
    return this.orders.get(market);
  }

  insert(order: Order): void {
    const current = this.orders.get(order.marketPubkey) ?? [];
    current.push(order);
    this.orders.set(order.marketPubkey, current);
  }

  upsert(update: OrderUpdate): void {
    const order = orderFromUpdate(update);
    const current = this.orders.get(order.marketPubkey) ?? [];
    const next = current.filter((existing) => existing.orderHash !== order.orderHash);
    next.push(order);
    this.orders.set(order.marketPubkey, next);
  }

  remove(orderHash: string): void {
    for (const [market, orders] of this.orders.entries()) {
      this.orders.set(
        market,
        orders.filter((order) => order.orderHash !== orderHash)
      );
    }
  }

  clear(): void {
    this.orders.clear();
  }

  isEmpty(): boolean {
    return Array.from(this.orders.values()).every((orders) => orders.length === 0);
  }
}

export class UserTriggerOrders {
  readonly orders: Map<OrderBookId, TriggerOrder[]>;

  constructor() {
    this.orders = new Map();
  }

  get(orderbookId: OrderBookId): TriggerOrder[] | undefined {
    return this.orders.get(orderbookId);
  }

  getById(triggerOrderId: string): TriggerOrder | undefined {
    for (const orders of this.orders.values()) {
      const found = orders.find((order) => order.triggerOrderId === triggerOrderId);
      if (found) {
        return found;
      }
    }

    return undefined;
  }

  insert(order: TriggerOrder): void {
    const current = this.orders.get(order.orderbookId) ?? [];
    current.push(order);
    this.orders.set(order.orderbookId, current);
  }

  remove(triggerOrderId: string): TriggerOrder | undefined {
    for (const [orderbookId, orders] of this.orders.entries()) {
      const index = orders.findIndex((order) => order.triggerOrderId === triggerOrderId);
      if (index >= 0) {
        const [removed] = orders.splice(index, 1);
        this.orders.set(orderbookId, orders);
        return removed;
      }
    }

    return undefined;
  }

  clear(): void {
    this.orders.clear();
  }

  isEmpty(): boolean {
    return Array.from(this.orders.values()).every((orders) => orders.length === 0);
  }

  len(): number {
    return Array.from(this.orders.values()).reduce((acc, orders) => acc + orders.length, 0);
  }

  all(): TriggerOrder[] {
    return Array.from(this.orders.values()).flatMap((orders) => orders);
  }
}
