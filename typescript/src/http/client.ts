import { HttpError, SdkError } from "../error";
import {
  ApiRejectedDetails,
  isApiResponse,
  type ApiResponse,
} from "../shared/api_response";
import { delayForAttempt, retryConfigForPolicy, type RetryPolicy } from "./retry";

type AuthMode = "cookie" | "adminCookie";

export class LightconeHttp {
  private readonly normalizedBaseUrl: string;
  private authToken: string | undefined;
  private adminToken: string | undefined;

  constructor(baseUrl: string) {
    this.normalizedBaseUrl = baseUrl.replace(/\/+$/, "");
  }

  baseUrl(): string {
    return this.normalizedBaseUrl;
  }

  async clearAuthToken(): Promise<void> {
    this.authToken = undefined;
  }

  authTokenRef(): () => Promise<string | undefined> {
    return async () => this.authToken;
  }

  clearAdminToken(): void {
    this.adminToken = undefined;
  }

  async adminGet<T>(url: string, retry: RetryPolicy): Promise<T> {
    return this.requestWithRetry<T>(
      "GET",
      url,
      undefined,
      retry,
      "adminCookie"
    );
  }

  async adminPost<T, B extends object>(url: string, body: B, retry: RetryPolicy): Promise<T> {
    return this.requestWithRetry<T>("POST", url, body, retry, "adminCookie");
  }

  async get<T>(url: string, retry: RetryPolicy): Promise<T> {
    return this.requestWithRetry<T>("GET", url, undefined, retry, "cookie");
  }

  async post<T, B extends object>(url: string, body: B, retry: RetryPolicy): Promise<T> {
    return this.requestWithRetry<T>("POST", url, body, retry, "cookie");
  }

  private async requestWithRetry<T>(
    method: "GET" | "POST",
    url: string,
    body: object | undefined,
    policy: RetryPolicy,
    authMode: AuthMode
  ): Promise<T> {
    const config = retryConfigForPolicy(policy);
    if (!config) {
      return this.sendAndParse<T>(method, url, body, authMode);
    }

    let lastError: HttpError | undefined;

    for (let attempt = 0; attempt <= config.maxRetries; attempt += 1) {
      try {
        const [apiResponse, requestId] = await this.sendRequest<ApiResponse<T>>(
          method,
          url,
          body,
          authMode
        );
        return parseApiResponse<T>(apiResponse, requestId);
      } catch (error) {
        if (!(error instanceof HttpError)) {
          throw error;
        }

        const shouldRetry = await this.shouldRetry(
          error,
          config.retryableStatuses
        );
        if (!shouldRetry || attempt >= config.maxRetries) {
          throw error;
        }

        lastError = error;
        const delay = delayForAttempt(config, attempt);

        await sleep(delay);
      }
    }

    throw HttpError.maxRetriesExceeded(config.maxRetries + 1, lastError?.message ?? "unknown");
  }

  private async shouldRetry(
    error: HttpError,
    retryableStatuses: readonly number[]
  ): Promise<boolean> {
    switch (error.variant) {
      case "Timeout":
      case "Request":
        return true;
      case "RateLimited":
        if (error.retryAfterMs !== undefined) {
          await sleep(error.retryAfterMs);
        }
        return true;
      case "ServerError":
        return error.status !== undefined && retryableStatuses.includes(error.status);
      default:
        return false;
    }
  }

  private async sendAndParse<T>(
    method: "GET" | "POST",
    url: string,
    body?: object,
    authMode: AuthMode = "cookie"
  ): Promise<T> {
    const [apiResponse, requestId] = await this.sendRequest<ApiResponse<T>>(
      method,
      url,
      body,
      authMode
    );
    return parseApiResponse<T>(apiResponse, requestId);
  }

  private async sendRequest<T>(
    method: "GET" | "POST",
    url: string,
    body?: object,
    authMode: AuthMode = "cookie"
  ): Promise<[T, string]> {
    const requestId = generateRequestId();
    const headers: Record<string, string> = {};

    if (body) {
      headers["Content-Type"] = "application/json";
    }
    headers["x-request-id"] = requestId;

    if (!hasBrowserWindow()) {
      const cookie = this.cookieHeader(authMode);
      if (cookie) {
        headers.Cookie = cookie;
      }
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 30_000);

    let response: Response;

    try {
      response = await fetch(url, {
        method,
        headers,
        body: body ? JSON.stringify(body) : undefined,
        signal: controller.signal,
        ...(hasBrowserWindow() ? { credentials: "include" as RequestCredentials } : {}),
      });
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof Error && error.name === "AbortError") {
        throw HttpError.timeout();
      }
      throw HttpError.request(error instanceof Error ? error.message : String(error));
    } finally {
      clearTimeout(timeoutId);
    }

    if (response.ok) {
      if (!hasBrowserWindow()) {
        this.captureCookies(response);
      }

      const text = await response.text();
      let payload: T;
      try {
        payload = JSON.parse(text) as T;
      } catch (e) {
        throw HttpError.request(e instanceof Error ? e.message : "JSON parse failed");
      }

      return [payload, requestId];
    }

    const bodyText = await response.text().catch(() => "");
    throw this.mapStatusError(response.status, bodyText, response.headers);
  }

  private captureCookies(response: Response): void {
    for (const cookieHeader of getSetCookieHeaders(response.headers)) {
      const authToken = extractCookieValue(cookieHeader, "auth_token");
      if (authToken) {
        this.authToken = authToken;
      }

      const adminToken = extractCookieValue(cookieHeader, "admin_token");
      if (adminToken) {
        this.adminToken = adminToken;
      }
    }
  }

  private cookieHeader(authMode: AuthMode): string | undefined {
    if (hasBrowserWindow()) {
      return undefined;
    }

    if (authMode === "adminCookie") {
      return this.adminToken ? `admin_token=${this.adminToken}` : undefined;
    }

    return this.authToken ? `auth_token=${this.authToken}` : undefined;
  }

  private mapStatusError(statusCode: number, bodyText: string, headers: Headers): HttpError {
    if (statusCode === 401) {
      return HttpError.unauthorized();
    }
    if (statusCode === 404) {
      return HttpError.notFound(bodyText);
    }
    if (statusCode === 429) {
      return HttpError.rateLimited(retryAfterMs(headers));
    }
    if (statusCode >= 400 && statusCode < 500) {
      return HttpError.badRequest(bodyText);
    }
    return HttpError.serverError(statusCode, bodyText);
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function parseApiResponse<T>(payload: unknown, requestId: string): T {
  if (!isApiResponse<T>(payload)) {
    throw SdkError.serde("Invalid ApiResponse envelope");
  }

  if (payload.status === "success") {
    return payload.body;
  }

  throw SdkError.apiRejected(
    ApiRejectedDetails.fromWire(payload.error_details, requestId)
  );
}

function generateRequestId(): string {
  if (typeof globalThis.crypto?.randomUUID === "function") {
    return globalThis.crypto.randomUUID();
  }

  return `lc-${Date.now()}-${Math.random().toString(16).slice(2)}`;
}

function getSetCookieHeaders(headers: Headers): string[] {
  const headersWithCookies = headers as Headers & {
    getSetCookie?: () => string[];
  };

  if (typeof headersWithCookies.getSetCookie === "function") {
    const values = headersWithCookies.getSetCookie();
    if (values.length > 0) {
      return values;
    }
  }

  const combined = headers.get("set-cookie");
  return combined ? [combined] : [];
}

function retryAfterMs(headers: Headers): number | undefined {
  const retryAfterMsValue = headers.get("retry-after-ms");
  if (retryAfterMsValue) {
    const parsed = Number.parseInt(retryAfterMsValue, 10);
    if (Number.isFinite(parsed)) {
      return parsed;
    }
  }

  const retryAfterValue = headers.get("retry-after");
  if (retryAfterValue) {
    const parsed = Number.parseFloat(retryAfterValue);
    if (Number.isFinite(parsed)) {
      return Math.round(parsed * 1000);
    }
  }

  return undefined;
}

function extractCookieValue(header: string, name: string): string | undefined {
  const match = header.match(
    new RegExp(`(?:^|,\\s*)${escapeRegExp(name)}=([^;,]+)`)
  );
  return match?.[1];
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function hasBrowserWindow(): boolean {
  return typeof globalThis !== "undefined" && "window" in globalThis;
}
