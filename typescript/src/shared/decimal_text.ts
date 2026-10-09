import Decimal from "decimal.js";

import type { Branded } from "./types";
import { wireError, wireString } from "./wire_decode";

/**
 * An exact, canonical decimal string such as `"12.500000"` or `"-0.000001"`.
 *
 * The committed backend formats some aggregates (funding reservations,
 * signed remainders) from `u128`/`i128` atom counts, which can exceed what
 * a JS `number` or a 96-bit decimal holds. They stay validated text; use
 * {@link decimalTextToDecimal} for arithmetic.
 */
export type DecimalText = Branded<string, "DecimalText">;

/** Validate canonical decimal text: no sign but a leading `-`, no exponent, no `-0`. */
export function parseDecimalText(text: string): DecimalText {
  const unsigned = text.startsWith("-") ? text.slice(1) : text;
  const [whole, fraction, ...rest] = unsigned.split(".");
  const digits = (part: string | undefined) => part !== undefined && /^[0-9]+$/.test(part);
  if (rest.length > 0 || !digits(whole) || (fraction !== undefined && !digits(fraction))) {
    throw new RangeError(`invalid decimal text: ${JSON.stringify(text)}`);
  }
  if (text.startsWith("-") && /^[0.]+$/.test(unsigned)) {
    throw new RangeError(`negative zero is not canonical: ${JSON.stringify(text)}`);
  }
  return text as DecimalText;
}

/** True when the value is negative (`-0` is never accepted). */
export function decimalTextIsNegative(text: DecimalText): boolean {
  return text.startsWith("-");
}

/** The exact value as a `Decimal` (construction never rounds). */
export function decimalTextToDecimal(text: DecimalText): Decimal {
  return new Decimal(text);
}

/** Decode a required `DecimalText` wire field; JSON numbers are rejected. */
export function wireDecimalText(value: unknown, field: string): DecimalText {
  const text = wireString(value, field);
  try {
    return parseDecimalText(text);
  } catch (error) {
    throw wireError(field, error instanceof Error ? error.message : String(error));
  }
}

/** Decode an optional `DecimalText` wire field; `null` or absent is `undefined`. */
export function wireOptDecimalText(value: unknown, field: string): DecimalText | undefined {
  return value === undefined || value === null ? undefined : wireDecimalText(value, field);
}
