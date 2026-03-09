import { RetryPolicy, type LightconeHttp } from "../../http";
import type { DecimalsResponse, OrderbookDepthResponse } from "./wire";

interface CacheContext {
  get(orderbookId: string): DecimalsResponse | undefined;
  set(orderbookId: string, response: DecimalsResponse): void;
  clear(): void;
}

interface ClientContext {
  http: LightconeHttp;
  decimalsCache: CacheContext;
}

export class Orderbooks {
  constructor(private readonly client: ClientContext) {}

  async get(orderbookId: string, depth?: number): Promise<OrderbookDepthResponse> {
    const query = depth !== undefined ? `?depth=${depth}` : "";
    const url = `${this.client.http.baseUrl()}/api/orderbook/${encodeURIComponent(orderbookId)}${query}`;
    return this.client.http.get<OrderbookDepthResponse>(url, RetryPolicy.Idempotent);
  }

  async decimals(orderbookId: string): Promise<DecimalsResponse> {
    const cached = this.client.decimalsCache.get(orderbookId);
    if (cached) {
      return cached;
    }

    const url = `${this.client.http.baseUrl()}/api/orderbooks/${encodeURIComponent(orderbookId)}/decimals`;
    const response = await this.client.http.get<DecimalsResponse>(url, RetryPolicy.Idempotent);
    this.client.decimalsCache.set(orderbookId, response);
    return response;
  }

  async clearCache(): Promise<void> {
    this.client.decimalsCache.clear();
  }
}
