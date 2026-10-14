import type { OrderBookId, PubkeyStr } from "../../shared";
import type { LimitOrder } from "./index";
import type { OrderUpdate } from "./wire";
import { orderFromUpdate } from "./convert";

export class UserOpenLimitOrders {
  readonly orders: Map<PubkeyStr, Map<OrderBookId, LimitOrder[]>>;

  constructor() {
    this.orders = new Map();
  }

  get(market: PubkeyStr, orderbookId: OrderBookId): LimitOrder[] | undefined {
    return this.orders.get(market)?.get(orderbookId);
  }

  getByMarket(market: PubkeyStr): Map<OrderBookId, LimitOrder[]> | undefined {
    return this.orders.get(market);
  }

  insert(order: LimitOrder): void {
    let marketMap = this.orders.get(order.marketPubkey);
    if (!marketMap) {
      marketMap = new Map();
      this.orders.set(order.marketPubkey, marketMap);
    }
    const current = marketMap.get(order.orderbookId) ?? [];
    current.push(order);
    marketMap.set(order.orderbookId, current);
  }

  upsert(update: OrderUpdate): void {
    const order = orderFromUpdate(update);
    let marketMap = this.orders.get(order.marketPubkey);
    if (!marketMap) {
      marketMap = new Map();
      this.orders.set(order.marketPubkey, marketMap);
    }
    const current = marketMap.get(order.orderbookId) ?? [];
    const next = current.filter((existing) => existing.orderHash !== order.orderHash);
    next.push(order);
    marketMap.set(order.orderbookId, next);
  }

  remove(orderHash: string): void {
    for (const [, marketMap] of this.orders.entries()) {
      for (const [orderbookId, orderList] of marketMap.entries()) {
        marketMap.set(
          orderbookId,
          orderList.filter((order) => order.orderHash !== orderHash)
        );
      }
    }
  }

  clear(): void {
    this.orders.clear();
  }

  isEmpty(): boolean {
    for (const marketMap of this.orders.values()) {
      for (const orderList of marketMap.values()) {
        if (orderList.length > 0) {
          return false;
        }
      }
    }
    return true;
  }
}
