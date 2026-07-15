import { RejectionCode } from "./rejection";

export interface ApiRejectedDetailsWire {
  reason: string;
  rejection_code?: string;
  error_code?: string;
  error_log_id?: string;
}

export type ApiResponse<T> =
  | { status: "success"; body: T }
  | { status: "error"; error_details: ApiRejectedDetailsWire };

export class ApiRejectedDetails {
  readonly reason: string;
  readonly rejectionCode?: RejectionCode;
  readonly errorCode?: string;
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
    errorLogId?: string;
    requestId?: string;
    httpStatus?: number;
  }) {
    this.reason = params.reason;
    this.rejectionCode = params.rejectionCode;
    this.errorCode = params.errorCode;
    this.errorLogId = params.errorLogId;
    this.requestId = params.requestId;
    this.httpStatus = params.httpStatus;
  }

  static fromWire(
    wire: ApiRejectedDetailsWire,
    requestId?: string,
    httpStatus?: number
  ): ApiRejectedDetails {
    return new ApiRejectedDetails({
      reason: wire.reason,
      rejectionCode: wire.rejection_code
        ? RejectionCode.from(wire.rejection_code)
        : undefined,
      errorCode: wire.error_code,
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
    if (this.errorLogId) {
      lines.push(`Error Log ID: ${this.errorLogId}`);
    }
    if (this.requestId) {
      lines.push(`Request ID: ${this.requestId}`);
    }
    return lines.join("\n");
  }
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
