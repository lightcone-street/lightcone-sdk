import { ApiRejectedDetails } from "./shared/api_response";

export type HttpErrorVariant =
  | "Request"
  | "ServerError"
  | "RateLimited"
  | "Unauthorized"
  | "NotFound"
  | "BadRequest"
  | "Timeout"
  | "MaxRetriesExceeded";

export class HttpError extends Error {
  readonly variant: HttpErrorVariant;
  readonly status?: number;
  readonly body?: string;
  readonly retryAfterMs?: number;
  readonly attempts?: number;

  constructor(params: {
    variant: HttpErrorVariant;
    message: string;
    status?: number;
    body?: string;
    retryAfterMs?: number;
    attempts?: number;
  }) {
    super(params.message);
    this.name = "HttpError";
    this.variant = params.variant;
    this.status = params.status;
    this.body = params.body;
    this.retryAfterMs = params.retryAfterMs;
    this.attempts = params.attempts;
  }

  static request(message: string): HttpError {
    return new HttpError({ variant: "Request", message: `Request failed: ${message}` });
  }

  static timeout(): HttpError {
    return new HttpError({ variant: "Timeout", message: "Timeout" });
  }

  static unauthorized(): HttpError {
    return new HttpError({ variant: "Unauthorized", message: "Unauthorized", status: 401 });
  }

  static notFound(body: string): HttpError {
    return new HttpError({
      variant: "NotFound",
      message: `Not found: ${body || "resource"}`,
      status: 404,
      body,
    });
  }

  static badRequest(body: string): HttpError {
    return new HttpError({
      variant: "BadRequest",
      message: `Bad request: ${body || "invalid request"}`,
      status: 400,
      body,
    });
  }

  static rateLimited(retryAfterMs?: number): HttpError {
    return new HttpError({
      variant: "RateLimited",
      message:
        retryAfterMs !== undefined
          ? `Rate limited (retry after ${retryAfterMs}ms)`
          : "Rate limited",
      status: 429,
      retryAfterMs,
    });
  }

  static serverError(status: number, body: string): HttpError {
    return new HttpError({
      variant: "ServerError",
      message: `Server error ${status}: ${body}`,
      status,
      body,
    });
  }

  /**
   * NOT produced by the SDK's HTTP retry loop. On retry exhaustion the FINAL
   * attempt's error propagates unchanged — structured rejection details,
   * status classification (isUnauthorized etc.), and request id intact —
   * because flattening it into this wrapper's message string would destroy
   * everything callers switch on (see the retry-exhaustion tests). The
   * factory stays public for consumers that build their own retry loops on
   * the raw/no-restore primitives and want a conventional exhaustion error.
   */
  static maxRetriesExceeded(attempts: number, lastError: string): HttpError {
    return new HttpError({
      variant: "MaxRetriesExceeded",
      message: `Max retries exceeded after ${attempts} attempts: ${lastError}`,
      attempts,
    });
  }
}

export type WsErrorVariant =
  | "NotConnected"
  | "ConnectionFailed"
  | "SendFailed"
  | "DeserializationError"
  | "ProtocolError"
  | "Closed";

export class WsError extends Error {
  readonly variant: WsErrorVariant;
  readonly code?: number;

  constructor(variant: WsErrorVariant, message: string, code?: number) {
    super(message);
    this.name = "WsError";
    this.variant = variant;
    this.code = code;
  }
}

export type AuthErrorVariant =
  | "NotAuthenticated"
  | "LoginFailed"
  | "SignatureVerificationFailed"
  | "TokenExpired";

export class AuthError extends Error {
  readonly variant: AuthErrorVariant;

  constructor(variant: AuthErrorVariant, message: string) {
    super(message);
    this.name = "AuthError";
    this.variant = variant;
  }
}

export type SdkErrorVariant =
  | "Http"
  | "Ws"
  | "Auth"
  | "Validation"
  | "InsufficientSolForTransactionFees"
  | "Serde"
  | "MissingMarketContext"
  | "Signing"
  | "UserCancelled"
  | "TransactionFailed"
  | "TransactionExpired"
  | "ConfirmationTimeout"
  | "ApiRejected"
  | "Program"
  | "Other";

export class SdkError extends Error {
  readonly variant: SdkErrorVariant;
  readonly causeError?: Error;
  readonly apiRejectedDetails?: ApiRejectedDetails;
  /** Transaction signature, set on the transaction-confirmation variants. */
  readonly signature?: string;
  /** Confirmed Native SOL Balance in the declared fee payer, in lamports. */
  readonly availableLamports?: bigint;
  /** Exact transaction fee or planner-owned reserve required, in lamports. */
  readonly requiredLamports?: bigint;

  constructor(
    variant: SdkErrorVariant,
    message: string,
    causeError?: Error,
    apiRejectedDetails?: ApiRejectedDetails,
    signature?: string,
    availableLamports?: bigint,
    requiredLamports?: bigint
  ) {
    super(message);
    this.name = "SdkError";
    this.variant = variant;
    this.causeError = causeError;
    this.apiRejectedDetails = apiRejectedDetails;
    this.signature = signature;
    this.availableLamports = availableLamports;
    this.requiredLamports = requiredLamports;
  }

  static from(error: unknown): SdkError {
    if (error instanceof SdkError) {
      return error;
    }
    if (error instanceof ApiRejectedDetails) {
      return SdkError.apiRejected(error);
    }
    if (error instanceof HttpError) {
      return new SdkError("Http", error.message, error);
    }
    if (error instanceof WsError) {
      return new SdkError("Ws", error.message, error);
    }
    if (error instanceof AuthError) {
      return new SdkError("Auth", error.message, error);
    }
    if (error instanceof SyntaxError) {
      return new SdkError("Serde", error.message, error);
    }
    // ProgramSdkError is imported lazily to avoid circular deps;
    // duck-type check on .name instead.
    if (error instanceof Error && error.name === "ProgramSdkError") {
      return new SdkError("Program", error.message, error);
    }
    if (error instanceof Error) {
      return new SdkError("Other", error.message, error);
    }
    if (typeof error === "object" && error !== null) {
      return new SdkError("Other", JSON.stringify(error));
    }
    return new SdkError("Other", String(error));
  }

  static validation(message: string): SdkError {
    return new SdkError("Validation", message);
  }

  /** Return the typed contract for a proven transaction-fee funding shortfall. */
  static insufficientSolForTransactionFees(
    availableLamports: bigint,
    requiredLamports: bigint
  ): SdkError {
    if (availableLamports < 0n || requiredLamports < 0n) {
      return SdkError.validation(
        "transaction fee funding values must be non-negative lamports"
      );
    }
    return new SdkError(
      "InsufficientSolForTransactionFees",
      "Insufficient SOL for transaction fees. Deposit SOL to your wallet and try again.",
      undefined,
      undefined,
      undefined,
      availableLamports,
      requiredLamports
    );
  }

  static serde(message: string): SdkError {
    return new SdkError("Serde", message);
  }

  static missingMarketContext(message: string): SdkError {
    return new SdkError("MissingMarketContext", message);
  }

  static signing(message: string): SdkError {
    return new SdkError("Signing", message);
  }

  static userCancelled(): SdkError {
    return new SdkError("UserCancelled", "User cancelled signing");
  }

  static transactionFailed(signature: string, error: string): SdkError {
    return new SdkError(
      "TransactionFailed",
      `Transaction ${signature} failed on-chain: ${error}`,
      undefined,
      undefined,
      signature
    );
  }

  static transactionExpired(signature: string): SdkError {
    return new SdkError(
      "TransactionExpired",
      `Transaction ${signature} expired before confirmation — it was never processed and is safe to resubmit`,
      undefined,
      undefined,
      signature
    );
  }

  static confirmationTimeout(signature: string): SdkError {
    return new SdkError(
      "ConfirmationTimeout",
      `Timed out confirming transaction ${signature} — status unknown; check the signature on-chain before resubmitting`,
      undefined,
      undefined,
      signature
    );
  }

  static apiRejected(details: ApiRejectedDetails): SdkError {
    return new SdkError("ApiRejected", details.toString(), undefined, details);
  }
}

/**
 * True when the backend rejected a request as unauthenticated (HTTP 401) —
 * either a bare 401 (`HttpError` with variant `Unauthorized`) or a 401 that
 * carried a structured rejection envelope (`SdkError` `ApiRejected` with an
 * `httpStatus` of 401). Lets callers decide whether refreshing credentials
 * and retrying makes sense without matching on backend error strings.
 * Accepts `unknown` because the transport surfaces both `HttpError` and
 * `SdkError` to callers.
 */
export function isUnauthorized(error: unknown): boolean {
  if (error instanceof HttpError) {
    return error.variant === "Unauthorized";
  }
  if (error instanceof SdkError) {
    if (error.variant === "ApiRejected") {
      return error.apiRejectedDetails?.httpStatus === 401;
    }
    if (error.variant === "Http" && error.causeError instanceof HttpError) {
      return error.causeError.variant === "Unauthorized";
    }
  }
  return false;
}
