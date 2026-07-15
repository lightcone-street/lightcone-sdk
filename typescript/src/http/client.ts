import { HttpError, SdkError } from "../error";
import {
  ApiRejectedDetails,
  isApiResponse,
  type ApiResponse,
} from "../shared/api_response";
import type { CredentialRestorer } from "./credentialRestorer";
import {
  delayForAttempt,
  retryConfigForPolicy,
  type RetryConfig,
  type RetryPolicy,
} from "./retry";

type AuthMode =
  | { kind: "cookie" }
  | { kind: "cookieOverride"; cookieHeader: string };

const DEFAULT_HTTP_TIMEOUT_MS = 180_000;

class HttpStatusError extends Error {
  readonly status: number;
  readonly body: string;
  readonly headers: Headers;
  readonly requestId: string;

  constructor(status: number, body: string, headers: Headers, requestId: string) {
    super(`HTTP status ${status}: ${body}`);
    this.name = "HttpStatusError";
    this.status = status;
    this.body = body;
    this.headers = headers;
    this.requestId = requestId;
  }
}

/** Zero-retry config for `RetryPolicy.None`: requests still run through the
 * retry loop so the credential-restore path covers every request, including
 * non-idempotent POSTs. */
const NO_RETRY_CONFIG: RetryConfig = {
  maxRetries: 0,
  initialDelayMs: 0,
  maxDelayMs: 0,
  backoffFactor: 1,
  jitter: false,
  retryableStatuses: [],
};

/** Upper bound on waiting for (or running) a credential restoration. Also the
 * rescue path for restorer reentrancy: a restorer whose own restore-enabled
 * SDK call ends up awaiting its own restoration degrades to a propagated 401
 * after this bound instead of deadlocking. */
const CREDENTIAL_RESTORE_TIMEOUT_MS = 30_000;

export class LightconeHttp {
  private readonly normalizedBaseUrl: string;
  private authToken: string | undefined;
  private credentialRestorer: CredentialRestorer | undefined;
  /**
   * The in-flight restoration, shared by every request that 401s while it
   * runs — concurrent 401s await the same promise (bounded by the timeout)
   * instead of failing fast. Cleared when the restoration settles.
   */
  private restorationPromise: Promise<boolean> | null = null;
  /**
   * Abort handle for the in-flight restoration, set and cleared with
   * `restorationPromise`. Fired when the restoration's deadline abandons it:
   * promises cannot be cancelled, so this is the signal a well-behaved
   * restorer uses to stop instead of running on concurrently with the next
   * restoration.
   */
  private restorationAbort: AbortController | null = null;
  /**
   * The restoration's own deadline, created WITH the restoration and shared
   * by every waiter. Racing each waiter against its own fresh timer instead
   * would let a late joiner outlive the leader's timeout and accept a `true`
   * from a zombie restorer while a replacement restoration is already
   * running.
   */
  private restorationDeadline: Promise<boolean> | null = null;
  /**
   * Bumped after every completed restoration. A request captures the epoch
   * when it starts; if the epoch moved by the time its 401 gets to restore,
   * another request already restored and the stored outcome is reused.
   */
  private restorationEpoch = 0;
  private lastRestoreOutcome = false;
  /** Overridable for tests (hung-restorer coverage at test speed). */
  private credentialRestoreTimeoutMs = CREDENTIAL_RESTORE_TIMEOUT_MS;

  constructor(baseUrl: string) {
    this.normalizedBaseUrl = baseUrl.replace(/\/+$/, "");
  }

  /**
   * Run — or share — a credential restoration for a logical request that
   * began at `startEpoch`. Returns whether replaying is worthwhile. Restorer
   * rejections count as "not restored" so the caller's original 401 is
   * preserved (a callback failure must not replace the auth error).
   */
  private async restoreCredentialsShared(startEpoch: number): Promise<boolean> {
    if (this.restorationEpoch !== startEpoch) {
      // Another request completed a restoration after this one began — its
      // outcome applies to our credentials too.
      return this.lastRestoreOutcome;
    }
    if (!this.restorationPromise) {
      const restorer = this.credentialRestorer;
      if (!restorer) {
        return false;
      }
      const controller = new AbortController();
      const tracked: Promise<boolean> = (async () => {
        try {
          return await restorer(controller.signal);
        } catch (error) {
          console.warn("Credential restorer threw; propagating the original 401", error);
          return false;
        }
      })().then((outcome) => {
        // Only record the outcome if this restoration still owns the slot —
        // an abandoned (timed-out) restoration settling late must not bump
        // the epoch or clobber a newer restoration.
        if (this.restorationPromise === tracked) {
          this.restorationEpoch += 1;
          this.lastRestoreOutcome = outcome;
          this.restorationPromise = null;
          this.restorationAbort = null;
          this.restorationDeadline = null;
        }
        return outcome;
      });
      this.restorationPromise = tracked;
      this.restorationAbort = controller;
      this.restorationDeadline = new Promise<boolean>((resolve) => {
        const timer = setTimeout(() => resolve(false), this.credentialRestoreTimeoutMs);
        // Node: don't hold the process open for the bound; no-op in browsers.
        (timer as { unref?: () => void }).unref?.();
      });
    }
    // Bound the wait with the RESTORATION's deadline (shared by all waiters),
    // not a per-waiter timer: a hung restorer (or a restorer awaiting itself
    // through a restore-enabled nested call) fails every waiting request with
    // its original 401 at the same moment, and no waiter is left listening to
    // an abandoned restoration after that.
    const awaited = this.restorationPromise;
    const deadline = this.restorationDeadline ?? Promise.resolve(false);
    const outcome = await Promise.race([awaited, deadline]);
    if (!outcome && this.restorationPromise === awaited) {
      // Timed out with the restoration still pending (a settled restoration
      // clears the slot itself): abort it — promises can't be cancelled, so
      // the signal is how a well-behaved restorer stops instead of racing the
      // next restoration — and drop the slot so the client isn't stuck
      // "restoring" forever. The next 401 starts a fresh restoration.
      // Identity-guarded: concurrent waiters all observe the same deadline,
      // and only the first one through here does the teardown.
      this.restorationAbort?.abort();
      this.restorationPromise = null;
      this.restorationAbort = null;
      this.restorationDeadline = null;
    }
    return outcome;
  }

  /**
   * True when `url` shares the configured API origin. Session credentials are
   * only injected — and the credential restorer only consulted — for
   * same-origin requests: a foreign URL that answers 401 must be able to
   * neither trigger a restoration nor receive the restored cookie on a
   * replay. Unparseable URLs count as foreign.
   */
  private isApiOrigin(url: string): boolean {
    try {
      return new URL(url).origin === new URL(this.normalizedBaseUrl).origin;
    } catch {
      return false;
    }
  }

  baseUrl(): string {
    return this.normalizedBaseUrl;
  }

  async clearAuthToken(): Promise<void> {
    this.authToken = undefined;
  }

  /**
   * Register (or replace) the credential restorer consulted on HTTP 401 —
   * see {@link CredentialRestorer}. Pass through
   * `LightconeClient.setCredentialRestorer` in normal use.
   */
  setCredentialRestorer(restorer: CredentialRestorer): void {
    this.credentialRestorer = restorer;
  }

  /** Remove the credential restorer; 401s propagate to callers unchanged. */
  clearCredentialRestorer(): void {
    this.credentialRestorer = undefined;
  }

  authTokenRef(): () => Promise<string | undefined> {
    return async () => this.authToken;
  }

  async get<T>(url: string, retry: RetryPolicy): Promise<T> {
    return this.requestWithRetry<T>("GET", url, undefined, retry, { kind: "cookie" });
  }

  async post<T, B extends object>(url: string, body: B, retry: RetryPolicy): Promise<T> {
    return this.requestWithRetry<T>("POST", url, body, retry, { kind: "cookie" });
  }

  /**
   * POST with the 401 credential-restore-and-replay disabled. For
   * credential-management endpoints (login, logout): they are the machinery
   * restoration would re-run, so replaying them after restoring credentials
   * is at best a no-op and at worst re-consumes single-use state (the login
   * nonce is consumed server-side before the signature is verified, so a
   * replayed login deterministically fails).
   */
  async postWithoutCredentialRestore<T, B extends object>(
    url: string,
    body: B,
    retry: RetryPolicy
  ): Promise<T> {
    return this.requestWithRetry<T>("POST", url, body, retry, { kind: "cookie" }, false);
  }

  /**
   * GET with retry, forwarding an explicit per-call raw `Cookie` header
   * (e.g. `privy-token=…; lightcone-token=…`) instead of the SDK's process-wide
   * cookie store. Intended for server-side cookie forwarding (SSR / server
   * functions) where the per-request browser cookies can't propagate to the
   * shared client. In a browser context this is equivalent to {@link get} — the
   * runtime is already attaching cookies via `credentials: "include"`.
   */
  async getWithCookies<T>(url: string, retry: RetryPolicy, cookieHeader: string): Promise<T> {
    return this.requestWithRetry<T>(
      "GET",
      url,
      undefined,
      retry,
      { kind: "cookieOverride", cookieHeader },
      // Forwarded per-user cookies are outside the global credential
      // machinery: the process-wide restorer can't mint a new cookie for
      // THIS user, and a replay would resend the same stale header.
      false
    );
  }

  /**
   * GET with the 401 credential restoration disabled. Use this (and
   * {@link postWithoutCredentialRestore}) inside credential restorers for any
   * SDK calls the restorer itself makes: a restore-enabled call from inside a
   * restorer ends up awaiting its own restoration and only the restoration
   * timeout rescues it.
   */
  async getWithoutCredentialRestore<T>(url: string, retry: RetryPolicy): Promise<T> {
    return this.requestWithRetry<T>("GET", url, undefined, retry, { kind: "cookie" }, false);
  }

  private async requestWithRetry<T>(
    method: "GET" | "POST",
    url: string,
    body: object | undefined,
    policy: RetryPolicy,
    authMode: AuthMode,
    allowCredentialRestore = true
  ): Promise<T> {
    // `None` still means "no transport retries"; it runs through the same
    // loop with zero retry attempts so the credential-restore path below
    // covers every request. It ALSO means "never auto-replay": mutations
    // declare themselves non-idempotent via RetryPolicy.None, so a 401 still
    // triggers restoration (healing the session for the caller's next
    // attempt) but the 401 propagates instead of replaying.
    const replayAllowed = policy.kind !== "none";
    const config = retryConfigForPolicy(policy) ?? NO_RETRY_CONFIG;

    // One request id and one body serialization per LOGICAL request:
    // transport retries and the auth replay resend the same id (tracing
    // correlation, and the hook for future server-side idempotency) and the
    // same bytes (a mutable body can't drift between attempts).
    const requestId = generateRequestId();
    const bodyText = body === undefined ? undefined : JSON.stringify(body);

    const startEpoch = this.restorationEpoch;
    let credentialsRestored = false;
    let attempt = 0;

    for (;;) {
      try {
        const apiResponse = await this.sendRequest<ApiResponse<T>>(
          method,
          url,
          bodyText,
          authMode,
          requestId
        );
        return parseApiResponse<T>(apiResponse, requestId);
      } catch (error) {
        if (error instanceof HttpStatusError) {
          // On the first 401, give the host a chance to restore its
          // credentials (e.g. refresh an auth session) — at most once per
          // logical request, shared with any concurrent requests (see
          // restoreCredentialsShared), and only for requests to the API
          // origin whose endpoint allows it (login/logout opt out). The
          // replay itself additionally requires the request to have
          // declared itself retry-safe.
          if (
            !credentialsRestored &&
            allowCredentialRestore &&
            error.status === 401 &&
            this.isApiOrigin(url)
          ) {
            credentialsRestored = true;
            const restored = await this.restoreCredentialsShared(startEpoch);
            if (restored && replayAllowed) {
              continue;
            }
          }

          const shouldRetry = config.retryableStatuses.includes(error.status);
          if (!shouldRetry || attempt >= config.maxRetries) {
            throw this.statusErrorToSdk(error);
          }

          const retryAfter = retryAfterMs(error.headers);
          const delay = retryAfter ?? delayForAttempt(config, attempt);
          attempt += 1;

          await sleep(delay);
          continue;
        }

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

        const delay = delayForAttempt(config, attempt);
        attempt += 1;

        await sleep(delay);
      }
    }
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
        return retryableStatuses.includes(429);
      case "ServerError":
        return error.status !== undefined && retryableStatuses.includes(error.status);
      default:
        return false;
    }
  }

  private async sendRequest<T>(
    method: "GET" | "POST",
    url: string,
    bodyText: string | undefined,
    authMode: AuthMode,
    requestId: string
  ): Promise<T> {
    const headers: Record<string, string> = {};

    if (bodyText !== undefined) {
      headers["Content-Type"] = "application/json";
    }
    headers["x-request-id"] = requestId;

    // Cookie injection is origin-gated: session credentials only ride to the
    // configured API origin, never to an arbitrary absolute URL a caller
    // supplies. In a browser the runtime owns cookie scoping instead.
    if (!hasBrowserWindow() && this.isApiOrigin(url)) {
      const cookie = this.cookieHeader(authMode);
      if (cookie) {
        headers.Cookie = cookie;
      }
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), DEFAULT_HTTP_TIMEOUT_MS);

    let response: Response;

    try {
      response = await fetch(url, {
        method,
        headers,
        body: bodyText,
        signal: controller.signal,
        // The API never legitimately redirects; following one would let a
        // redirect target observe the request while origin checks still see
        // the original URL. A 3xx therefore surfaces as an error below.
        redirect: "manual",
        // Browser credentials mode is origin-gated like the Cookie header
        // above: the browser's own cookie scoping already keeps the API
        // cookie off foreign origins, this just avoids asking it to attach
        // ANY cookies to a URL the SDK shouldn't be talking to. The foreign
        // branch is an explicit "omit" because the fetch default is
        // "same-origin" — a URL foreign to the API but matching the PAGE
        // origin would still get that origin's cookies.
        ...(hasBrowserWindow()
          ? {
              credentials: (this.isApiOrigin(url)
                ? "include"
                : "omit") as RequestCredentials,
            }
          : {}),
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
      // Capture only for the built-in session: a cookieOverride request
      // carries one specific user's forwarded cookies, and capturing its
      // Set-Cookie into the process-wide token slot would leak that user's
      // rotated token to every later request from a shared server client
      // (the python SDK has always skipped this; typescript now matches).
      if (!hasBrowserWindow() && authMode.kind !== "cookieOverride") {
        this.captureCookies(response);
      }

      const text = await response.text();
      let payload: T;
      try {
        payload = JSON.parse(text) as T;
      } catch (e) {
        throw HttpError.request(e instanceof Error ? e.message : "JSON parse failed");
      }

      return payload;
    }

    const errorBody = await response.text().catch(() => "");
    throw new HttpStatusError(response.status, errorBody, response.headers, requestId);
  }

  private captureCookies(response: Response): void {
    for (const cookieHeader of getSetCookieHeaders(response.headers)) {
      const authToken = extractCookieValue(cookieHeader, "lightcone-token");
      if (authToken) {
        this.authToken = authToken;
      }
    }
  }

  private cookieHeader(authMode: AuthMode): string | undefined {
    if (hasBrowserWindow()) {
      return undefined;
    }

    switch (authMode.kind) {
      case "cookieOverride":
        // Forward the supplied Cookie header verbatim (may carry privy-token
        // and/or lightcone-token).
        return authMode.cookieHeader;
      case "cookie":
        return this.authToken ? `lightcone-token=${this.authToken}` : undefined;
    }
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

  private statusErrorToSdk(error: HttpStatusError): Error {
    const rejected = parseRejectedBody(error.body, error.requestId, error.status);
    if (rejected) {
      return rejected;
    }
    return this.mapStatusError(error.status, error.body, error.headers);
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

function parseRejectedBody(
  body: string,
  requestId: string,
  httpStatus: number
): SdkError | undefined {
  let payload: unknown;
  try {
    payload = JSON.parse(body);
  } catch {
    return undefined;
  }

  if (!isApiResponse<unknown>(payload) || payload.status !== "error") {
    return undefined;
  }

  return SdkError.apiRejected(
    ApiRejectedDetails.fromWire(payload.error_details, requestId, httpStatus)
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
