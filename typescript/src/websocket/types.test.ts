import { describe, it, expect } from "vitest";
import {
  createSubscribeRequest,
  createUnsubscribeRequest,
  createPingRequest,
  bookUpdateParams,
  tradesParams,
  userParams,
  priceHistoryParams,
  marketParams,
  toCandle,
  parseMessageType,
  parseMarketEventType,
  parseErrorCode,
  parseSide,
  sideToNumber,
  parsePriceLevelSide,
} from "./types";

/** Type assertion helper for tests */
function assertDefined<T>(value: T | undefined | null): asserts value is T {
  expect(value).toBeDefined();
}

describe("WebSocket types", () => {
  describe("request helpers", () => {
    it("creates subscribe request", () => {
      const params = bookUpdateParams(["ob1"]);
      const request = createSubscribeRequest(params);
      expect(request.method).toBe("subscribe");
      expect(request.params).toEqual({
        type: "book_update",
        orderbook_ids: ["ob1"],
      });
    });

    it("creates unsubscribe request", () => {
      const params = bookUpdateParams(["ob1"]);
      const request = createUnsubscribeRequest(params);
      expect(request.method).toBe("unsubscribe");
    });

    it("creates ping request", () => {
      const request = createPingRequest();
      expect(request.method).toBe("ping");
      expect(request.params).toBeUndefined();
    });
  });

  describe("params helpers", () => {
    it("creates bookUpdateParams", () => {
      const params = bookUpdateParams(["ob1", "ob2"]);
      expect(params.type).toBe("book_update");
      expect(params.orderbook_ids).toEqual(["ob1", "ob2"]);
    });

    it("creates tradesParams", () => {
      const params = tradesParams(["ob1"]);
      expect(params.type).toBe("trades");
      expect(params.orderbook_ids).toEqual(["ob1"]);
    });

    it("creates userParams", () => {
      const params = userParams("user123");
      expect(params.type).toBe("user");
      expect(params.user).toBe("user123");
    });

    it("creates priceHistoryParams", () => {
      const params = priceHistoryParams("ob1", "1m", true);
      expect(params.type).toBe("price_history");
      expect(params.orderbook_id).toBe("ob1");
      expect(params.resolution).toBe("1m");
      expect(params.include_ohlcv).toBe(true);
    });

    it("creates marketParams", () => {
      const params = marketParams("market1");
      expect(params.type).toBe("market");
      expect(params.market_pubkey).toBe("market1");
    });
  });

  describe("toCandle", () => {
    it("converts inline candle data", () => {
      const data = {
        event_type: "update",
        prices: [],
        t: 1704067200000,
        o: "0.500000",
        h: "0.510000",
        l: "0.490000",
        c: "0.505000",
        v: "100.000000",
        m: "0.502500",
        bb: "0.500000",
        ba: "0.505000",
      };

      const candle = toCandle(data);
      assertDefined(candle);
      expect(candle.t).toBe(1704067200000);
      expect(candle.o).toBe("0.500000");
      expect(candle.h).toBe("0.510000");
      expect(candle.l).toBe("0.490000");
      expect(candle.c).toBe("0.505000");
      expect(candle.m).toBe("0.502500");
    });

    it("returns undefined when t is missing", () => {
      const data = {
        event_type: "heartbeat",
        prices: [],
      };

      const candle = toCandle(data);
      expect(candle).toBeUndefined();
    });
  });

  describe("parseMessageType", () => {
    it("parses known types", () => {
      expect(parseMessageType("book_update")).toBe("BookUpdate");
      expect(parseMessageType("trades")).toBe("Trades");
      expect(parseMessageType("user")).toBe("User");
      expect(parseMessageType("price_history")).toBe("PriceHistory");
      expect(parseMessageType("market")).toBe("Market");
      expect(parseMessageType("error")).toBe("Error");
      expect(parseMessageType("pong")).toBe("Pong");
    });

    it("returns Unknown for unrecognized types", () => {
      expect(parseMessageType("unknown_type")).toBe("Unknown");
      expect(parseMessageType("")).toBe("Unknown");
    });
  });

  describe("parseMarketEventType", () => {
    it("parses known types", () => {
      expect(parseMarketEventType("orderbook_created")).toBe("OrderbookCreated");
      expect(parseMarketEventType("settled")).toBe("Settled");
      expect(parseMarketEventType("opened")).toBe("Opened");
      expect(parseMarketEventType("paused")).toBe("Paused");
    });

    it("returns Unknown for unrecognized types", () => {
      expect(parseMarketEventType("unknown")).toBe("Unknown");
    });
  });

  describe("parseErrorCode", () => {
    it("parses known codes", () => {
      expect(parseErrorCode("ENGINE_UNAVAILABLE")).toBe("EngineUnavailable");
      expect(parseErrorCode("INVALID_JSON")).toBe("InvalidJson");
      expect(parseErrorCode("INVALID_METHOD")).toBe("InvalidMethod");
      expect(parseErrorCode("RATE_LIMITED")).toBe("RateLimited");
    });

    it("returns Unknown for unrecognized codes", () => {
      expect(parseErrorCode("UNKNOWN_CODE")).toBe("Unknown");
    });
  });

  describe("side helpers", () => {
    it("parseSide converts numbers to Side", () => {
      expect(parseSide(0)).toBe("Buy");
      expect(parseSide(1)).toBe("Sell");
      expect(parseSide(99)).toBe("Sell"); // Defaults to Sell
    });

    it("sideToNumber converts Side to number", () => {
      expect(sideToNumber("Buy")).toBe(0);
      expect(sideToNumber("Sell")).toBe(1);
    });

    it("parsePriceLevelSide converts strings", () => {
      expect(parsePriceLevelSide("bid")).toBe("Bid");
      expect(parsePriceLevelSide("ask")).toBe("Ask");
      expect(parsePriceLevelSide("other")).toBe("Ask"); // Defaults to Ask
    });
  });
});
