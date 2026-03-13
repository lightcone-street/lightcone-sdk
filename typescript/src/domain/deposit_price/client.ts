import { SdkError } from "../../error";
import { Resolution } from "../../shared";
import { RetryPolicy, type LightconeHttp } from "../../http";
import type { DepositTokenPriceHistoryResponse } from "./wire";

interface ClientContext {
  http: LightconeHttp;
}

export interface DepositPriceHistoryQuery {
  resolution?: Resolution;
  from?: number;
  to?: number;
  cursor?: number;
  limit?: number;
}

export class DepositPriceClient {
  constructor(private readonly client: ClientContext) {}

  async get(
    depositAsset: string,
    resolution?: Resolution,
    from?: number,
    to?: number,
    cursor?: number,
    limit?: number
  ): Promise<DepositTokenPriceHistoryResponse>;

  async get(
    depositAsset: string,
    query?: DepositPriceHistoryQuery
  ): Promise<DepositTokenPriceHistoryResponse>;

  async get(
    depositAsset: string,
    resolutionOrQuery: Resolution | DepositPriceHistoryQuery = Resolution.Minute1,
    from?: number,
    to?: number,
    cursor?: number,
    limit?: number
  ): Promise<DepositTokenPriceHistoryResponse> {
    const query = normalizeDepositPriceHistoryQuery(
      resolutionOrQuery,
      from,
      to,
      cursor,
      limit
    );
    const params = new URLSearchParams({
      deposit_asset: depositAsset,
      resolution: query.resolution,
    });
    if (query.from !== undefined) {
      params.set("from", String(ensureUnixMilliseconds("from", query.from)));
    }
    if (query.to !== undefined) {
      params.set("to", String(ensureUnixMilliseconds("to", query.to)));
    }
    if (query.cursor !== undefined) {
      params.set("cursor", String(ensureUnixMilliseconds("cursor", query.cursor)));
    }
    if (query.limit !== undefined) {
      params.set("limit", String(ensurePageLimit(query.limit)));
    }

    const url = `${this.client.http.baseUrl()}/api/price-history?${params.toString()}`;
    return this.client.http.get<DepositTokenPriceHistoryResponse>(url, RetryPolicy.Idempotent);
  }
}

function normalizeDepositPriceHistoryQuery(
  resolutionOrQuery: Resolution | DepositPriceHistoryQuery,
  from?: number,
  to?: number,
  cursor?: number,
  limit?: number
): Required<Pick<DepositPriceHistoryQuery, "resolution">> & Omit<DepositPriceHistoryQuery, "resolution"> {
  if (typeof resolutionOrQuery === "string") {
    return {
      resolution: resolutionOrQuery,
      from,
      to,
      cursor,
      limit,
    };
  }

  return {
    resolution: resolutionOrQuery.resolution ?? Resolution.Minute1,
    from: resolutionOrQuery.from,
    to: resolutionOrQuery.to,
    cursor: resolutionOrQuery.cursor,
    limit: resolutionOrQuery.limit,
  };
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

function ensurePageLimit(value: number): number {
  if (!Number.isFinite(value) || !Number.isInteger(value) || value < 1 || value > 1_000) {
    throw SdkError.validation("limit must be an integer between 1 and 1000");
  }
  return value;
}
