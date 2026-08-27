import { RejectionCode } from "./rejection";
import type { LinkedIdentityType } from "../auth";

/** Raw backend rejection details before bounded codes and methods are decoded. */
export interface ApiRejectedDetailsWire {
  reason: string;
  rejection_code?: string;
  error_code?: string;
  /** Primary method of a deterministic identity owner; unknown values are ignored. */
  existing_method?: string;
  error_log_id?: string;
}

export type ApiResponse<T> =
  | { status: "success"; body: T }
  | { status: "error"; error_details: ApiRejectedDetailsWire };

/** Structured rejection details with correlation and transport context. */
export class ApiRejectedDetails {
  readonly reason: string;
  readonly rejectionCode?: RejectionCode;
  readonly errorCode?: string;
  readonly existingMethod?: LinkedIdentityType;
  readonly errorLogId?: string;
  readonly requestId?: string;
  /**
   * HTTP status of the response that carried this rejection. Set by the HTTP
   * client when the rejection rode a non-2xx response; not present in the
   * backend JSON. Lets callers classify rejections at the transport level
   * (e.g. `isUnauthorized`) without matching on backend error strings.
   */
  readonly httpStatus?: number;

  constructor(params: {
    reason: string;
    rejectionCode?: RejectionCode;
    errorCode?: string;
    existingMethod?: LinkedIdentityType;
    errorLogId?: string;
    requestId?: string;
    httpStatus?: number;
  }) {
    this.reason = params.reason;
    this.rejectionCode = params.rejectionCode;
    this.errorCode = params.errorCode;
    this.existingMethod = params.existingMethod;
    this.errorLogId = params.errorLogId;
    this.requestId = params.requestId;
    this.httpStatus = params.httpStatus;
  }

  static fromWire(
    wire: ApiRejectedDetailsWire,
    requestId?: string,
    httpStatus?: number,
  ): ApiRejectedDetails {
    return new ApiRejectedDetails({
      reason: wire.reason,
      rejectionCode: wire.rejection_code
        ? RejectionCode.from(wire.rejection_code)
        : undefined,
      errorCode: wire.error_code,
      existingMethod: linkedIdentityType(wire.existing_method),
      errorLogId: wire.error_log_id,
      requestId,
      httpStatus,
    });
  }

  toString(): string {
    const lines = [`Reason: ${this.reason}`];
    if (this.rejectionCode) {
      lines.push(`Rejection Code: ${this.rejectionCode.label()}`);
    }
    if (this.errorCode) {
      lines.push(`Error Code: ${this.errorCode}`);
    }
    if (this.existingMethod) {
      lines.push(`Existing Method: ${this.existingMethod}`);
    }
    if (this.errorLogId) {
      lines.push(`Error Log ID: ${this.errorLogId}`);
    }
    if (this.requestId) {
      lines.push(`Request ID: ${this.requestId}`);
    }
    return lines.join("\n");
  }
}

/** Preserve known public method guidance while tolerating future backend variants. */
function linkedIdentityType(
  value: string | undefined,
): LinkedIdentityType | undefined {
  return value === "email" ||
    value === "google" ||
    value === "x" ||
    value === "wallet"
    ? value
    : undefined;
}

export function isApiResponse<T>(value: unknown): value is ApiResponse<T> {
  if (typeof value !== "object" || value === null || !("status" in value)) {
    return false;
  }

  const status = (value as { status?: unknown }).status;
  if (status === "success") {
    return "body" in value;
  }
  if (status === "error") {
    return "error_details" in value;
  }
  return false;
}
