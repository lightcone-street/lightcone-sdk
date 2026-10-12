import { RetryPolicy, type LightconeHttp } from "../../http";
import type { TradesPage } from "./index";
import { tradeFromResponse } from "./convert";
import { marketTradesResponseFromWire, tradesResponseFromWire } from "./wire";

interface ClientContext {
  http: LightconeHttp;
}

export class Trades {
  constructor(private readonly client: ClientContext) {}

  /**
   * One page of an orderbook's trades, newest first. `cursor` is a previous
   * page's `nextCursor`. A malformed trade fails the page.
   */
  async get(orderbookId: string, limit?: number, cursor?: bigint): Promise<TradesPage> {
    const params = new URLSearchParams({ orderbook_id: orderbookId });
    if (limit !== undefined) params.set("limit", String(limit));
    if (cursor !== undefined) params.set("cursor", String(cursor));

    const url = `${this.client.http.baseUrl()}/api/trades?${params.toString()}`;
    const response = tradesResponseFromWire(
      await this.client.http.get<unknown>(url, RetryPolicy.Idempotent),
    );

    return {
      trades: response.trades.map(tradeFromResponse),
      nextCursor: response.next_cursor,
      hasMore: response.has_more,
    };
  }

  /** One page of a market's trades across all of its orderbooks, newest first. */
  async getByMarket(
    marketPubkey: string,
    limit?: number,
    cursor?: bigint,
  ): Promise<TradesPage> {
    const params = new URLSearchParams({ market_pubkey: marketPubkey });
    if (limit !== undefined) params.set("limit", String(limit));
    if (cursor !== undefined) params.set("cursor", String(cursor));

    const url = `${this.client.http.baseUrl()}/api/trades/market?${params.toString()}`;
    const response = marketTradesResponseFromWire(
      await this.client.http.get<unknown>(url, RetryPolicy.Idempotent),
    );

    return {
      trades: response.trades.map(tradeFromResponse),
      nextCursor: response.next_cursor,
      hasMore: response.has_more,
    };
  }
}
