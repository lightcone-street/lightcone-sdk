import type { Resolution } from "../../shared";
import { RetryPolicy, type LightconeHttp } from "../../http";
import type { PriceHistoryRestResponse } from "./wire";

interface ClientContext {
  http: LightconeHttp;
}

export class PriceHistoryClient {
  constructor(private readonly client: ClientContext) {}

  async get(
    orderbookId: string,
    resolution: Resolution,
    from?: number,
    to?: number
  ): Promise<PriceHistoryRestResponse> {
    const params = new URLSearchParams({
      orderbook_id: orderbookId,
      resolution,
    });
    if (from !== undefined) params.set("from", String(from));
    if (to !== undefined) params.set("to", String(to));

    const url = `${this.client.http.baseUrl()}/api/price-history?${params.toString()}`;
    return this.client.http.get<PriceHistoryRestResponse>(url, RetryPolicy.Idempotent);
  }
}
