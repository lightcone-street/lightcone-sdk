import { describe, it, expect, beforeEach } from "vitest";
import { UserState } from "./user";
import type { UserEventData } from "../types";

/** Type assertion helper for tests */
function assertDefined<T>(value: T | undefined | null): asserts value is T {
  expect(value).toBeDefined();
}

describe("UserState", () => {
  let state: UserState;

  function createSnapshot(): UserEventData {
    return {
      event_type: "snapshot",
      orders: [
        {
          order_hash: "hash1",
          market_pubkey: "market1",
          orderbook_id: "ob1",
          side: 0,
          maker_amount: "0.001000",
          taker_amount: "0.000500",
          remaining: "0.000800",
          filled: "0.000200",
          price: "0.500000",
          created_at: 1704067200000,
          expiration: 0,
        },
      ],
      balances: {
        "market1:mint1": {
          market_pubkey: "market1",
          deposit_mint: "mint1",
          outcomes: [
            {
              outcome_index: 0,
              mint: "outcome_mint",
              idle: "0.005000",
              on_book: "0.001000",
            },
          ],
        },
      },
      timestamp: "2024-01-01T00:00:00.000Z",
    };
  }

  beforeEach(() => {
    state = new UserState("user1");
  });

  describe("applySnapshot", () => {
    it("applies a snapshot correctly", () => {
      state.applySnapshot(createSnapshot());

      expect(state.hasSnapshot()).toBe(true);
      expect(state.orderCount()).toBe(1);
      expect(state.getOrder("hash1")).toBeDefined();
    });

    it("applies balances", () => {
      state.applySnapshot(createSnapshot());

      const balance = state.getBalance("market1", "mint1");
      assertDefined(balance);
      expect(balance.outcomes[0].idle).toBe("0.005000");
    });

    it("clears previous state on new snapshot", () => {
      state.applySnapshot(createSnapshot());

      const newSnapshot: UserEventData = {
        event_type: "snapshot",
        orders: [],
        balances: {},
        timestamp: "2024-01-01T00:00:01.000Z",
      };
      state.applySnapshot(newSnapshot);

      expect(state.orderCount()).toBe(0);
      expect(state.allBalances()).toHaveLength(0);
    });
  });

  describe("applyOrderUpdate", () => {
    it("updates existing order", () => {
      state.applySnapshot(createSnapshot());

      const update: UserEventData = {
        event_type: "order_update",
        orders: [],
        balances: {},
        order: {
          order_hash: "hash1",
          price: "0.500000",
          fill_amount: "0.000100",
          remaining: "0.000700",
          filled: "0.000300",
          side: 0,
          is_maker: true,
          created_at: 1704067200000,
        },
        market_pubkey: "market1",
        orderbook_id: "ob1",
        timestamp: "2024-01-01T00:00:01.000Z",
      };
      state.applyOrderUpdate(update);

      const order = state.getOrder("hash1");
      assertDefined(order);
      expect(order.remaining).toBe("0.000700");
      expect(order.filled).toBe("0.000300");
    });

    it("removes order when fully filled", () => {
      state.applySnapshot(createSnapshot());

      const update: UserEventData = {
        event_type: "order_update",
        orders: [],
        balances: {},
        order: {
          order_hash: "hash1",
          price: "0.500000",
          fill_amount: "0.000800",
          remaining: "0",
          filled: "0.001000",
          side: 0,
          is_maker: true,
          created_at: 1704067200000,
        },
        market_pubkey: "market1",
        orderbook_id: "ob1",
        timestamp: "2024-01-01T00:00:01.000Z",
      };
      state.applyOrderUpdate(update);

      expect(state.getOrder("hash1")).toBeUndefined();
      expect(state.orderCount()).toBe(0);
    });
  });

  describe("applyBalanceUpdate", () => {
    it("updates balance", () => {
      state.applySnapshot(createSnapshot());

      const update: UserEventData = {
        event_type: "balance_update",
        orders: [],
        balances: {},
        balance: {
          outcomes: [
            {
              outcome_index: 0,
              mint: "outcome_mint",
              idle: "0.006000",
              on_book: "0.000500",
            },
          ],
        },
        market_pubkey: "market1",
        deposit_mint: "mint1",
        timestamp: "2024-01-01T00:00:01.000Z",
      };
      state.applyBalanceUpdate(update);

      const balance = state.getBalance("market1", "mint1");
      assertDefined(balance);
      expect(balance.outcomes[0].idle).toBe("0.006000");
      expect(balance.outcomes[0].on_book).toBe("0.000500");
    });
  });

  describe("applyEvent", () => {
    it("routes snapshot events", () => {
      state.applyEvent(createSnapshot());
      expect(state.hasSnapshot()).toBe(true);
    });

    it("routes order_update events", () => {
      state.applySnapshot(createSnapshot());

      const update: UserEventData = {
        event_type: "order_update",
        orders: [],
        balances: {},
        order: {
          order_hash: "hash1",
          price: "0.500000",
          fill_amount: "0.000100",
          remaining: "0.000700",
          filled: "0.000300",
          side: 0,
          is_maker: true,
          created_at: 1704067200000,
        },
        market_pubkey: "market1",
        orderbook_id: "ob1",
        timestamp: "2024-01-01T00:00:01.000Z",
      };
      state.applyEvent(update);

      const order = state.getOrder("hash1");
      assertDefined(order);
      expect(order.remaining).toBe("0.000700");
    });

    it("routes balance_update events", () => {
      state.applySnapshot(createSnapshot());

      const update: UserEventData = {
        event_type: "balance_update",
        orders: [],
        balances: {},
        balance: {
          outcomes: [
            {
              outcome_index: 0,
              mint: "outcome_mint",
              idle: "0.010000",
              on_book: "0.000000",
            },
          ],
        },
        market_pubkey: "market1",
        deposit_mint: "mint1",
        timestamp: "2024-01-01T00:00:01.000Z",
      };
      state.applyEvent(update);

      const balance = state.getBalance("market1", "mint1");
      assertDefined(balance);
      expect(balance.outcomes[0].idle).toBe("0.010000");
    });
  });

  describe("query methods", () => {
    beforeEach(() => {
      state.applySnapshot(createSnapshot());
    });

    it("openOrders returns all orders", () => {
      const orders = state.openOrders();
      expect(orders).toHaveLength(1);
    });

    it("ordersForMarket filters by market", () => {
      const orders = state.ordersForMarket("market1");
      expect(orders).toHaveLength(1);

      const noOrders = state.ordersForMarket("market2");
      expect(noOrders).toHaveLength(0);
    });

    it("ordersForOrderbook filters by orderbook", () => {
      const orders = state.ordersForOrderbook("ob1");
      expect(orders).toHaveLength(1);
    });

    it("idleBalanceForOutcome returns idle balance", () => {
      const idle = state.idleBalanceForOutcome("market1", "mint1", 0);
      expect(idle).toBe("0.005000");
    });

    it("onBookBalanceForOutcome returns on-book balance", () => {
      const onBook = state.onBookBalanceForOutcome("market1", "mint1", 0);
      expect(onBook).toBe("0.001000");
    });
  });

  describe("clear", () => {
    it("clears all state", () => {
      state.applySnapshot(createSnapshot());
      state.clear();

      expect(state.hasSnapshot()).toBe(false);
      expect(state.orderCount()).toBe(0);
      expect(state.allBalances()).toHaveLength(0);
      expect(state.lastTimestamp()).toBeUndefined();
    });
  });
});
