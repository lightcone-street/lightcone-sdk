import { HttpError, SdkError } from "../error";
import { ApiRejectedDetails, isApiResponse } from "../shared/api_response";
import { delayForAttempt, retryConfigForPolicy, type RetryPolicy } from "./retry";

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
    return this.requestWithRetry<T>("GET", url, undefined, retry, this.adminCookieHeaders());
  }

  async adminPost<T, B extends object>(url: string, body: B, retry: RetryPolicy): Promise<T> {
    return this.requestWithRetry<T>("POST", url, body, retry, this.adminCookieHeaders());
  }

  private adminCookieHeaders(): Record<string, string> {
    if (hasBrowserWindow() || !this.adminToken) {
      return {};
    }
    return { Cookie: `admin_token=${this.adminToken}` };
  }

  async get<T>(url: string, retry: RetryPolicy): Promise<T> {
    return this.requestWithRetry<T>("GET", url, undefined, retry);
  }

  async post<T, B extends object>(url: string, body: B, retry: RetryPolicy): Promise<T> {
    return this.requestWithRetry<T>("POST", url, body, retry);
  }

  private async requestWithRetry<T>(
    method: "GET" | "POST",
    url: string,
    body: object | undefined,
    policy: RetryPolicy,
    extraHeaders?: Record<string, string>
  ): Promise<T> {
    const config = retryConfigForPolicy(policy);
    if (!config) {
      return this.doRequest<T>(method, url, body, extraHeaders);
    }

    let lastError: HttpError | undefined;

    for (let attempt = 0; attempt <= config.maxRetries; attempt += 1) {
      try {
        return await this.doRequest<T>(method, url, body, extraHeaders);
      } catch (error) {
        if (!(error instanceof HttpError)) {
          throw error;
        }

        const shouldRetry = this.shouldRetry(error, config.retryableStatuses);
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

  private shouldRetry(error: HttpError, retryableStatuses: readonly number[]): boolean {
    switch (error.variant) {
      case "Timeout":
      case "Request":
      case "RateLimited":
        return true;
      case "ServerError":
        return error.status !== undefined && retryableStatuses.includes(error.status);
      default:
        return false;
    }
  }

  private async doRequest<T>(
    method: "GET" | "POST",
    url: string,
    body?: object,
    extraHeaders?: Record<string, string>
  ): Promise<T> {
    const requestId = generateRequestId();
    const headers: Record<string, string> = {};

    if (body) {
      headers["Content-Type"] = "application/json";
    }
    headers["x-request-id"] = requestId;

    if (!hasBrowserWindow()) {
      const cookieParts: string[] = [];
      if (this.authToken) {
        cookieParts.push(`auth_token=${this.authToken}`);
      }
      if (extraHeaders?.Cookie) {
        cookieParts.push(extraHeaders.Cookie);
      }
      if (cookieParts.length > 0) {
        headers.Cookie = cookieParts.join("; ");
      }
    }

    if (extraHeaders) {
      for (const [key, value] of Object.entries(extraHeaders)) {
        if (key !== "Cookie") {
          headers[key] = value;
        }
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
      try {
        return parseResponsePayload<T>(JSON.parse(text), requestId);
      } catch (e) {
        throw HttpError.request(e instanceof Error ? e.message : "JSON parse failed");
      }
    }

    const bodyText = await response.text().catch(() => "");

    const statusCode = response.status;
    if (statusCode === 401) throw HttpError.unauthorized();
    if (statusCode === 404) throw HttpError.notFound(bodyText);
    if (statusCode === 429) throw HttpError.rateLimited();
    if (statusCode >= 400 && statusCode < 500) throw HttpError.badRequest(bodyText);
    throw HttpError.serverError(statusCode, bodyText);
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
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function parseResponsePayload<T>(payload: unknown, requestId: string): T {
  if (!isApiResponse<T>(payload)) {
    return payload as T;
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
