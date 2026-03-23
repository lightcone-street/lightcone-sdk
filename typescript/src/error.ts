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

export type SdkErrorVariant = "Http" | "Ws" | "Auth" | "Validation" | "Serde" | "MissingMarketContext" | "Signing" | "UserCancelled" | "Program" | "Other";

export class SdkError extends Error {
  readonly variant: SdkErrorVariant;
  readonly causeError?: Error;

  constructor(variant: SdkErrorVariant, message: string, causeError?: Error) {
    super(message);
    this.name = "SdkError";
    this.variant = variant;
    this.causeError = causeError;
  }

  static from(error: unknown): SdkError {
    if (error instanceof SdkError) {
      return error;
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
}
