import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { LightconeApiClient, DEFAULT_API_URL } from "./client";
import { ApiError } from "./error";
import { Resolution } from "../shared";

// Valid Solana pubkeys for testing
const TEST_PUBKEY = "11111111111111111111111111111111"; // System Program
const TEST_PUBKEY_2 = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; // Token Program
const TEST_PUBKEY_3 = "So11111111111111111111111111111111111111112"; // SOL mint
const TEST_PUBKEY_4 = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC
// Valid 128-char hex signature (64 bytes)
const TEST_SIGNATURE = "a".repeat(128);

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
        market: { market_pubkey: TEST_PUBKEY },
        deposit_assets: [],
        deposit_asset_count: 0,
      };
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => mockResponse,
      });

      const result = await client.getMarket(TEST_PUBKEY);

      expect(fetchMock).toHaveBeenCalledWith(
        `${DEFAULT_API_URL}/markets/${TEST_PUBKEY}`,
        expect.any(Object)
      );
      expect(result.market.market_pubkey).toBe(TEST_PUBKEY);
    });

    it("throws for invalid pubkey", async () => {
      await expect(client.getMarket("invalid")).rejects.toMatchObject({
        variant: "InvalidParameter",
      });
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
        maker: TEST_PUBKEY,
        nonce: 1,
        market_pubkey: TEST_PUBKEY_2,
        base_token: TEST_PUBKEY_3,
        quote_token: TEST_PUBKEY_4,
        side: 0,
        maker_amount: 1000,
        taker_amount: 500,
        signature: TEST_SIGNATURE,
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

    it("throws for invalid maker pubkey", async () => {
      const orderRequest = {
        maker: "invalid",
        nonce: 1,
        market_pubkey: TEST_PUBKEY_2,
        base_token: TEST_PUBKEY_3,
        quote_token: TEST_PUBKEY_4,
        side: 0,
        maker_amount: 1000,
        taker_amount: 500,
        signature: TEST_SIGNATURE,
        orderbook_id: "ob1",
      };
      await expect(client.submitOrder(orderRequest)).rejects.toMatchObject({
        variant: "InvalidParameter",
      });
    });

    it("throws for invalid signature", async () => {
      const orderRequest = {
        maker: TEST_PUBKEY,
        nonce: 1,
        market_pubkey: TEST_PUBKEY_2,
        base_token: TEST_PUBKEY_3,
        quote_token: TEST_PUBKEY_4,
        side: 0,
        maker_amount: 1000,
        taker_amount: 500,
        signature: "tooshort",
        orderbook_id: "ob1",
      };
      await expect(client.submitOrder(orderRequest)).rejects.toMatchObject({
        variant: "InvalidParameter",
      });
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

      const result = await client.cancelOrder("hash123", TEST_PUBKEY);

      expect(fetchMock).toHaveBeenCalledWith(
        `${DEFAULT_API_URL}/orders/cancel`,
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({
            order_hash: "hash123",
            maker: TEST_PUBKEY,
          }),
        })
      );
      expect(result.status).toBe("cancelled");
    });

    it("throws for invalid maker pubkey", async () => {
      await expect(client.cancelOrder("hash123", "invalid")).rejects.toMatchObject({
        variant: "InvalidParameter",
      });
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

      await expect(client.getMarket(TEST_PUBKEY)).rejects.toMatchObject({
        variant: "NotFound",
      });
    });

    it("handles 401 errors", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: false,
        status: 401,
        json: async () => ({ message: "unauthorized" }),
      });

      await expect(client.healthCheck()).rejects.toMatchObject({
        variant: "Unauthorized",
      });
    });

    it("handles 429 errors", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: false,
        status: 429,
        json: async () => ({ message: "too many requests" }),
      });

      await expect(client.healthCheck()).rejects.toMatchObject({
        variant: "RateLimited",
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

  describe("retry logic", () => {
    it("retries on server error", async () => {
      const clientWithRetry = new LightconeApiClient({
        retry: { maxRetries: 2, baseDelayMs: 10, maxDelayMs: 100 },
      });
      global.fetch = fetchMock as unknown as typeof fetch;

      // First call fails with 500, second succeeds
      fetchMock
        .mockResolvedValueOnce({
          ok: false,
          status: 500,
          json: async () => ({ error: "server error" }),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ status: "ok" }),
        });

      await clientWithRetry.healthCheck();

      expect(fetchMock).toHaveBeenCalledTimes(2);
    });

    it("retries on rate limit", async () => {
      const clientWithRetry = new LightconeApiClient({
        retry: { maxRetries: 2, baseDelayMs: 10, maxDelayMs: 100 },
      });
      global.fetch = fetchMock as unknown as typeof fetch;

      // First call fails with 429, second succeeds
      fetchMock
        .mockResolvedValueOnce({
          ok: false,
          status: 429,
          json: async () => ({ error: "rate limited" }),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ status: "ok" }),
        });

      await clientWithRetry.healthCheck();

      expect(fetchMock).toHaveBeenCalledTimes(2);
    });

    it("does not retry on 400 bad request", async () => {
      const clientWithRetry = new LightconeApiClient({
        retry: { maxRetries: 2, baseDelayMs: 10, maxDelayMs: 100 },
      });
      global.fetch = fetchMock as unknown as typeof fetch;

      fetchMock.mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: async () => ({ error: "bad request" }),
      });

      await expect(clientWithRetry.healthCheck()).rejects.toMatchObject({
        variant: "BadRequest",
      });

      expect(fetchMock).toHaveBeenCalledTimes(1);
    });

    it("exhausts retries and throws last error", async () => {
      const clientWithRetry = new LightconeApiClient({
        retry: { maxRetries: 2, baseDelayMs: 10, maxDelayMs: 100 },
      });
      global.fetch = fetchMock as unknown as typeof fetch;

      // All calls fail with 500
      fetchMock.mockResolvedValue({
        ok: false,
        status: 500,
        json: async () => ({ error: "server error" }),
      });

      await expect(clientWithRetry.healthCheck()).rejects.toMatchObject({
        variant: "ServerError",
      });

      expect(fetchMock).toHaveBeenCalledTimes(3); // 1 initial + 2 retries
    });
  });

  describe("limit validation", () => {
    it("throws for limit below 1", async () => {
      await expect(
        client.getTrades({ orderbook_id: "ob1", limit: 0 })
      ).rejects.toMatchObject({
        variant: "InvalidParameter",
      });
    });

    it("throws for limit above 500", async () => {
      await expect(
        client.getTrades({ orderbook_id: "ob1", limit: 501 })
      ).rejects.toMatchObject({
        variant: "InvalidParameter",
      });
    });

    it("accepts valid limit", async () => {
      fetchMock.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          orderbook_id: "ob1",
          trades: [],
          has_more: false,
        }),
      });

      await client.getTrades({ orderbook_id: "ob1", limit: 100 });

      expect(fetchMock).toHaveBeenCalled();
    });
  });
});
