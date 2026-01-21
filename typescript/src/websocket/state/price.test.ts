import { describe, it, expect, beforeEach } from "vitest";
import { PriceHistory, PriceHistoryKey } from "./price";
import { Resolution } from "../../shared";
import type { PriceHistoryData } from "../types";

/** Type assertion helper for tests */
function assertDefined<T>(value: T | undefined | null): asserts value is T {
  expect(value).toBeDefined();
}

describe("PriceHistoryKey", () => {
  it("creates a key", () => {
    const key = new PriceHistoryKey("ob1", "1m");
    expect(key.orderbookId).toBe("ob1");
    expect(key.resolution).toBe("1m");
  });

  it("converts to string", () => {
    const key = new PriceHistoryKey("ob1", "1m");
    expect(key.toString()).toBe("ob1:1m");
  });

  it("compares equality", () => {
    const key1 = new PriceHistoryKey("ob1", "1m");
    const key2 = new PriceHistoryKey("ob1", "1m");
    const key3 = new PriceHistoryKey("ob1", "5m");

    expect(key1.equals(key2)).toBe(true);
    expect(key1.equals(key3)).toBe(false);
  });
});

describe("PriceHistory", () => {
  let history: PriceHistory;

  function createSnapshot(): PriceHistoryData {
    return {
      event_type: "snapshot",
      orderbook_id: "ob1",
      resolution: "1m",
      include_ohlcv: true,
      prices: [
        {
          t: 1704067260000, // Newer
          o: "0.505000",
          h: "0.508000",
          l: "0.503000",
          c: "0.507000",
          v: "0.005000",
          m: "0.505500",
          bb: "0.505000",
          ba: "0.506000",
        },
        {
          t: 1704067200000, // Older
          o: "0.500000",
          h: "0.510000",
          l: "0.495000",
          c: "0.505000",
          v: "0.010000",
          m: "0.502500",
          bb: "0.500000",
          ba: "0.505000",
        },
      ],
      last_timestamp: 1704067260000,
      server_time: 1704067320000,
    };
  }

  beforeEach(() => {
    history = new PriceHistory("ob1", "1m", true);
  });

  describe("applySnapshot", () => {
    it("applies a snapshot correctly", () => {
      history.applySnapshot(createSnapshot());

      expect(history.hasSnapshot()).toBe(true);
      expect(history.candleCount()).toBe(2);
      expect(history.lastTimestamp()).toBe(1704067260000);
      expect(history.serverTime()).toBe(1704067320000);
    });

    it("maintains candle order (newest first)", () => {
      history.applySnapshot(createSnapshot());

      const candles = history.candles();
      expect(candles[0].t).toBe(1704067260000);
      expect(candles[1].t).toBe(1704067200000);
    });

    it("clears previous state", () => {
      history.applySnapshot(createSnapshot());

      const newSnapshot: PriceHistoryData = {
        event_type: "snapshot",
        orderbook_id: "ob1",
        resolution: "1m",
        prices: [{ t: 1704067320000, m: "0.510000" }],
        last_timestamp: 1704067320000,
        server_time: 1704067380000,
      };
      history.applySnapshot(newSnapshot);

      expect(history.candleCount()).toBe(1);
    });
  });

  describe("applyUpdate", () => {
    it("updates existing candle", () => {
      history.applySnapshot(createSnapshot());

      const update: PriceHistoryData = {
        event_type: "update",
        prices: [],
        t: 1704067260000,
        o: "0.505000",
        h: "0.510000", // Updated high
        l: "0.503000",
        c: "0.509000", // Updated close
        v: "0.006000", // Updated volume
        m: "0.507000",
        bb: "0.506000",
        ba: "0.508000",
      };
      history.applyUpdate(update);

      const candle = history.getCandle(1704067260000);
      assertDefined(candle);
      expect(candle.h).toBe("0.510000");
      expect(candle.c).toBe("0.509000");
      expect(candle.v).toBe("0.006000");
    });

    it("adds new candle", () => {
      history.applySnapshot(createSnapshot());

      const update: PriceHistoryData = {
        event_type: "update",
        prices: [],
        t: 1704067320000, // New timestamp
        o: "0.507000",
        h: "0.512000",
        l: "0.506000",
        c: "0.511000",
        v: "0.008000",
        m: "0.509000",
        bb: "0.508000",
        ba: "0.510000",
      };
      history.applyUpdate(update);

      expect(history.candleCount()).toBe(3);
      const latestCandle = history.latestCandle();
      assertDefined(latestCandle);
      expect(latestCandle.t).toBe(1704067320000);
    });
  });

  describe("applyHeartbeat", () => {
    it("updates server time", () => {
      history.applySnapshot(createSnapshot());

      const heartbeat: PriceHistoryData = {
        event_type: "heartbeat",
        prices: [],
        server_time: 1704067400000,
      };
      history.applyHeartbeat(heartbeat);

      expect(history.serverTime()).toBe(1704067400000);
    });
  });

  describe("applyEvent", () => {
    it("routes events correctly", () => {
      history.applyEvent(createSnapshot());
      expect(history.hasSnapshot()).toBe(true);

      const update: PriceHistoryData = {
        event_type: "update",
        prices: [],
        t: 1704067320000,
        m: "0.510000",
      };
      history.applyEvent(update);
      expect(history.candleCount()).toBe(3);

      const heartbeat: PriceHistoryData = {
        event_type: "heartbeat",
        prices: [],
        server_time: 1704067500000,
      };
      history.applyEvent(heartbeat);
      expect(history.serverTime()).toBe(1704067500000);
    });
  });

  describe("query methods", () => {
    beforeEach(() => {
      history.applySnapshot(createSnapshot());
    });

    it("candles returns all candles", () => {
      const candles = history.candles();
      expect(candles).toHaveLength(2);
    });

    it("recentCandles limits results", () => {
      const candles = history.recentCandles(1);
      expect(candles).toHaveLength(1);
      expect(candles[0].t).toBe(1704067260000);
    });

    it("getCandle returns candle by timestamp", () => {
      const candle = history.getCandle(1704067200000);
      assertDefined(candle);
      expect(candle.o).toBe("0.500000");
    });

    it("latestCandle returns newest candle", () => {
      const latest = history.latestCandle();
      assertDefined(latest);
      expect(latest.t).toBe(1704067260000);
    });

    it("oldestCandle returns oldest candle", () => {
      const oldest = history.oldestCandle();
      assertDefined(oldest);
      expect(oldest.t).toBe(1704067200000);
    });

    it("currentMidpoint returns latest midpoint", () => {
      expect(history.currentMidpoint()).toBe("0.505500");
    });

    it("currentBestBid returns latest best bid", () => {
      expect(history.currentBestBid()).toBe("0.505000");
    });

    it("currentBestAsk returns latest best ask", () => {
      expect(history.currentBestAsk()).toBe("0.506000");
    });
  });

  describe("resolutionEnum", () => {
    it("returns correct enum value", () => {
      expect(history.resolutionEnum()).toBe(Resolution.OneMinute);

      const h5m = new PriceHistory("ob1", "5m", true);
      expect(h5m.resolutionEnum()).toBe(Resolution.FiveMinutes);

      const h1h = new PriceHistory("ob1", "1h", true);
      expect(h1h.resolutionEnum()).toBe(Resolution.OneHour);
    });

    it("returns undefined for unknown resolution", () => {
      const h = new PriceHistory("ob1", "2m", true);
      expect(h.resolutionEnum()).toBeUndefined();
    });
  });

  describe("clear", () => {
    it("clears all state", () => {
      history.applySnapshot(createSnapshot());
      history.clear();

      expect(history.hasSnapshot()).toBe(false);
      expect(history.candleCount()).toBe(0);
      expect(history.lastTimestamp()).toBeUndefined();
      expect(history.serverTime()).toBeUndefined();
    });
  });
});
