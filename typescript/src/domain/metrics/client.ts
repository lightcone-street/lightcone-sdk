import type { ClientContext } from "../../context";
import { RetryPolicy } from "../../http";
import type { OrderBookId, PubkeyStr } from "../../shared";
import type {
  CategoriesMetrics,
  CategoryVolumeMetrics,
  DepositTokensMetrics,
  Leaderboard,
  MarketDetailMetrics,
  MarketsMetrics,
  MetricsHistory,
  MetricsHistoryQuery,
  OrderbookTickersResponse,
  OrderbookVolumeMetrics,
  PlatformMetrics,
  UserMetrics,
} from "./wire";

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
   * Batch BBO + midpoint per active orderbook (same shape as the WS
   * `Ticker` stream, delivered in one REST call). Optionally filter to
   * orderbooks whose base conditional-token is backed by `depositAsset`.
   * Prices per orderbook are scaled using that orderbook's own decimals.
   *
   * `GET /api/metrics/orderbooks/tickers[?deposit_asset=<mint>]`
   */
  async orderbookTickers(depositAsset?: string): Promise<OrderbookTickersResponse> {
    let url = `${this.client.http.baseUrl()}/api/metrics/orderbooks/tickers`;
    const trimmed = depositAsset?.trim();
    if (trimmed) {
      url += `?deposit_asset=${encodeURIComponent(trimmed)}`;
    }
    return this.client.http.get<OrderbookTickersResponse>(url, RetryPolicy.Idempotent);
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
    search.set("resolution", query.resolution ?? "1h");
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
   * `auth_token` cookie.
   *
   * `GET /api/metrics/user`
   */
  async user(): Promise<UserMetrics> {
    const url = `${this.client.http.baseUrl()}/api/metrics/user`;
    return this.client.http.get<UserMetrics>(url, RetryPolicy.Idempotent);
  }

  /**
   * Same as {@link user} but uses the supplied `authToken` for this call
   * instead of the SDK's process-wide cookie store. For server-side cookie
   * forwarding (SSR / route handlers).
   */
  async userWithAuth(authToken: string): Promise<UserMetrics> {
    const url = `${this.client.http.baseUrl()}/api/metrics/user`;
    return this.client.http.getWithAuth<UserMetrics>(
      url,
      RetryPolicy.Idempotent,
      authToken,
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
