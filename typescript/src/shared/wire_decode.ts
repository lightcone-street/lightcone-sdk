import { SdkError } from "../error";
import {
  asOrderBookId,
  asPubkeyStr,
  FundingSource,
  Side,
  type CommitInfo,
  type OrderBookId,
  type PubkeyStr,
} from "./types";

/**
 * Strict decoders for the committed trading backend's wire encodings.
 *
 * The backend sends exact integers as JSON strings on REST and in WebSocket
 * snapshots but as JSON numbers in live WebSocket facts, and an empty string
 * where a protobuf string field is unset. `parseJsonExact` keeps integer
 * tokens above 2^53 as `bigint`, so integer decoders accept a digit-only
 * string, a non-negative safe `number`, or a `bigint`.
 *
 * Every decoder throws `SdkError` (`Serde`) naming the field, so a payload
 * that does not match the contract fails loudly instead of decoding to a
 * silently wrong value.
 */

export type WireRecord = Record<string, unknown>;

const U64_MAX = (1n << 64n) - 1n;
const I128_MIN = -(1n << 127n);
const I128_MAX = (1n << 127n) - 1n;
const DIGITS = /^[0-9]+$/;
const DECIMAL = /^-?[0-9]+(\.[0-9]+)?$/;

/** True for plain decimal text (`"12.5"`, `"-0.000001"`); no exponent or padding. */
export function isDecimalText(text: string): boolean {
  return DECIMAL.test(text);
}

/** Decoding failure that names the offending field. */
export function wireError(field: string, message: string): SdkError {
  return SdkError.serde(`${field}: ${message}`);
}

function describe(value: unknown): string {
  if (typeof value === "string") return JSON.stringify(value);
  if (typeof value === "bigint") return `${value}`;
  if (value === null) return "null";
  if (Array.isArray(value)) return "array";
  if (typeof value === "object") return "object";
  return String(value);
}

/** A JSON object (not null, not an array). */
export function wireObject(value: unknown, field: string): WireRecord {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw wireError(field, `expected object, got ${describe(value)}`);
  }
  return value as WireRecord;
}

/** A JSON array decoded element by element; `null`/absent is `[]` when `defaultEmpty`. */
export function wireArray<T>(
  value: unknown,
  field: string,
  decode: (item: unknown, field: string) => T,
  defaultEmpty = false,
): T[] {
  if ((value === undefined || value === null) && defaultEmpty) return [];
  if (!Array.isArray(value)) {
    throw wireError(field, `expected array, got ${describe(value)}`);
  }
  return value.map((item, index) => decode(item, `${field}[${index}]`));
}

/** A required JSON string (empty allowed). */
export function wireString(value: unknown, field: string): string {
  if (typeof value !== "string") {
    throw wireError(field, `expected string, got ${describe(value)}`);
  }
  return value;
}

/** A JSON string, or `fallback` when the field is `null` or absent. */
export function wireStringOr(value: unknown, field: string, fallback: string): string {
  if (value === undefined || value === null) return fallback;
  return wireString(value, field);
}

/** An optional string; `null` or absent is `undefined` (an empty string is kept). */
export function wireOptString(value: unknown, field: string): string | undefined {
  return value === undefined || value === null ? undefined : wireString(value, field);
}

/** A base58 public key string (not decoded; the backend owns canonical form). */
export function wirePubkey(value: unknown, field: string): PubkeyStr {
  return asPubkeyStr(wireString(value, field));
}

/** An orderbook id string. */
export function wireOrderBookId(value: unknown, field: string): OrderBookId {
  return asOrderBookId(wireString(value, field));
}

/**
 * An optional string where `null`, absent, and `""` all mean unset. The
 * backend sends `""` for unset protobuf strings (closure reasons, snapshot
 * cursors, global-account market keys).
 */
export function wireNonEmptyString(value: unknown, field: string): string | undefined {
  if (value === undefined || value === null) return undefined;
  const text = wireString(value, field);
  return text === "" ? undefined : text;
}

/** A JSON boolean, or `fallback` when the field is `null` or absent. */
export function wireBool(value: unknown, field: string, fallback?: boolean): boolean {
  if ((value === undefined || value === null) && fallback !== undefined) return fallback;
  if (typeof value !== "boolean") {
    throw wireError(field, `expected boolean, got ${describe(value)}`);
  }
  return value;
}

function integerText(value: unknown, field: string, signed: boolean): bigint {
  if (typeof value === "bigint") return value;
  if (typeof value === "number") {
    if (!Number.isSafeInteger(value)) {
      throw wireError(field, `expected an exact integer, got ${describe(value)}`);
    }
    return BigInt(value);
  }
  if (typeof value === "string") {
    const digits = signed && value.startsWith("-") ? value.slice(1) : value;
    if (!DIGITS.test(digits)) {
      throw wireError(field, `invalid integer text ${describe(value)}`);
    }
    return BigInt(value);
  }
  throw wireError(field, `expected integer text or number, got ${describe(value)}`);
}

/** An exact `u64` sent as decimal text or a JSON number. */
export function wireU64(value: unknown, field: string): bigint {
  const parsed = integerText(value, field, false);
  if (parsed < 0n || parsed > U64_MAX) {
    throw wireError(field, `integer out of u64 range: ${parsed}`);
  }
  return parsed;
}

/** An optional `u64`; `null` or absent is `undefined`. */
export function wireOptU64(value: unknown, field: string): bigint | undefined {
  return value === undefined || value === null ? undefined : wireU64(value, field);
}

const I64_MIN = -(1n << 63n);
const I64_MAX = (1n << 63n) - 1n;

/** An exact signed `i64` sent as a JSON number (or integer text). */
export function wireI64(value: unknown, field: string): bigint {
  const parsed = integerText(value, field, true);
  if (parsed < I64_MIN || parsed > I64_MAX) {
    throw wireError(field, `integer out of i64 range: ${parsed}`);
  }
  return parsed;
}

/** An optional `i64`; `null` or absent is `undefined`. */
export function wireOptI64(value: unknown, field: string): bigint | undefined {
  return value === undefined || value === null ? undefined : wireI64(value, field);
}

/** An exact signed `i128` sent as decimal text (raw signed atoms). */
export function wireI128(value: unknown, field: string): bigint {
  const parsed = integerText(value, field, true);
  if (parsed < I128_MIN || parsed > I128_MAX) {
    throw wireError(field, `integer out of i128 range: ${parsed}`);
  }
  return parsed;
}

/** An optional `i128`; `null` or absent is `undefined`. */
export function wireOptI128(value: unknown, field: string): bigint | undefined {
  return value === undefined || value === null ? undefined : wireI128(value, field);
}

/**
 * A small JSON integer such as basis points or an index, within
 * `[min, max]`. Accepts integer text as well, which the backend uses for
 * some protobuf integers.
 */
export function wireInt(value: unknown, field: string, min: number, max: number): number {
  const parsed = integerText(value, field, true);
  if (parsed < BigInt(min) || parsed > BigInt(max)) {
    throw wireError(field, `integer out of range [${min}, ${max}]: ${parsed}`);
  }
  return Number(parsed);
}

/** `wireInt`, or `fallback` when the field is `null` or absent. */
export function wireIntOr(
  value: unknown,
  field: string,
  min: number,
  max: number,
  fallback: number,
): number {
  if (value === undefined || value === null) return fallback;
  return wireInt(value, field, min, max);
}

/**
 * A plain decimal string (`"12.5"`, `"-0.000001"`), kept as exact text.
 * JSON numbers, exponents, signs other than a leading `-`, and padding are
 * rejected: decimal.js and `Number` would accept forms the backend never
 * sends.
 */
export function wireDecimal(value: unknown, field: string): string {
  const text = wireString(value, field);
  if (!DECIMAL.test(text)) {
    throw wireError(field, `invalid decimal text ${describe(value)}`);
  }
  return text;
}

/** An optional decimal string; `null` or absent is `undefined`. */
export function wireOptDecimal(value: unknown, field: string): string | undefined {
  return value === undefined || value === null ? undefined : wireDecimal(value, field);
}

/** A decimal string, or `fallback` when the field is `null` or absent. */
export function wireDecimalOr(value: unknown, field: string, fallback: string): string {
  return value === undefined || value === null ? fallback : wireDecimal(value, field);
}

/**
 * `Side` from its text form (`"bid"`/`"ask"`, or `"buy"`/`"sell"`) or the
 * numeric engine encoding used by live committed facts (`0` = bid,
 * `1` = ask).
 */
export function wireSide(value: unknown, field: string): Side {
  if (value === "bid" || value === "buy") return Side.Bid;
  if (value === "ask" || value === "sell") return Side.Ask;
  if (value === 0 || value === 0n) return Side.Bid;
  if (value === 1 || value === 1n) return Side.Ask;
  throw wireError(field, `unknown side ${describe(value)}`);
}

/** A string enum value that must be one of `known`. */
export function wireEnum<T extends string>(value: unknown, field: string, known: readonly T[]): T {
  const text = wireString(value, field);
  if (!(known as readonly string[]).includes(text)) {
    throw wireError(field, `unknown value ${describe(value)}`);
  }
  return text as T;
}

/**
 * A string enum value mapped to `unknown` when this SDK version does not
 * know it, so a new backend value does not fail the surrounding payload.
 */
export function wireEnumOrUnknown<T extends string, U extends string>(
  value: unknown,
  field: string,
  known: readonly T[],
  unknown: U,
): T | U {
  const text = wireString(value, field);
  return (known as readonly string[]).includes(text) ? (text as T) : unknown;
}

/**
 * A string enum value, or `undefined` when it is `null`, absent, or a value
 * this SDK version does not know (such as the backend's `"unsupported"`).
 */
export function wireKnownOrUndefined<T extends string>(
  value: unknown,
  field: string,
  known: readonly T[],
): T | undefined {
  if (value === undefined || value === null) return undefined;
  const text = wireString(value, field);
  return (known as readonly string[]).includes(text) ? (text as T) : undefined;
}

/** Unix epoch milliseconds (`u64`) as a `Date`. */
export function wireTimestampMs(value: unknown, field: string): Date {
  const millis = wireU64(value, field);
  if (millis > 8_640_000_000_000_000n) {
    throw wireError(field, `timestamp out of range: ${millis}`);
  }
  return new Date(Number(millis));
}

/** An RFC 3339 timestamp string as a `Date`. */
export function wireRfc3339(value: unknown, field: string): Date {
  const text = wireString(value, field);
  const parsed = new Date(text);
  if (!/^\d{4}-\d{2}-\d{2}T/.test(text) || Number.isNaN(parsed.getTime())) {
    throw wireError(field, `invalid RFC 3339 timestamp ${describe(value)}`);
  }
  return parsed;
}

const FUNDING_SOURCES = [FundingSource.Global, FundingSource.Conditional] as const;

/** `FundingSource`, mapping a value this SDK version does not know to `Unknown`. */
export function wireFundingSource(value: unknown, field: string): FundingSource {
  return wireEnumOrUnknown(value, field, FUNDING_SOURCES, FundingSource.Unknown);
}

/** The commit metadata flattened into a live committed fact. */
export function wireCommitInfo(data: WireRecord, context: string): CommitInfo {
  return {
    effect_id: wireString(data.effect_id, `${context}.effect_id`),
    committed_revision: wireU64(data.committed_revision, `${context}.committed_revision`),
    projection_generation: wireU64(
      data.projection_generation,
      `${context}.projection_generation`,
    ),
    actionable: wireBool(data.actionable, `${context}.actionable`, false),
  };
}
