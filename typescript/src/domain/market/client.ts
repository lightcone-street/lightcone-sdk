import { SdkError } from "../../error";
import { RetryPolicy } from "../../http";
import type { LightconeHttp } from "../../http";
import { marketFromWire } from "./convert";
import { Status, type Market } from "./index";
import type { MarketSearchResult, MarketsResponse, SingleMarketResponse } from "./wire";

export interface MarketsResult {
  markets: Market[];
  validationErrors: string[];
}

interface ClientContext {
  http: LightconeHttp;
}

export class Markets {
  constructor(private readonly client: ClientContext) {}

  async get(cursor?: number, limit?: number): Promise<MarketsResult> {
    const search = new URLSearchParams();
    if (cursor !== undefined) search.set("cursor", String(cursor));
    if (limit !== undefined) search.set("limit", String(limit));

    const suffix = search.size > 0 ? `?${search.toString()}` : "";
    const url = `${this.client.http.baseUrl()}/api/markets${suffix}`;

    const response = await this.client.http.get<MarketsResponse>(url, RetryPolicy.Idempotent);
    const markets: Market[] = [];
    const validationErrors: string[] = [];

    for (const marketWire of response.markets) {
      try {
        const market = marketFromWire(marketWire);
        if (market.status === Status.Active || market.status === Status.Resolved) {
          markets.push(market);
        }
      } catch (error) {
        validationErrors.push(error instanceof Error ? error.message : String(error));
      }
    }

    return { markets, validationErrors };
  }

  async getBySlug(slug: string): Promise<Market> {
    const url = `${this.client.http.baseUrl()}/api/markets/by-slug/${encodeURIComponent(slug)}`;
    const response = await this.client.http.get<SingleMarketResponse>(url, RetryPolicy.Idempotent);

    try {
      return marketFromWire(response.market);
    } catch (error) {
      throw SdkError.validation(error instanceof Error ? error.message : String(error));
    }
  }

  async getByPubkey(pubkey: string): Promise<Market> {
    const url = `${this.client.http.baseUrl()}/api/markets/${encodeURIComponent(pubkey)}`;
    const response = await this.client.http.get<SingleMarketResponse>(url, RetryPolicy.Idempotent);

    try {
      return marketFromWire(response.market);
    } catch (error) {
      throw SdkError.validation(error instanceof Error ? error.message : String(error));
    }
  }

  async search(query: string, limit?: number): Promise<MarketSearchResult[]> {
    const encoded = encodeURIComponent(query);
    const suffix = limit !== undefined ? `?limit=${limit}` : "";
    const url = `${this.client.http.baseUrl()}/api/markets/search/by-query/${encoded}${suffix}`;
    return this.client.http.get<MarketSearchResult[]>(url, RetryPolicy.Idempotent);
  }

  async featured(): Promise<MarketSearchResult[]> {
    const url = `${this.client.http.baseUrl()}/api/markets/search/featured`;
    const result = await this.client.http.get<MarketSearchResult[]>(url, RetryPolicy.Idempotent);
    return result.filter(
      (item) => item.market_status === Status.Active || item.market_status === Status.Resolved
    );
  }
}
