import { HttpError } from "../error";
import { delayForAttempt, retryConfigForPolicy, type RetryPolicy } from "./retry";

export class LightconeHttp {
  private readonly normalizedBaseUrl: string;
  private authToken: string | undefined;

  constructor(baseUrl: string) {
    this.normalizedBaseUrl = baseUrl.replace(/\/+$/, "");
  }

  baseUrl(): string {
    return this.normalizedBaseUrl;
  }

  async setAuthToken(token: string | undefined): Promise<void> {
    this.authToken = token;
  }

  async clearAuthToken(): Promise<void> {
    this.authToken = undefined;
  }

  authTokenRef(): () => Promise<string | undefined> {
    return async () => this.authToken;
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
    policy: RetryPolicy
  ): Promise<T> {
    const config = retryConfigForPolicy(policy);
    if (!config) {
      return this.doRequest<T>(method, url, body);
    }

    let lastError: HttpError | undefined;

    for (let attempt = 0; attempt <= config.maxRetries; attempt += 1) {
      try {
        return await this.doRequest<T>(method, url, body);
      } catch (error) {
        if (!(error instanceof HttpError)) {
          throw error;
        }

        const shouldRetry = this.shouldRetry(error, config.retryableStatuses);
        if (!shouldRetry || attempt >= config.maxRetries) {
          throw error;
        }

        lastError = error;
        const delay = error.variant === "RateLimited" && error.retryAfterMs !== undefined
          ? error.retryAfterMs
          : delayForAttempt(config, attempt);

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

  private async doRequest<T>(method: "GET" | "POST", url: string, body?: object): Promise<T> {
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
    };

    if (this.authToken && !hasBrowserWindow()) {
      headers.Cookie = `auth_token=${this.authToken}`;
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
        credentials: "include",
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
      const text = await response.text();
      if (!text) {
        return undefined as T;
      }
      return JSON.parse(text) as T;
    }

    const bodyText = await response.text().catch(() => "");

    switch (response.status) {
      case 400:
        throw HttpError.badRequest(bodyText);
      case 401:
        throw HttpError.unauthorized();
      case 404:
        throw HttpError.notFound(bodyText);
      case 429:
        throw HttpError.rateLimited(parseRetryAfterMs(response.headers.get("retry-after")));
      default:
        if (response.status >= 500) {
          throw HttpError.serverError(response.status, bodyText);
        }
        throw HttpError.badRequest(bodyText || `HTTP ${response.status}`);
    }
  }
}

function parseRetryAfterMs(header: string | null): number | undefined {
  if (!header) {
    return undefined;
  }

  const seconds = Number.parseFloat(header);
  if (Number.isFinite(seconds) && seconds >= 0) {
    return Math.floor(seconds * 1000);
  }

  const retryDate = Date.parse(header);
  if (Number.isFinite(retryDate)) {
    return Math.max(0, retryDate - Date.now());
  }

  return undefined;
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function hasBrowserWindow(): boolean {
  return typeof globalThis !== "undefined" && "window" in globalThis;
}
