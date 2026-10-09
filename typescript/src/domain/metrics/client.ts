import type { ClientContext } from "../../context";
import { SdkError } from "../../error";
import { RetryPolicy } from "../../http";
import { wireArray, wireBool, wireObject, wireOptString } from "../../shared/wire_decode";
import { Resolution, type OrderBookId, type PubkeyStr } from "../../shared";
import type {
  CategoriesMetrics,
  CategoryVolumeMetrics,
  DepositTokenVolumeHistory,
  DepositTokenVolumeHistoryQuery,
  DepositTokensMetrics,
  Leaderboard,
  MarketDetailMetrics,
  MarketsMetrics,
  MetricsHistory,
  MetricsHistoryQuery,
  OpenInterestHistory,
  OpenInterestHistoryQuery,
  OrderbookTickersResponse,
  OrderbookVolumeMetrics,
  PlatformMetrics,
  UniqueTradersHistory,
  UniqueTradersHistoryQuery,
  UserMetrics,
} from "./wire";

/**
 * Upper bound on pages followed by {@link Metrics.orderbookTickers} (8 tickers
 * per page), so a misbehaving cursor can never loop forever.
 */
const MAX_TICKER_PAGES = 1_000;

/**
 * Follow ticker pages until the last one or `maxPages`, which leaves
 * `has_more` true with the cursor to resume from. A page that reports more
 * without an advancing cursor fails instead of truncating silently.
 *
 * @internal Exported for tests.
 */
export async function collectTickerPages(
  maxPages: number,
  fetch: (cursor: string | undefined) => Promise<OrderbookTickersResponse>,
): Promise<OrderbookTickersResponse> {
  const tickers: OrderbookTickersResponse["tickers"] = [];
  let cursor: string | undefined;
  for (let page = 0; page < maxPages; page += 1) {
    const response = await fetch(cursor);
    tickers.push(...response.tickers);
    if (!response.has_more) {
      return { tickers, next_cursor: undefined, has_more: false };
    }
    const next = response.next_cursor;
    if (next === undefined || next === cursor) {
      throw SdkError.validation(
        `orderbook tickers page reported more without an advancing cursor (after ${String(cursor)}: ${String(next)})`,
      );
    }
    cursor = next;
  }
  return { tickers, next_cursor: cursor, has_more: true };
}

function orderbookTickersPageFromWire(value: unknown): OrderbookTickersResponse {
  const data = wireObject(value, "orderbook_tickers");
  return {
    tickers: wireArray(
      data.tickers,
      "orderbook_tickers.tickers",
      (item, field) => wireObject(item, field) as unknown as OrderbookTickersResponse["tickers"][number],
    ),
    next_cursor: wireOptString(data.next_cursor, "orderbook_tickers.next_cursor"),
    has_more: wireBool(data.has_more, "orderbook_tickers.has_more", false),
  };
}

/**
 * Metrics sub-client — platform / market / orderbook / category / deposit-token
 * volume metrics, market leaderboard, and time-series history.
 *
 * Obtain via `client.metrics()`.
 */
export class Metrics {
  constructor(private readonly client: ClientContext) {}

  /** `GET /api/metrics/platform` */
  async platform(): Promise<PlatformMetrics> {
    const url = `${this.client.http.baseUrl()}/api/metrics/platform`;
    return this.client.http.get<PlatformMetrics>(url, RetryPolicy.Idempotent);
  }

  /** `GET /api/metrics/markets` */
  async markets(): Promise<MarketsMetrics> {
    const url = `${this.client.http.baseUrl()}/api/metrics/markets`;
    return this.client.http.get<MarketsMetrics>(url, RetryPolicy.Idempotent);
  }

  /** `GET /api/metrics/markets/{market_pubkey}` */
  async market(marketPubkey: PubkeyStr): Promise<MarketDetailMetrics> {
    const url = `${this.client.http.baseUrl()}/api/metrics/markets/${encodeURIComponent(marketPubkey)}`;
    return this.client.http.get<MarketDetailMetrics>(url, RetryPolicy.Idempotent);
  }

  /**
   * Batch BBO + midpoint for every ready orderbook (same shape as the WS
   * `Ticker` stream). Optionally filter to orderbooks whose base
   * conditional-token is backed by `depositAsset`. Prices per orderbook are
   * scaled using that orderbook's own decimals.
   *
   * The endpoint pages at most 8 tickers per request; this follows every page
   * and returns them merged with `has_more` false. If it stops at its page cap
   * first, `has_more` stays true and `next_cursor` resumes with
   * {@link orderbookTickersPage}. A page that reports more without an
   * advancing cursor throws `SdkError` `Validation`.
   *
   * `GET /api/metrics/orderbooks/tickers[?deposit_asset=<mint>&cursor=<orderbook>]`
   */
  async orderbookTickers(depositAsset?: string): Promise<OrderbookTickersResponse> {
    return collectTickerPages(MAX_TICKER_PAGES, (cursor) =>
      this.orderbookTickersPage(depositAsset, cursor),
    );
  }

  /**
   * One page of {@link orderbookTickers}. `cursor` is a previous page's
   * `next_cursor` (an orderbook pubkey); `limit` defaults to 8 and must be
   * within 1..8 (other values are rejected with `INVALID_LIMIT`).
   */
  async orderbookTickersPage(
    depositAsset?: string,
    cursor?: string,
    limit?: number,
  ): Promise<OrderbookTickersResponse> {
    const params = new URLSearchParams();
    const trimmed = depositAsset?.trim();
    if (trimmed) params.set("deposit_asset", trimmed);
    if (cursor !== undefined) params.set("cursor", cursor);
    if (limit !== undefined) params.set("limit", String(limit));
    const qs = params.toString();
    const url = `${this.client.http.baseUrl()}/api/metrics/orderbooks/tickers${qs ? `?${qs}` : ""}`;
    return orderbookTickersPageFromWire(await this.client.http.get<unknown>(url, RetryPolicy.Idempotent));
  }

  /** `GET /api/metrics/orderbooks/{orderbook_id}` */
  async orderbook(orderbookId: OrderBookId): Promise<OrderbookVolumeMetrics> {
    const url = `${this.client.http.baseUrl()}/api/metrics/orderbooks/${encodeURIComponent(orderbookId)}`;
    return this.client.http.get<OrderbookVolumeMetrics>(url, RetryPolicy.Idempotent);
  }

  /** `GET /api/metrics/categories` */
  async categories(): Promise<CategoriesMetrics> {
    const url = `${this.client.http.baseUrl()}/api/metrics/categories`;
    return this.client.http.get<CategoriesMetrics>(url, RetryPolicy.Idempotent);
  }

  /** `GET /api/metrics/categories/{category}` */
  async category(category: string): Promise<CategoryVolumeMetrics> {
    const url = `${this.client.http.baseUrl()}/api/metrics/categories/${encodeURIComponent(category)}`;
    return this.client.http.get<CategoryVolumeMetrics>(url, RetryPolicy.Idempotent);
  }

  /** `GET /api/metrics/deposit-tokens` */
  async depositTokens(): Promise<DepositTokensMetrics> {
    const url = `${this.client.http.baseUrl()}/api/metrics/deposit-tokens`;
    return this.client.http.get<DepositTokensMetrics>(url, RetryPolicy.Idempotent);
  }

  /** `GET /api/metrics/deposit-tokens/volume-history` */
  async depositTokensVolumeHistory(
    query: DepositTokenVolumeHistoryQuery = {}
  ): Promise<DepositTokenVolumeHistory> {
    const search = new URLSearchParams();
    if (query.from !== undefined) search.set("from", String(query.from));
    if (query.to !== undefined) search.set("to", String(query.to));
    if (query.limit !== undefined) search.set("limit", String(query.limit));

    const qs = search.toString();
    const url =
      `${this.client.http.baseUrl()}/api/metrics/deposit-tokens/volume-history` +
      (qs ? `?${qs}` : "");
    return this.client.http.get<DepositTokenVolumeHistory>(
      url,
      RetryPolicy.Idempotent
    );
  }

  /** `GET /api/metrics/open-interest/history` */
  async openInterestHistory(
    query: OpenInterestHistoryQuery = {}
  ): Promise<OpenInterestHistory> {
    const search = new URLSearchParams();
    if (query.from !== undefined) search.set("from", String(query.from));
    if (query.to !== undefined) search.set("to", String(query.to));
    if (query.limit !== undefined) search.set("limit", String(query.limit));

    const qs = search.toString();
    const url =
      `${this.client.http.baseUrl()}/api/metrics/open-interest/history` +
      (qs ? `?${qs}` : "");
    return this.client.http.get<OpenInterestHistory>(
      url,
      RetryPolicy.Idempotent
    );
  }

  /** `GET /api/metrics/unique-traders/history` */
  async uniqueTradersHistory(
    query: UniqueTradersHistoryQuery = {}
  ): Promise<UniqueTradersHistory> {
    const search = new URLSearchParams();
    if (query.scope !== undefined) search.set("scope", query.scope);
    if (query.scope_key !== undefined) search.set("scope_key", query.scope_key);
    if (query.from !== undefined) search.set("from", String(query.from));
    if (query.to !== undefined) search.set("to", String(query.to));
    if (query.limit !== undefined) search.set("limit", String(query.limit));

    const qs = search.toString();
    const url =
      `${this.client.http.baseUrl()}/api/metrics/unique-traders/history` +
      (qs ? `?${qs}` : "");
    return this.client.http.get<UniqueTradersHistory>(
      url,
      RetryPolicy.Idempotent
    );
  }

  /** `GET /api/metrics/leaderboard/markets` */
  async leaderboard(limit?: number): Promise<Leaderboard> {
    let url = `${this.client.http.baseUrl()}/api/metrics/leaderboard/markets`;
    if (limit !== undefined) {
      url += `?limit=${limit}`;
    }
    return this.client.http.get<Leaderboard>(url, RetryPolicy.Idempotent);
  }

  /**
   * `GET /api/metrics/history/{scope}/{scope_key}`
   *
   * `scope` is one of `"orderbook" | "market" | "category" | "deposit_token" | "platform"`.
   */
  async history(
    scope: string,
    scopeKey: string,
    query: MetricsHistoryQuery = {}
  ): Promise<MetricsHistory> {
    const search = new URLSearchParams();
    search.set("resolution", query.resolution ?? Resolution.Hour1);
    if (query.from !== undefined) search.set("from", String(query.from));
    if (query.to !== undefined) search.set("to", String(query.to));
    if (query.limit !== undefined) search.set("limit", String(query.limit));

    const url = `${this.client.http.baseUrl()}/api/metrics/history/${encodeURIComponent(scope)}/${encodeURIComponent(scopeKey)}?${search.toString()}`;
    return this.client.http.get<MetricsHistory>(url, RetryPolicy.Idempotent);
  }

  /**
   * Fetch per-wallet trading + referral aggregates for the authenticated
   * user: distinct outcomes traded, total USD volume across all the
   * wallet's trades, and the number of times the wallet's referral codes
   * have been redeemed. The wallet is resolved server-side from the
   * auth cookie.
   *
   * `GET /api/metrics/user`
   */
  async user(): Promise<UserMetrics> {
    const url = `${this.client.http.baseUrl()}/api/metrics/user`;
    return this.client.http.get<UserMetrics>(url, RetryPolicy.Idempotent);
  }

  /**
   * Same as {@link user} but uses the supplied `cookieHeader` for this call
   * instead of the SDK's process-wide cookie store. For server-side cookie
   * forwarding (SSR / route handlers).
   */
  async userWithCookies(cookieHeader: string): Promise<UserMetrics> {
    const url = `${this.client.http.baseUrl()}/api/metrics/user`;
    return this.client.http.getWithCookies<UserMetrics>(
      url,
      RetryPolicy.Idempotent,
      cookieHeader,
    );
  }

  /**
   * Public variant of {@link user}. Takes the user's wallet via the URL
   * path (`GET /api/metrics/user/{wallet_address}`) and requires no auth.
   */
  async userByWallet(walletAddress: string): Promise<UserMetrics> {
    const url = `${this.client.http.baseUrl()}/api/metrics/user/${encodeURIComponent(walletAddress)}`;
    return this.client.http.get<UserMetrics>(url, RetryPolicy.Idempotent);
  }
}
