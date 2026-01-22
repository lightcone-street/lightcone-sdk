/**
 * API error types for the Lightcone REST API client.
 */

/**
 * API error variants
 */
export type ApiErrorVariant =
  | "Http"
  | "NotFound"
  | "BadRequest"
  | "Forbidden"
  | "Conflict"
  | "ServerError"
  | "Deserialize"
  | "InvalidParameter"
  | "UnexpectedStatus"
  | "RateLimited"
  | "Unauthorized";

/**
 * API-specific error class for the Lightcone REST API client.
 */
export class ApiError extends Error {
  readonly variant: ApiErrorVariant;
  readonly statusCode?: number;
  readonly details?: string;

  constructor(variant: ApiErrorVariant, message: string, statusCode?: number, details?: string) {
    super(message);
    this.name = "ApiError";
    this.variant = variant;
    this.statusCode = statusCode;
    this.details = details;
  }

  /** HTTP/network error */
  static http(message: string): ApiError {
    return new ApiError("Http", `HTTP error: ${message}`);
  }

  /** Resource not found (404) */
  static notFound(message: string): ApiError {
    return new ApiError("NotFound", `Not found: ${message}`, 404);
  }

  /** Invalid request parameters (400) */
  static badRequest(message: string): ApiError {
    return new ApiError("BadRequest", `Bad request: ${message}`, 400);
  }

  /** Permission denied, signature mismatch (403) */
  static forbidden(message: string): ApiError {
    return new ApiError("Forbidden", `Permission denied: ${message}`, 403);
  }

  /** Resource already exists (409) */
  static conflict(message: string): ApiError {
    return new ApiError("Conflict", `Conflict: ${message}`, 409);
  }

  /** Server-side error (500) */
  static serverError(message: string): ApiError {
    return new ApiError("ServerError", `Server error: ${message}`, 500);
  }

  /** JSON deserialization error */
  static deserialize(message: string): ApiError {
    return new ApiError("Deserialize", `Deserialization error: ${message}`);
  }

  /** Invalid parameter provided */
  static invalidParameter(message: string): ApiError {
    return new ApiError("InvalidParameter", `Invalid parameter: ${message}`);
  }

  /** Unexpected HTTP status code */
  static unexpectedStatus(statusCode: number, message: string): ApiError {
    return new ApiError("UnexpectedStatus", `Unexpected status ${statusCode}: ${message}`, statusCode);
  }

  /** Rate limited (429) */
  static rateLimited(message: string): ApiError {
    return new ApiError("RateLimited", `Rate limited: ${message}`, 429);
  }

  /** Unauthorized - invalid or missing authentication (401) */
  static unauthorized(message: string): ApiError {
    return new ApiError("Unauthorized", `Unauthorized: ${message}`, 401);
  }

  /** Create from HTTP status code */
  static fromStatus(statusCode: number, message: string): ApiError {
    switch (statusCode) {
      case 401:
        return ApiError.unauthorized(message);
      case 400:
        return ApiError.badRequest(message);
      case 403:
        return ApiError.forbidden(message);
      case 404:
        return ApiError.notFound(message);
      case 409:
        return ApiError.conflict(message);
      case 429:
        return ApiError.rateLimited(message);
      case 500:
      case 502:
      case 503:
      case 504:
        return ApiError.serverError(message);
      default:
        return ApiError.unexpectedStatus(statusCode, message);
    }
  }
}

/**
 * Result type alias for API operations.
 */
export type ApiResult<T> = T;

/**
 * Error response format from the API.
 */
export interface ErrorResponse {
  /** Error status (usually "error") */
  status?: string;
  /** Human-readable error message */
  message?: string;
  /** Alternative error field */
  error?: string;
  /** Additional error details */
  details?: string;
}

/**
 * Get the error message from an ErrorResponse.
 */
export function getErrorMessage(response: ErrorResponse): string {
  return response.message || response.error || response.details || "Unknown error";
}
