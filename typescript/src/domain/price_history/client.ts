import { SdkError } from "../../error";
import type { Resolution } from "../../shared";
import { RetryPolicy, type LightconeHttp } from "../../http";
import type { PriceHistoryRestResponse } from "./wire";

interface ClientContext {
  http: LightconeHttp;
}

export class PriceHistoryClient {
  constructor(private readonly client: ClientContext) {}

  // `from`/`to` are Unix timestamps in milliseconds.
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
    if (from !== undefined) params.set("from", String(ensureUnixMilliseconds("from", from)));
    if (to !== undefined) params.set("to", String(ensureUnixMilliseconds("to", to)));

    const url = `${this.client.http.baseUrl()}/api/price-history?${params.toString()}`;
    return this.client.http.get<PriceHistoryRestResponse>(url, RetryPolicy.Idempotent);
  }
}

function ensureUnixMilliseconds(name: string, value: number): number {
  if (!Number.isFinite(value) || !Number.isInteger(value) || value < 0) {
    throw SdkError.validation(`${name} must be a non-negative Unix timestamp in milliseconds`);
  }
  if (value < 10_000_000_000) {
    throw SdkError.validation(`${name} must be a Unix timestamp in milliseconds, not seconds`);
  }
  return value;
}
