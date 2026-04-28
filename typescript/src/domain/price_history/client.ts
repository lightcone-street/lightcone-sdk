import { SdkError } from "../../error";
import { Resolution } from "../../shared";
import { RetryPolicy, type LightconeHttp } from "../../http";
import type {
  DepositAssetPricesSnapshotResponse,
  DepositTokenPriceHistoryResponse,
  OrderbookPriceHistoryResponse,
  PriceCandle,
} from "./wire";

interface ClientContext {
  http: LightconeHttp;
}

export interface OrderbookPriceHistoryQuery {
  resolution?: Resolution;
  from?: number;
  to?: number;
  cursor?: number;
  limit?: number;
  includeOhlcv?: boolean;
}

export interface DepositPriceHistoryQuery {
  resolution?: Resolution;
  from?: number;
  to?: number;
  cursor?: number;
  limit?: number;
}

export class PriceHistoryClient {
  constructor(private readonly client: ClientContext) {}

  async get(
    orderbookId: string,
    resolution?: Resolution,
    from?: number,
    to?: number
  ): Promise<OrderbookPriceHistoryResponse>;

  async get(
    orderbookId: string,
    query?: OrderbookPriceHistoryQuery
  ): Promise<OrderbookPriceHistoryResponse>;

  // `from`/`to` are Unix timestamps in milliseconds.
  async get(
    orderbookId: string,
    resolutionOrQuery: Resolution | OrderbookPriceHistoryQuery = Resolution.Minute1,
    from?: number,
    to?: number
  ): Promise<OrderbookPriceHistoryResponse> {
    const query = normalizeOrderbookPriceHistoryQuery(resolutionOrQuery, from, to);
    const params = new URLSearchParams({
      orderbook_id: orderbookId,
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
    if (query.includeOhlcv !== undefined) {
      params.set("include_ohlcv", String(query.includeOhlcv));
    }

    const url = `${this.client.http.baseUrl()}/api/price-history?${params.toString()}`;
    return this.client.http.get<OrderbookPriceHistoryResponse>(url, RetryPolicy.Idempotent);
  }

  async getDepositAsset(
    depositAsset: string,
    resolution?: Resolution,
    from?: number,
    to?: number,
    cursor?: number,
    limit?: number
  ): Promise<DepositTokenPriceHistoryResponse>;

  async getDepositAsset(
    depositAsset: string,
    query?: DepositPriceHistoryQuery
  ): Promise<DepositTokenPriceHistoryResponse>;

  async getDepositAsset(
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

  /**
   * Snapshot of current prices for every active mint in `global_deposit_tokens`.
   *
   * No params. Returns a map of mint -> price (Decimal-as-string). The
   * backend prefers the live tick from `deposit_token_prices` and falls
   * back to the most recent 1m candle close. Assets with neither are
   * silently absent.
   *
   * For live updates, subscribe via `subscribeDepositAssetPrice` per asset.
   */
  async getDepositAssetPricesSnapshot(): Promise<DepositAssetPricesSnapshotResponse> {
    const url = `${this.client.http.baseUrl()}/api/deposit-asset-prices-snapshot`;
    return this.client.http.get<DepositAssetPricesSnapshotResponse>(url, RetryPolicy.Idempotent);
  }

  async getLineData(
    orderbookId: string,
    resolution: Resolution = Resolution.Minute1,
    from?: number,
    to?: number,
    cursor?: number,
    limit?: number
  ): Promise<{ time: number; value: string }[]> {
    const response = await this.get(orderbookId, {
      resolution,
      from,
      to,
      cursor,
      limit,
      includeOhlcv: false,
    });
    return response.prices.map((c: PriceCandle) => ({
      time: c.t,
      value: c.m ?? "",
    }));
  }
}

function normalizeOrderbookPriceHistoryQuery(
  resolutionOrQuery: Resolution | OrderbookPriceHistoryQuery,
  from?: number,
  to?: number
): Required<Pick<OrderbookPriceHistoryQuery, "resolution">> & Omit<OrderbookPriceHistoryQuery, "resolution"> {
  if (typeof resolutionOrQuery === "string") {
    return {
      resolution: resolutionOrQuery,
      from,
      to,
    };
  }

  return {
    resolution: resolutionOrQuery.resolution ?? Resolution.Minute1,
    from: resolutionOrQuery.from,
    to: resolutionOrQuery.to,
    cursor: resolutionOrQuery.cursor,
    limit: resolutionOrQuery.limit,
    includeOhlcv: resolutionOrQuery.includeOhlcv,
  };
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
