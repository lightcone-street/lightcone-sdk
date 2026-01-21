import { describe, it, expect, beforeEach } from "vitest";
import { MessageHandler } from "./handlers";

/** Type assertion helper for tests */
function assertDefined<T>(value: T | undefined | null): asserts value is T {
  expect(value).toBeDefined();
}

describe("MessageHandler", () => {
  let handler: MessageHandler;

  beforeEach(() => {
    handler = new MessageHandler();
  });

  describe("handleMessage", () => {
    it("handles book_update snapshot", () => {
      const msg = JSON.stringify({
        type: "book_update",
        version: 0.1,
        data: {
          orderbook_id: "ob1",
          timestamp: "2024-01-01T00:00:00.000Z",
          seq: 0,
          bids: [{ side: "bid", price: "0.500000", size: "0.001000" }],
          asks: [{ side: "ask", price: "0.510000", size: "0.000500" }],
          is_snapshot: true,
          resync: false,
        },
      });

      const events = handler.handleMessage(msg);

      expect(events).toHaveLength(1);
      expect(events[0].type).toBe("BookUpdate");
      if (events[0].type === "BookUpdate") {
        expect(events[0].orderbookId).toBe("ob1");
        expect(events[0].isSnapshot).toBe(true);
      }
    });

    it("handles book_update resync", () => {
      const msg = JSON.stringify({
        type: "book_update",
        version: 0.1,
        data: {
          orderbook_id: "ob1",
          resync: true,
          message: "Please re-subscribe",
        },
      });

      const events = handler.handleMessage(msg);

      expect(events).toHaveLength(1);
      expect(events[0].type).toBe("ResyncRequired");
      if (events[0].type === "ResyncRequired") {
        expect(events[0].orderbookId).toBe("ob1");
      }
    });

    it("handles trades message", () => {
      const msg = JSON.stringify({
        type: "trades",
        version: 0.1,
        data: {
          orderbook_id: "ob1",
          price: "0.505000",
          size: "0.000250",
          side: "bid",
          timestamp: "2024-01-01T00:00:00.000Z",
          trade_id: "trade123",
        },
      });

      const events = handler.handleMessage(msg);

      expect(events).toHaveLength(1);
      expect(events[0].type).toBe("Trade");
      if (events[0].type === "Trade") {
        expect(events[0].orderbookId).toBe("ob1");
        expect(events[0].trade.price).toBe("0.505000");
        expect(events[0].trade.size).toBe("0.000250");
      }
    });

    it("handles pong message", () => {
      const msg = JSON.stringify({
        type: "pong",
        version: 0.1,
        data: {},
      });

      const events = handler.handleMessage(msg);

      expect(events).toHaveLength(1);
      expect(events[0].type).toBe("Pong");
    });

    it("handles error message", () => {
      const msg = JSON.stringify({
        type: "error",
        version: 0.1,
        data: {
          error: "Engine unavailable",
          code: "ENGINE_UNAVAILABLE",
        },
      });

      const events = handler.handleMessage(msg);

      expect(events).toHaveLength(1);
      expect(events[0].type).toBe("Error");
    });

    it("handles invalid JSON", () => {
      const events = handler.handleMessage("not valid json");

      expect(events).toHaveLength(1);
      expect(events[0].type).toBe("Error");
    });

    it("handles unknown message type", () => {
      const msg = JSON.stringify({
        type: "unknown_type",
        version: 0.1,
        data: {},
      });

      const events = handler.handleMessage(msg);

      expect(events).toHaveLength(0);
    });

    it("handles market event", () => {
      const msg = JSON.stringify({
        type: "market",
        version: 0.1,
        data: {
          event_type: "settled",
          market_pubkey: "market1",
          timestamp: "2024-01-01T00:00:00.000Z",
        },
      });

      const events = handler.handleMessage(msg);

      expect(events).toHaveLength(1);
      expect(events[0].type).toBe("MarketEvent");
      if (events[0].type === "MarketEvent") {
        expect(events[0].eventType).toBe("settled");
        expect(events[0].marketPubkey).toBe("market1");
      }
    });
  });

  describe("state management", () => {
    it("initializes and retrieves orderbook state", () => {
      handler.initOrderbook("ob1");

      const book = handler.getOrderbook("ob1");
      assertDefined(book);
      expect(book.orderbookId).toBe("ob1");
    });

    it("initializes and retrieves user state", () => {
      handler.initUserState("user1");

      const state = handler.getUserState("user1");
      assertDefined(state);
      expect(state.user).toBe("user1");
    });

    it("initializes and retrieves price history state", () => {
      handler.initPriceHistory("ob1", "1m", true);

      const history = handler.getPriceHistory("ob1", "1m");
      assertDefined(history);
      expect(history.orderbookId).toBe("ob1");
      expect(history.resolution).toBe("1m");
    });

    it("clears orderbook state", () => {
      handler.initOrderbook("ob1");

      // Apply a snapshot
      handler.handleMessage(
        JSON.stringify({
          type: "book_update",
          version: 0.1,
          data: {
            orderbook_id: "ob1",
            timestamp: "2024-01-01T00:00:00.000Z",
            seq: 0,
            bids: [{ side: "bid", price: "0.500000", size: "0.001000" }],
            asks: [],
            is_snapshot: true,
            resync: false,
          },
        })
      );

      handler.clearOrderbook("ob1");

      const book = handler.getOrderbook("ob1");
      assertDefined(book);
      expect(book.hasSnapshot()).toBe(false);
    });

    it("clears all state", () => {
      handler.initOrderbook("ob1");
      handler.initUserState("user1");
      handler.initPriceHistory("ob1", "1m", true);

      handler.clearAll();

      expect(handler.getOrderbook("ob1")).toBeUndefined();
      expect(handler.getUserState("user1")).toBeUndefined();
      expect(handler.getPriceHistory("ob1", "1m")).toBeUndefined();
    });
  });

  describe("sequence gap handling", () => {
    it("detects sequence gap and emits ResyncRequired", () => {
      handler.initOrderbook("ob1");

      // Apply snapshot
      handler.handleMessage(
        JSON.stringify({
          type: "book_update",
          version: 0.1,
          data: {
            orderbook_id: "ob1",
            timestamp: "2024-01-01T00:00:00.000Z",
            seq: 0,
            bids: [],
            asks: [],
            is_snapshot: true,
            resync: false,
          },
        })
      );

      // Apply delta with gap
      const events = handler.handleMessage(
        JSON.stringify({
          type: "book_update",
          version: 0.1,
          data: {
            orderbook_id: "ob1",
            timestamp: "2024-01-01T00:00:00.050Z",
            seq: 5, // Gap - expected 1
            bids: [],
            asks: [],
            is_snapshot: false,
            resync: false,
          },
        })
      );

      expect(events).toHaveLength(1);
      expect(events[0].type).toBe("ResyncRequired");
    });
  });
});
