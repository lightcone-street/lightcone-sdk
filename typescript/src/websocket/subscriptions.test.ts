import { describe, it, expect, beforeEach } from "vitest";
import { SubscriptionManager, subscriptionToParams, subscriptionType } from "./subscriptions";
import type { BookUpdateParams, UserParams, PriceHistoryParams } from "./types";

describe("SubscriptionManager", () => {
  let manager: SubscriptionManager;

  beforeEach(() => {
    manager = new SubscriptionManager();
  });

  describe("book updates", () => {
    it("adds book update subscriptions", () => {
      manager.addBookUpdate(["ob1", "ob2"]);

      expect(manager.isSubscribedBookUpdate("ob1")).toBe(true);
      expect(manager.isSubscribedBookUpdate("ob2")).toBe(true);
      expect(manager.isSubscribedBookUpdate("ob3")).toBe(false);
    });

    it("removes book update subscriptions", () => {
      manager.addBookUpdate(["ob1", "ob2"]);
      manager.removeBookUpdate(["ob1"]);

      expect(manager.isSubscribedBookUpdate("ob1")).toBe(false);
      expect(manager.isSubscribedBookUpdate("ob2")).toBe(true);
    });

    it("returns subscribed orderbooks", () => {
      manager.addBookUpdate(["ob1", "ob2"]);
      const orderbooks = manager.bookUpdateOrderbooks();

      expect(orderbooks).toContain("ob1");
      expect(orderbooks).toContain("ob2");
    });
  });

  describe("trades", () => {
    it("adds trades subscriptions", () => {
      manager.addTrades(["ob1"]);

      expect(manager.isSubscribedTrades("ob1")).toBe(true);
      expect(manager.isSubscribedTrades("ob2")).toBe(false);
    });

    it("removes trades subscriptions", () => {
      manager.addTrades(["ob1", "ob2"]);
      manager.removeTrades(["ob1"]);

      expect(manager.isSubscribedTrades("ob1")).toBe(false);
      expect(manager.isSubscribedTrades("ob2")).toBe(true);
    });
  });

  describe("users", () => {
    it("adds user subscriptions", () => {
      manager.addUser("user1");

      expect(manager.isSubscribedUser("user1")).toBe(true);
      expect(manager.isSubscribedUser("user2")).toBe(false);
    });

    it("removes user subscriptions", () => {
      manager.addUser("user1");
      manager.removeUser("user1");

      expect(manager.isSubscribedUser("user1")).toBe(false);
    });
  });

  describe("price history", () => {
    it("adds price history subscriptions", () => {
      manager.addPriceHistory("ob1", "1m", true);

      expect(manager.isSubscribedPriceHistory("ob1", "1m")).toBe(true);
      expect(manager.isSubscribedPriceHistory("ob1", "5m")).toBe(false);
    });

    it("removes price history subscriptions", () => {
      manager.addPriceHistory("ob1", "1m", true);
      manager.removePriceHistory("ob1", "1m");

      expect(manager.isSubscribedPriceHistory("ob1", "1m")).toBe(false);
    });
  });

  describe("markets", () => {
    it("adds market subscriptions", () => {
      manager.addMarket("market1");

      expect(manager.isSubscribedMarket("market1")).toBe(true);
    });

    it("removes market subscriptions", () => {
      manager.addMarket("market1");
      manager.removeMarket("market1");

      expect(manager.isSubscribedMarket("market1")).toBe(false);
    });

    it("all markets subscription matches any market", () => {
      manager.addMarket("all");

      expect(manager.isSubscribedMarket("any_market")).toBe(true);
      expect(manager.isSubscribedMarket("another_market")).toBe(true);
    });
  });

  describe("getAllSubscriptions", () => {
    it("returns all subscriptions", () => {
      manager.addBookUpdate(["ob1"]);
      manager.addTrades(["ob1"]);
      manager.addUser("user1");
      manager.addPriceHistory("ob1", "1m", true);
      manager.addMarket("market1");

      const subs = manager.getAllSubscriptions();

      expect(subs).toHaveLength(5);

      const types = subs.map((s) => s.type);
      expect(types).toContain("BookUpdate");
      expect(types).toContain("Trades");
      expect(types).toContain("User");
      expect(types).toContain("PriceHistory");
      expect(types).toContain("Market");
    });

    it("groups orderbook subscriptions", () => {
      manager.addBookUpdate(["ob1", "ob2", "ob3"]);

      const subs = manager.getAllSubscriptions();
      const bookUpdate = subs.find((s) => s.type === "BookUpdate");

      expect(bookUpdate).toBeDefined();
      if (bookUpdate && bookUpdate.type === "BookUpdate") {
        expect(bookUpdate.orderbookIds).toHaveLength(3);
      }
    });
  });

  describe("subscription counts", () => {
    it("hasSubscriptions returns false when empty", () => {
      expect(manager.hasSubscriptions()).toBe(false);
    });

    it("hasSubscriptions returns true when has subscriptions", () => {
      manager.addBookUpdate(["ob1"]);
      expect(manager.hasSubscriptions()).toBe(true);
    });

    it("subscriptionCount returns total count", () => {
      expect(manager.subscriptionCount()).toBe(0);

      manager.addBookUpdate(["ob1", "ob2"]);
      manager.addUser("user1");
      manager.addPriceHistory("ob1", "1m", true);

      expect(manager.subscriptionCount()).toBe(4);
    });
  });

  describe("clear", () => {
    it("clears all subscriptions", () => {
      manager.addBookUpdate(["ob1"]);
      manager.addUser("user1");
      manager.addPriceHistory("ob1", "1m", true);

      manager.clear();

      expect(manager.hasSubscriptions()).toBe(false);
      expect(manager.subscriptionCount()).toBe(0);
    });
  });
});

describe("subscriptionToParams", () => {
  it("converts BookUpdate subscription", () => {
    const params = subscriptionToParams({
      type: "BookUpdate",
      orderbookIds: ["ob1", "ob2"],
    });

    expect(params.type).toBe("book_update");
    const bookParams = params as BookUpdateParams;
    expect(bookParams.orderbook_ids).toEqual(["ob1", "ob2"]);
  });

  it("converts User subscription", () => {
    const params = subscriptionToParams({
      type: "User",
      user: "user123",
    });

    expect(params.type).toBe("user");
    const userParams = params as UserParams;
    expect(userParams.user).toBe("user123");
  });

  it("converts PriceHistory subscription", () => {
    const params = subscriptionToParams({
      type: "PriceHistory",
      orderbookId: "ob1",
      resolution: "1m",
      includeOhlcv: true,
    });

    expect(params.type).toBe("price_history");
    const priceParams = params as PriceHistoryParams;
    expect(priceParams.orderbook_id).toBe("ob1");
    expect(priceParams.resolution).toBe("1m");
    expect(priceParams.include_ohlcv).toBe(true);
  });
});

describe("subscriptionType", () => {
  it("returns lowercase type string", () => {
    expect(subscriptionType({ type: "BookUpdate", orderbookIds: [] })).toBe("bookupdate");
    expect(subscriptionType({ type: "User", user: "test" })).toBe("user");
    expect(subscriptionType({ type: "Market", marketPubkey: "test" })).toBe("market");
  });
});
