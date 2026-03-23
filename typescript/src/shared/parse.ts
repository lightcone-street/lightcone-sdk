import { SdkError } from "../error";
import { TimeInForce } from "./types";

/**
 * Convert a Unix-millisecond timestamp to a Date.
 */
export function timestampMsToDate(ms: unknown): Date {
  if (typeof ms !== "number" || !Number.isFinite(ms)) {
    throw SdkError.validation(`Invalid timestamp: ${String(ms)}`);
  }
  return new Date(ms);
}

/**
 * Convert a numeric TimeInForce value (0–3) to the enum variant.
 */
export function tifFromNumeric(value: unknown): TimeInForce {
  switch (value) {
    case 0: return TimeInForce.Gtc;
    case 1: return TimeInForce.Ioc;
    case 2: return TimeInForce.Fok;
    case 3: return TimeInForce.Alo;
    default:
      throw SdkError.validation(`Invalid TimeInForce numeric value: ${String(value)}`);
  }
}

/**
 * Convert a numeric TimeInForce value to the enum, returning undefined for
 * null/undefined inputs.
 */
export function tifFromNumericOpt(value: unknown): TimeInForce | undefined {
  if (value === null || value === undefined) return undefined;
  return tifFromNumeric(value);
}

/**
 * Convert an empty string to undefined, pass through non-empty values.
 */
export function emptyStringAsUndefined<T>(value: T | ""): T | undefined {
  return value === "" ? undefined : value;
}
