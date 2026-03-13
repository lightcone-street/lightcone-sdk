import type { Resolution } from "../../shared";
import { RetryPolicy, type LightconeHttp } from "../../http";
import type { DepositTokenPriceHistoryResponse } from "./wire";

interface ClientContext {
  http: LightconeHttp;
}

export class DepositPriceClient {
  constructor(private readonly client: ClientContext) {}

  async get(
    depositAsset: string,
    resolution: Resolution,
    from?: number,
    to?: number,
    cursor?: number,
    limit?: number
  ): Promise<DepositTokenPriceHistoryResponse> {
    const params = new URLSearchParams({
      deposit_asset: depositAsset,
      resolution,
    });
    if (from !== undefined) params.set("from", String(from));
    if (to !== undefined) params.set("to", String(to));
    if (cursor !== undefined) params.set("cursor", String(cursor));
    if (limit !== undefined) params.set("limit", String(limit));

    const url = `${this.client.http.baseUrl()}/api/price-history?${params.toString()}`;
    return this.client.http.get<DepositTokenPriceHistoryResponse>(url, RetryPolicy.Idempotent);
  }
}
