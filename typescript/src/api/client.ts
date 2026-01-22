/**
 * REST API client for Lightcone.
 */

import { ApiError, getErrorMessage, type ErrorResponse } from "./error";
import {
  validatePubkey,
  validateSignature,
  validateLimit,
  DEFAULT_TIMEOUT_MS,
} from "./validation";
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
 * Configuration for retry behavior.
 */
export interface RetryConfig {
  /** Maximum number of retry attempts (default: 0 = disabled) */
  maxRetries: number;
  /** Initial delay in milliseconds (default: 100) */
  baseDelayMs: number;
  /** Maximum delay cap in milliseconds (default: 10000) */
  maxDelayMs: number;
}

/** Default retry configuration (disabled) */
export const DEFAULT_RETRY_CONFIG: RetryConfig = {
  maxRetries: 0,
  baseDelayMs: 100,
  maxDelayMs: 10000,
};

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
  /** Retry configuration for transient failures */
  retry?: Partial<RetryConfig>;
}

/**
 * Calculate delay with exponential backoff and jitter.
 * Jitter: 75-100% of calculated delay (prevents thundering herd)
 */
function calculateRetryDelay(attempt: number, config: RetryConfig): number {
  const expDelay = config.baseDelayMs * Math.pow(2, Math.min(attempt, 10));
  const cappedDelay = Math.min(expDelay, config.maxDelayMs);
  const jitterRange = cappedDelay * 0.25;
  const jitter = Math.random() * jitterRange;
  return cappedDelay - jitterRange + jitter;
}

/**
 * Check if error is retryable.
 */
function isRetryable(error: ApiError): boolean {
  if (error.variant === "ServerError") return true;
  if (error.variant === "RateLimited") return true;
  if (error.variant === "Http") return true; // Network errors
  return false;
}

/**
 * Sleep for a duration.
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
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
  private readonly retryConfig: RetryConfig;

  constructor(config: LightconeApiClientConfig = {}) {
    this.baseUrl = config.baseUrl || DEFAULT_API_URL;
    this.timeout = config.timeout || DEFAULT_TIMEOUT_MS;
    this.headers = {
      "Content-Type": "application/json",
      ...config.headers,
    };
    this.retryConfig = {
      ...DEFAULT_RETRY_CONFIG,
      ...config.retry,
    };
  }

  // ============================================================================
  // PRIVATE HELPERS
  // ============================================================================

  /**
   * Make an HTTP request with retry support.
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

    let lastError: ApiError | undefined;

    for (let attempt = 0; attempt <= this.retryConfig.maxRetries; attempt++) {
      // Delay before retry (not on first attempt)
      if (attempt > 0) {
        const delay = calculateRetryDelay(attempt - 1, this.retryConfig);
        await sleep(delay);
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
          } catch (parseError) {
            console.warn(
              `Failed to parse error response: ${parseError instanceof Error ? parseError.message : String(parseError)}`
            );
          }
          const error = ApiError.fromStatus(response.status, errorMessage);

          // Check if retryable
          if (isRetryable(error) && attempt < this.retryConfig.maxRetries) {
            lastError = error;
            continue;
          }
          throw error;
        }

        const data = (await response.json()) as T;
        return data;
      } catch (error) {
        clearTimeout(timeoutId);

        if (error instanceof ApiError) {
          // Already an ApiError - check if retryable
          if (isRetryable(error) && attempt < this.retryConfig.maxRetries) {
            lastError = error;
            continue;
          }
          throw error;
        }

        let apiError: ApiError;
        if (error instanceof Error) {
          if (error.name === "AbortError") {
            apiError = ApiError.http("Request timeout");
          } else {
            apiError = ApiError.http(error.message);
          }
        } else {
          apiError = ApiError.http("Unknown error");
        }

        // Network errors are retryable
        if (isRetryable(apiError) && attempt < this.retryConfig.maxRetries) {
          lastError = apiError;
          continue;
        }
        throw apiError;
      }
    }

    // Should not reach here, but throw last error if we do
    throw lastError || ApiError.http("Unknown error");
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
   * @throws {ApiError} If marketPubkey is invalid
   */
  async getMarket(marketPubkey: string): Promise<MarketInfoResponse> {
    validatePubkey(marketPubkey, "marketPubkey");
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
   * @throws {ApiError} If marketPubkey is invalid
   */
  async getDepositAssets(marketPubkey: string): Promise<DepositAssetsResponse> {
    validatePubkey(marketPubkey, "marketPubkey");
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
   * @throws {ApiError} If any pubkey or signature is invalid
   */
  async submitOrder(request: SubmitOrderRequest): Promise<OrderResponse> {
    validatePubkey(request.maker, "maker");
    validatePubkey(request.market_pubkey, "market_pubkey");
    validatePubkey(request.base_token, "base_token");
    validatePubkey(request.quote_token, "quote_token");
    validateSignature(request.signature);
    return this.request<OrderResponse>("POST", "/orders/submit", request);
  }

  /**
   * Cancel an order.
   *
   * @param orderHash - Hash of the order to cancel (hex)
   * @param maker - Order creator's pubkey (Base58)
   * @returns Cancel response
   * @throws {ApiError} If maker pubkey is invalid
   */
  async cancelOrder(orderHash: string, maker: string): Promise<CancelResponse> {
    validatePubkey(maker, "maker");
    const request: CancelOrderRequest = { order_hash: orderHash, maker };
    return this.request<CancelResponse>("POST", "/orders/cancel", request);
  }

  /**
   * Cancel all orders for a user.
   *
   * @param userPubkey - User's public key (Base58)
   * @param marketPubkey - Optional market filter (Base58)
   * @returns Cancel all response
   * @throws {ApiError} If any pubkey is invalid
   */
  async cancelAllOrders(
    userPubkey: string,
    marketPubkey?: string
  ): Promise<CancelAllResponse> {
    validatePubkey(userPubkey, "userPubkey");
    if (marketPubkey) {
      validatePubkey(marketPubkey, "marketPubkey");
    }
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
   * @throws {ApiError} If userPubkey is invalid
   */
  async getUserPositions(userPubkey: string): Promise<PositionsResponse> {
    validatePubkey(userPubkey, "userPubkey");
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
   * @throws {ApiError} If any pubkey is invalid
   */
  async getUserMarketPositions(
    userPubkey: string,
    marketPubkey: string
  ): Promise<MarketPositionsResponse> {
    validatePubkey(userPubkey, "userPubkey");
    validatePubkey(marketPubkey, "marketPubkey");
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
   * @throws {ApiError} If userPubkey is invalid
   */
  async getUserOrders(userPubkey: string): Promise<UserOrdersResponse> {
    validatePubkey(userPubkey, "userPubkey");
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
   * @throws {ApiError} If limit is out of bounds (1-500)
   */
  async getPriceHistory(params: PriceHistoryParams): Promise<PriceHistoryResponse> {
    validateLimit(params.limit);
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
   * @throws {ApiError} If limit is out of bounds (1-500)
   */
  async getTrades(params: TradesParams): Promise<TradesResponse> {
    validateLimit(params.limit);
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
