import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { LightconeApiClient, DEFAULT_API_URL } from "./client";
import { ApiError } from "./error";
import { Resolution } from "../shared";

describe("LightconeApiClient", () => {
  let client: LightconeApiClient;
  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    client = new LightconeApiClient();
    fetchMock = vi.fn();
    global.fetch = fetchMock as unknown as typeof fetch;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("constructor", () => {
    it("uses default config", () => {
      const c = new LightconeApiClient();
      expect(c).toBeDefined();
    });

    it("accepts custom baseUrl", () => {
      const c = new LightconeApiClient({ baseUrl: "https://custom.api" });
      expect(c).toBeDefined();
    });

    it("accepts custom timeout", () => {
      const c = new LightconeApiClient({ timeout: 5000 });
      expect(c).toBeDefined();
    });

    it("accepts custom headers", () => {
      const c = new LightconeApiClient({
        headers: { "X-Custom": "value" },
      });
      expect(c).toBeDefined();
    });
  });

  describe("healthCheck", () => {
    it("calls health endpoint", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ status: "ok" }),
      });

      await client.healthCheck();

      expect(fetchMock).toHaveBeenCalledWith(
        `${DEFAULT_API_URL}/health`,
        expect.objectContaining({
          method: "GET",
        })
      );
    });

    it("throws on non-ok response", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: async () => ({ error: "server error" }),
      });

      await expect(client.healthCheck()).rejects.toThrow(ApiError);
    });
  });

  describe("getMarkets", () => {
    it("fetches markets", async () => {
      const mockResponse = {
        markets: [{ market_pubkey: "abc123" }],
        total: 1,
      };
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await client.getMarkets();

      expect(result.markets).toHaveLength(1);
      expect(result.total).toBe(1);
    });
  });

  describe("getMarket", () => {
    it("fetches market by pubkey", async () => {
      const mockResponse = {
        market: { market_pubkey: "abc123" },
        deposit_assets: [],
        deposit_asset_count: 0,
      };
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await client.getMarket("abc123");

      expect(fetchMock).toHaveBeenCalledWith(
        `${DEFAULT_API_URL}/markets/abc123`,
        expect.any(Object)
      );
      expect(result.market.market_pubkey).toBe("abc123");
    });
  });

  describe("getOrderbook", () => {
    it("fetches orderbook without depth", async () => {
      const mockResponse = {
        market_pubkey: "market1",
        orderbook_id: "ob1",
        bids: [],
        asks: [],
        tick_size: "0.001",
      };
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      await client.getOrderbook("ob1");

      expect(fetchMock).toHaveBeenCalledWith(
        `${DEFAULT_API_URL}/orderbook/ob1`,
        expect.any(Object)
      );
    });

    it("fetches orderbook with depth", async () => {
      const mockResponse = {
        market_pubkey: "market1",
        orderbook_id: "ob1",
        bids: [],
        asks: [],
        tick_size: "0.001",
      };
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      await client.getOrderbook("ob1", 10);

      expect(fetchMock).toHaveBeenCalledWith(
        `${DEFAULT_API_URL}/orderbook/ob1?depth=10`,
        expect.any(Object)
      );
    });
  });

  describe("submitOrder", () => {
    it("submits order via POST", async () => {
      const orderRequest = {
        maker: "maker123",
        nonce: 1,
        market_pubkey: "market1",
        base_token: "base1",
        quote_token: "quote1",
        side: 0,
        maker_amount: 1000,
        taker_amount: 500,
        signature: "sig".repeat(64),
        orderbook_id: "ob1",
      };
      const mockResponse = {
        order_hash: "hash123",
        status: "accepted",
        remaining: "1000",
        filled: "0",
        fills: [],
      };
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await client.submitOrder(orderRequest);

      expect(fetchMock).toHaveBeenCalledWith(
        `${DEFAULT_API_URL}/orders/submit`,
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify(orderRequest),
        })
      );
      expect(result.order_hash).toBe("hash123");
    });
  });

  describe("cancelOrder", () => {
    it("cancels order via POST", async () => {
      const mockResponse = {
        status: "cancelled",
        order_hash: "hash123",
        remaining: "1000",
      };
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await client.cancelOrder("hash123", "maker123");

      expect(fetchMock).toHaveBeenCalledWith(
        `${DEFAULT_API_URL}/orders/cancel`,
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({
            order_hash: "hash123",
            maker: "maker123",
          }),
        })
      );
      expect(result.status).toBe("cancelled");
    });
  });

  describe("getPriceHistory", () => {
    it("builds query params correctly", async () => {
      const mockResponse = {
        orderbook_id: "ob1",
        resolution: "1m",
        include_ohlcv: true,
        prices: [],
        has_more: false,
      };
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      await client.getPriceHistory({
        orderbook_id: "ob1",
        resolution: Resolution.OneMinute,
        include_ohlcv: true,
        limit: 100,
      });

      const calledUrl = fetchMock.mock.calls[0][0] as string;
      expect(calledUrl).toContain("orderbook_id=ob1");
      expect(calledUrl).toContain("resolution=1m");
      expect(calledUrl).toContain("include_ohlcv=true");
      expect(calledUrl).toContain("limit=100");
    });
  });

  describe("error handling", () => {
    it("handles 404 errors", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: async () => ({ message: "not found" }),
      });

      await expect(client.getMarket("nonexistent")).rejects.toMatchObject({
        variant: "NotFound",
      });
    });

    it("handles network errors", async () => {
      fetchMock.mockRejectedValueOnce(new Error("Network error"));

      await expect(client.healthCheck()).rejects.toThrow(ApiError);
    });

    it("handles timeout", async () => {
      fetchMock.mockImplementationOnce(
        () =>
          new Promise((_, reject) => {
            const err = new Error("aborted");
            err.name = "AbortError";
            reject(err);
          })
      );

      await expect(client.healthCheck()).rejects.toMatchObject({
        variant: "Http",
        message: expect.stringContaining("timeout"),
      });
    });
  });
});
