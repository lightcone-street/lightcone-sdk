import { describe, it, expect, beforeEach } from "vitest";
import { LocalOrderbook } from "./orderbook";
import { WebSocketError } from "../error";
import type { BookUpdateData } from "../types";

describe("LocalOrderbook", () => {
  let book: LocalOrderbook;

  function createSnapshot(): BookUpdateData {
    return {
      orderbook_id: "test",
      timestamp: "2024-01-01T00:00:00.000Z",
      seq: 0,
      bids: [
        { side: "bid", price: "0.500000", size: "0.001000" },
        { side: "bid", price: "0.490000", size: "0.002000" },
      ],
      asks: [
        { side: "ask", price: "0.510000", size: "0.000500" },
        { side: "ask", price: "0.520000", size: "0.001500" },
      ],
      is_snapshot: true,
      resync: false,
    };
  }

  beforeEach(() => {
    book = new LocalOrderbook("test");
  });

  describe("applySnapshot", () => {
    it("applies a snapshot correctly", () => {
      const snapshot = createSnapshot();
      book.applySnapshot(snapshot);

      expect(book.hasSnapshot()).toBe(true);
      expect(book.expectedSequence()).toBe(1);
      expect(book.bidCount()).toBe(2);
      expect(book.askCount()).toBe(2);
    });

    it("sets best bid and ask", () => {
      book.applySnapshot(createSnapshot());

      expect(book.bestBid()).toEqual(["0.500000", "0.001000"]);
      expect(book.bestAsk()).toEqual(["0.510000", "0.000500"]);
    });

    it("clears existing state on new snapshot", () => {
      book.applySnapshot(createSnapshot());

      const newSnapshot: BookUpdateData = {
        orderbook_id: "test",
        timestamp: "2024-01-01T00:00:01.000Z",
        seq: 10,
        bids: [{ side: "bid", price: "0.400000", size: "0.003000" }],
        asks: [{ side: "ask", price: "0.600000", size: "0.004000" }],
        is_snapshot: true,
        resync: false,
      };
      book.applySnapshot(newSnapshot);

      expect(book.bidCount()).toBe(1);
      expect(book.askCount()).toBe(1);
      expect(book.expectedSequence()).toBe(11);
    });

    it("ignores zero-size levels", () => {
      const snapshot: BookUpdateData = {
        orderbook_id: "test",
        timestamp: "2024-01-01T00:00:00.000Z",
        seq: 0,
        bids: [
          { side: "bid", price: "0.500000", size: "0.001000" },
          { side: "bid", price: "0.490000", size: "0" },
        ],
        asks: [],
        is_snapshot: true,
        resync: false,
      };
      book.applySnapshot(snapshot);

      expect(book.bidCount()).toBe(1);
    });
  });

  describe("applyDelta", () => {
    it("applies a delta update", () => {
      book.applySnapshot(createSnapshot());

      const delta: BookUpdateData = {
        orderbook_id: "test",
        timestamp: "2024-01-01T00:00:00.050Z",
        seq: 1,
        bids: [{ side: "bid", price: "0.500000", size: "0.001500" }],
        asks: [],
        is_snapshot: false,
        resync: false,
      };
      book.applyDelta(delta);

      expect(book.bestBid()).toEqual(["0.500000", "0.001500"]);
      expect(book.expectedSequence()).toBe(2);
    });

    it("removes levels with zero size", () => {
      book.applySnapshot(createSnapshot());

      const delta: BookUpdateData = {
        orderbook_id: "test",
        timestamp: "2024-01-01T00:00:00.050Z",
        seq: 1,
        bids: [],
        asks: [{ side: "ask", price: "0.510000", size: "0" }],
        is_snapshot: false,
        resync: false,
      };
      book.applyDelta(delta);

      expect(book.askCount()).toBe(1);
      expect(book.bestAsk()).toEqual(["0.520000", "0.001500"]);
    });

    it("throws on sequence gap", () => {
      book.applySnapshot(createSnapshot());

      const delta: BookUpdateData = {
        orderbook_id: "test",
        timestamp: "2024-01-01T00:00:00.050Z",
        seq: 5, // Gap!
        bids: [],
        asks: [],
        is_snapshot: false,
        resync: false,
      };

      expect(() => book.applyDelta(delta)).toThrow(WebSocketError);
    });
  });

  describe("applyUpdate", () => {
    it("routes to applySnapshot for snapshots", () => {
      const snapshot = createSnapshot();
      book.applyUpdate(snapshot);

      expect(book.hasSnapshot()).toBe(true);
    });

    it("routes to applyDelta for deltas", () => {
      book.applySnapshot(createSnapshot());

      const delta: BookUpdateData = {
        orderbook_id: "test",
        timestamp: "2024-01-01T00:00:00.050Z",
        seq: 1,
        bids: [{ side: "bid", price: "0.500000", size: "0.002000" }],
        asks: [],
        is_snapshot: false,
        resync: false,
      };
      book.applyUpdate(delta);

      expect(book.bestBid()).toEqual(["0.500000", "0.002000"]);
    });
  });

  describe("getters", () => {
    beforeEach(() => {
      book.applySnapshot(createSnapshot());
    });

    it("getBids returns sorted bids (descending)", () => {
      const bids = book.getBids();
      expect(bids[0].price).toBe("0.500000");
      expect(bids[1].price).toBe("0.490000");
    });

    it("getAsks returns sorted asks (ascending)", () => {
      const asks = book.getAsks();
      expect(asks[0].price).toBe("0.510000");
      expect(asks[1].price).toBe("0.520000");
    });

    it("getTopBids limits results", () => {
      const bids = book.getTopBids(1);
      expect(bids).toHaveLength(1);
      expect(bids[0].price).toBe("0.500000");
    });

    it("getTopAsks limits results", () => {
      const asks = book.getTopAsks(1);
      expect(asks).toHaveLength(1);
      expect(asks[0].price).toBe("0.510000");
    });
  });

  describe("calculations", () => {
    beforeEach(() => {
      book.applySnapshot(createSnapshot());
    });

    it("calculates spread", () => {
      expect(book.spread()).toBe("0.010000");
    });

    it("calculates midpoint", () => {
      expect(book.midpoint()).toBe("0.505000");
    });

    it("calculates total bid depth", () => {
      expect(book.totalBidDepth()).toBeCloseTo(0.003);
    });

    it("calculates total ask depth", () => {
      expect(book.totalAskDepth()).toBeCloseTo(0.002);
    });
  });

  describe("clear", () => {
    it("clears all state", () => {
      book.applySnapshot(createSnapshot());
      book.clear();

      expect(book.hasSnapshot()).toBe(false);
      expect(book.bidCount()).toBe(0);
      expect(book.askCount()).toBe(0);
      expect(book.expectedSequence()).toBe(0);
      expect(book.lastTimestamp()).toBeUndefined();
    });
  });
});
