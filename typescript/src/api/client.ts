/**
 * REST API client for Lightcone.
 */

import { ApiError, getErrorMessage, type ErrorResponse } from "./error";
import type {
  MarketsResponse,
  MarketInfoResponse,
  DepositAssetsResponse,
  OrderbookResponse,
  SubmitOrderRequest,
  OrderResponse,
  CancelOrderRequest,
  CancelResponse,
  CancelAllOrdersRequest,
  CancelAllResponse,
  PositionsResponse,
  MarketPositionsResponse,
  UserOrdersResponse,
  PriceHistoryParams,
  PriceHistoryResponse,
  TradesParams,
  TradesResponse,
  AdminResponse,
  CreateOrderbookRequest,
  CreateOrderbookResponse,
} from "./types";

/**
 * Convert a typed params object to query string record.
 */
function toQueryParams(
  params: object
): Record<string, string> {
  const result: Record<string, string> = {};
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null) {
      result[key] = String(value);
    }
  }
  return result;
}

/**
 * Default API base URL for Lightcone.
 */
export const DEFAULT_API_URL = "https://lightcone.xyz/api";

/**
 * Configuration for the Lightcone API client.
 */
export interface LightconeApiClientConfig {
  /** Base URL for the API (default: https://lightcone.xyz/api) */
  baseUrl?: string;
  /** Request timeout in milliseconds (default: 30000) */
  timeout?: number;
  /** Additional headers to include in requests */
  headers?: Record<string, string>;
}

/**
 * REST API client for the Lightcone platform.
 *
 * @example
 * ```typescript
 * import { LightconeApiClient } from "@lightcone/sdk/api";
 *
 * const client = new LightconeApiClient();
 *
 * // Get all markets
 * const markets = await client.getMarkets();
 *
 * // Get orderbook
 * const orderbook = await client.getOrderbook("market1:ob1");
 * ```
 */
export class LightconeApiClient {
  private readonly baseUrl: string;
  private readonly timeout: number;
  private readonly headers: Record<string, string>;

  constructor(config: LightconeApiClientConfig = {}) {
    this.baseUrl = config.baseUrl || DEFAULT_API_URL;
    this.timeout = config.timeout || 30000;
    this.headers = {
      "Content-Type": "application/json",
      ...config.headers,
    };
  }

  // ============================================================================
  // PRIVATE HELPERS
  // ============================================================================

  /**
   * Make an HTTP request.
   */
  private async request<T>(
    method: string,
    path: string,
    body?: unknown,
    queryParams?: Record<string, string>
  ): Promise<T> {
    let url = `${this.baseUrl}${path}`;

    // Add query parameters
    if (queryParams && Object.keys(queryParams).length > 0) {
      const params = new URLSearchParams(queryParams);
      url += `?${params.toString()}`;
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(url, {
        method,
        headers: this.headers,
        body: body ? JSON.stringify(body) : undefined,
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        let errorMessage = `HTTP ${response.status}`;
        try {
          const errorData = (await response.json()) as ErrorResponse;
          errorMessage = getErrorMessage(errorData);
        } catch {
          // Ignore JSON parse errors
        }
        throw ApiError.fromStatus(response.status, errorMessage);
      }

      const data = (await response.json()) as T;
      return data;
    } catch (error) {
      clearTimeout(timeoutId);

      if (error instanceof ApiError) {
        throw error;
      }

      if (error instanceof Error) {
        if (error.name === "AbortError") {
          throw ApiError.http("Request timeout");
        }
        throw ApiError.http(error.message);
      }

      throw ApiError.http("Unknown error");
    }
  }

  // ============================================================================
  // HEALTH CHECK
  // ============================================================================

  /**
   * Check if the API is healthy.
   *
   * @throws {ApiError} If the health check fails
   */
  async healthCheck(): Promise<void> {
    await this.request<{ status: string }>("GET", "/health");
  }

  // ============================================================================
  // MARKETS
  // ============================================================================

  /**
   * Get all available markets.
   *
   * @returns List of markets
   */
  async getMarkets(): Promise<MarketsResponse> {
    return this.request<MarketsResponse>("GET", "/markets");
  }

  /**
   * Get market information by pubkey.
   *
   * @param marketPubkey - Market PDA address (Base58)
   * @returns Market details
   */
  async getMarket(marketPubkey: string): Promise<MarketInfoResponse> {
    return this.request<MarketInfoResponse>(
      "GET",
      `/markets/${encodeURIComponent(marketPubkey)}`
    );
  }

  /**
   * Get market information by slug.
   *
   * @param slug - URL-friendly market slug
   * @returns Market details
   */
  async getMarketBySlug(slug: string): Promise<MarketInfoResponse> {
    return this.request<MarketInfoResponse>(
      "GET",
      `/markets/by-slug/${encodeURIComponent(slug)}`
    );
  }

  /**
   * Get deposit assets for a market.
   *
   * @param marketPubkey - Market PDA address (Base58)
   * @returns Deposit assets
   */
  async getDepositAssets(marketPubkey: string): Promise<DepositAssetsResponse> {
    return this.request<DepositAssetsResponse>(
      "GET",
      `/markets/${encodeURIComponent(marketPubkey)}/deposit-assets`
    );
  }

  // ============================================================================
  // ORDERBOOK
  // ============================================================================

  /**
   * Get orderbook snapshot.
   *
   * @param orderbookId - Orderbook identifier
   * @param depth - Number of levels (default: 20)
   * @returns Orderbook snapshot
   */
  async getOrderbook(
    orderbookId: string,
    depth?: number
  ): Promise<OrderbookResponse> {
    return this.request<OrderbookResponse>(
      "GET",
      `/orderbook/${encodeURIComponent(orderbookId)}`,
      undefined,
      toQueryParams({ depth })
    );
  }

  // ============================================================================
  // ORDERS
  // ============================================================================

  /**
   * Submit a new order.
   *
   * @param request - Order details
   * @returns Order response with hash and fill info
   */
  async submitOrder(request: SubmitOrderRequest): Promise<OrderResponse> {
    return this.request<OrderResponse>("POST", "/orders/submit", request);
  }

  /**
   * Cancel an order.
   *
   * @param orderHash - Hash of the order to cancel (hex)
   * @param maker - Order creator's pubkey (Base58)
   * @returns Cancel response
   */
  async cancelOrder(orderHash: string, maker: string): Promise<CancelResponse> {
    const request: CancelOrderRequest = { order_hash: orderHash, maker };
    return this.request<CancelResponse>("POST", "/orders/cancel", request);
  }

  /**
   * Cancel all orders for a user.
   *
   * @param userPubkey - User's public key (Base58)
   * @param marketPubkey - Optional market filter (Base58)
   * @returns Cancel all response
   */
  async cancelAllOrders(
    userPubkey: string,
    marketPubkey?: string
  ): Promise<CancelAllResponse> {
    const request: CancelAllOrdersRequest = {
      user_pubkey: userPubkey,
      market_pubkey: marketPubkey,
    };
    return this.request<CancelAllResponse>("POST", "/orders/cancel-all", request);
  }

  // ============================================================================
  // USER POSITIONS
  // ============================================================================

  /**
   * Get user positions across all markets.
   *
   * @param userPubkey - User's public key (Base58)
   * @returns User positions
   */
  async getUserPositions(userPubkey: string): Promise<PositionsResponse> {
    return this.request<PositionsResponse>(
      "GET",
      `/users/${encodeURIComponent(userPubkey)}/positions`
    );
  }

  /**
   * Get user positions in a specific market.
   *
   * @param userPubkey - User's public key (Base58)
   * @param marketPubkey - Market PDA address (Base58)
   * @returns Market positions
   */
  async getUserMarketPositions(
    userPubkey: string,
    marketPubkey: string
  ): Promise<MarketPositionsResponse> {
    return this.request<MarketPositionsResponse>(
      "GET",
      `/users/${encodeURIComponent(userPubkey)}/markets/${encodeURIComponent(marketPubkey)}/positions`
    );
  }

  // ============================================================================
  // USER ORDERS
  // ============================================================================

  /**
   * Get user's open orders.
   *
   * @param userPubkey - User's public key (Base58)
   * @returns User orders and balances
   */
  async getUserOrders(userPubkey: string): Promise<UserOrdersResponse> {
    return this.request<UserOrdersResponse>("POST", "/users/orders", {
      user_pubkey: userPubkey,
    });
  }

  // ============================================================================
  // PRICE HISTORY
  // ============================================================================

  /**
   * Get historical price data.
   *
   * @param params - Query parameters
   * @returns Price history data
   */
  async getPriceHistory(params: PriceHistoryParams): Promise<PriceHistoryResponse> {
    return this.request<PriceHistoryResponse>(
      "GET",
      "/price-history",
      undefined,
      toQueryParams(params)
    );
  }

  // ============================================================================
  // TRADES
  // ============================================================================

  /**
   * Get recent trades.
   *
   * @param params - Query parameters
   * @returns Trade history
   */
  async getTrades(params: TradesParams): Promise<TradesResponse> {
    return this.request<TradesResponse>(
      "GET",
      "/trades",
      undefined,
      toQueryParams(params)
    );
  }

  // ============================================================================
  // ADMIN
  // ============================================================================

  /**
   * Admin health check.
   *
   * @returns Admin status
   */
  async adminHealthCheck(): Promise<AdminResponse> {
    return this.request<AdminResponse>("GET", "/admin/test");
  }

  /**
   * Create a new orderbook (admin only).
   *
   * @param request - Orderbook creation request
   * @returns Created orderbook info
   */
  async createOrderbook(
    request: CreateOrderbookRequest
  ): Promise<CreateOrderbookResponse> {
    return this.request<CreateOrderbookResponse>(
      "POST",
      "/admin/create-orderbook",
      request
    );
  }
}
