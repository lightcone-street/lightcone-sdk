import { HttpError } from "../error";
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
    const headers: Record<string, string> = {};

    if (body) {
      headers["Content-Type"] = "application/json";
    }

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
      const { Cookie: _cookie, ...rest } = extraHeaders;
      Object.assign(headers, rest);
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
        const cookieHeader = response.headers.get("set-cookie") ?? "";
        for (const part of cookieHeader.split(",")) {
          const trimmed = part.trim();
          if (trimmed.startsWith("auth_token=")) {
            const token = trimmed.slice("auth_token=".length).split(";")[0];
            if (token) {
              this.authToken = token;
            }
          }
          if (trimmed.startsWith("admin_token=")) {
            const token = trimmed.slice("admin_token=".length).split(";")[0];
            if (token) {
              this.adminToken = token;
            }
          }
        }
      }

      const text = await response.text();
      try {
        return JSON.parse(text) as T;
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
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function hasBrowserWindow(): boolean {
  return typeof globalThis !== "undefined" && "window" in globalThis;
}
