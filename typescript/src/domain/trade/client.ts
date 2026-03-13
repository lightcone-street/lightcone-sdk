import { RetryPolicy, type LightconeHttp } from "../../http";
import type { TradesPage } from "./index";
import { tradeFromResponse } from "./convert";
import type { TradesResponse } from "./wire";

interface ClientContext {
  http: LightconeHttp;
}

export class Trades {
  constructor(private readonly client: ClientContext) {}

  async get(orderbookId: string, limit?: number, cursor?: number): Promise<TradesPage> {
    const params = new URLSearchParams({ orderbook_id: orderbookId });
    if (limit !== undefined) params.set("limit", String(limit));
    if (cursor !== undefined) params.set("cursor", String(cursor));

    const url = `${this.client.http.baseUrl()}/api/trades?${params.toString()}`;
    const response = await this.client.http.get<TradesResponse>(url, RetryPolicy.Idempotent);

    return {
      trades: response.trades.map(tradeFromResponse),
      nextCursor: response.next_cursor,
      hasMore: response.has_more ?? false,
    };
  }
}
