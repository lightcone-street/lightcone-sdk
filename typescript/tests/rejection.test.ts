import { describe, it } from "node:test";
import assert from "node:assert/strict";

import {
  ApiRejectedDetails,
  ErrorCode,
  RejectionCode,
  type ApiResponse,
  type KnownErrorCode,
  type KnownRejectionCode,
} from "../src/shared";
import { parseJsonExact } from "../src/shared/json";

const REJECTION_CODES: KnownRejectionCode[] = [
  "DUPLICATE_ORDER",
  "INSUFFICIENT_BALANCE",
  "FOK_INSUFFICIENT_LIQUIDITY",
  "SELF_TRADE",
  "INVALID_ORDER",
  "INVALID_REQUEST",
  "ORDER_EXPIRED",
  "TRADING_PAUSED",
  "INVALID_SIGNATURE",
  "CANCEL_ALL_REPLAY",
  "STALE_STATE",
  "REQUEST_EXPIRED",
  "TRADING_CAPACITY_UNAVAILABLE",
  "TRADING_NOT_READY",
  "INTERNAL_ERROR",
  "ORDER_NOT_FOUND",
  "ORDERBOOK_NOT_FOUND",
];

const ERROR_CODES: KnownErrorCode[] = [
  "INVALID_ARGUMENT",
  "FAILED_PRECONDITION",
  "NOT_FOUND",
  "FORBIDDEN",
  "ALREADY_EXISTS",
  "ABORTED",
  "RESOURCE_EXHAUSTED",
  "ENGINE_UNAVAILABLE",
  "ENGINE_INTERNAL_ERROR",
  "TRADING_UNAVAILABLE",
  "AUTH_REQUIRED",
  "AUTH_WALLET_MISMATCH",
  "INVALID_TIF",
  "INVALID_DEPOSIT_SOURCE",
  "INVALID_SIGNATURE",
  "INVALID_PUBKEY",
  "INVALID_CURSOR",
  "INVALID_ORDER_HASH",
  "UNEXPECTED_QUERY",
  "INVALID_LIMIT",
  "RATE_LIMITED",
  "ORDERBOOK_DECIMALS_NOT_FOUND",
  "ORDERBOOK_CONFIGURATION_ERROR",
  "NON_CANONICAL_ENCODING",
];

function rejectedDetails(json: string): ApiRejectedDetails {
  const response = parseJsonExact<ApiResponse<unknown>>(json);
  assert.equal(response.status, "error");
  if (response.status !== "error") throw new Error("expected an error envelope");
  return ApiRejectedDetails.fromWire(response.error_details);
}

describe("RejectionCode", () => {
  it("matches any case format", () => {
    assert.equal(RejectionCode.from("INSUFFICIENT_BALANCE").known(), "INSUFFICIENT_BALANCE");
    assert.equal(RejectionCode.from("insufficient_balance").known(), "INSUFFICIENT_BALANCE");
  });

  it("keeps an unknown code's original text", () => {
    const code = RejectionCode.from("new_thing");
    assert.equal(code.isKnown(), false);
    assert.equal(code.label(), "new_thing");
    assert.equal(code.wireName(), "new_thing");
  });

  it("decodes retired codes as unknown", () => {
    for (const removed of [
      "NONCE_MISMATCH",
      "INVALID_NONCE",
      "FOK_NO_FILL",
      "BELOW_MIN_ORDER_SIZE",
      "EXPIRED",
      "PRICE_NOT_EXACTLY_REPRESENTABLE",
    ]) {
      const code = RejectionCode.from(removed);
      assert.equal(code.isKnown(), false, removed);
      assert.equal(code.label(), removed);
    }
  });

  it("labels known codes", () => {
    assert.equal(RejectionCode.from("INSUFFICIENT_BALANCE").label(), "Insufficient Balance");
    assert.equal(RejectionCode.from("SELF_TRADE").label(), "Self Trade");
    assert.equal(RejectionCode.from("FOK_INSUFFICIENT_LIQUIDITY").label(), "FOK Insufficient Liquidity");
    assert.equal(String(RejectionCode.from("INSUFFICIENT_BALANCE")), "Insufficient Balance");
  });

  it("round-trips every engine code", () => {
    for (const wire of REJECTION_CODES) {
      const code = RejectionCode.from(wire);
      assert.ok(code.isKnown(), wire);
      assert.equal(code.wireName(), wire);
      assert.notEqual(code.label(), wire);
    }
  });

  it("treats only the retryable admission failures as transient", () => {
    assert.ok(RejectionCode.from("STALE_STATE").isTransient());
    assert.ok(RejectionCode.from("TRADING_NOT_READY").isTransient());
    assert.ok(!RejectionCode.from("DUPLICATE_ORDER").isTransient());
    assert.ok(!RejectionCode.from("INSUFFICIENT_BALANCE").isTransient());
    assert.ok(!RejectionCode.from("UNKNOWN_FUTURE").isTransient());
  });
});

describe("ErrorCode", () => {
  it("round-trips every code and tolerates unknown values", () => {
    for (const wire of ERROR_CODES) {
      const code = ErrorCode.from(wire);
      assert.equal(code.known(), wire);
      assert.equal(code.wireName(), wire);
    }
    const unknown = ErrorCode.from("WALLET_NOT_AUTHORIZED");
    assert.equal(unknown.isKnown(), false);
    assert.equal(unknown.wireName(), "WALLET_NOT_AUTHORIZED");
    // Error codes match exactly; lowercase is not the wire form.
    assert.equal(ErrorCode.from("engine_unavailable").isKnown(), false);
    assert.ok(ErrorCode.from("ENGINE_UNAVAILABLE").isRetryable());
    assert.ok(!ErrorCode.from("AUTH_WALLET_MISMATCH").isRetryable());
  });

  it("types engine error codes from the envelope", () => {
    for (const code of [
      "ABORTED",
      "RESOURCE_EXHAUSTED",
      "ENGINE_UNAVAILABLE",
      "TRADING_UNAVAILABLE",
      "AUTH_WALLET_MISMATCH",
      "FORBIDDEN",
      "ALREADY_EXISTS",
      "INVALID_ARGUMENT",
    ] as const) {
      const details = rejectedDetails(
        `{"status":"error","error_details":{"reason":"Engine temporarily unavailable","error_code":"${code}","error_log_id":"LCERR_1"}}`,
      );
      assert.equal(details.errorCode, code);
      assert.equal(details.errorCodeKind()?.known(), code);
    }
  });

  it("carries a business rejection on a 200 envelope without an error code", () => {
    const details = rejectedDetails(
      '{"status":"error","error_details":{"reason":"The full original BASE target is not executable","rejection_code":"FOK_INSUFFICIENT_LIQUIDITY","error_log_id":"LCERR_2"}}',
    );
    assert.equal(details.rejectionCode?.known(), "FOK_INSUFFICIENT_LIQUIDITY");
    assert.equal(details.errorCodeKind(), undefined);
  });
});
