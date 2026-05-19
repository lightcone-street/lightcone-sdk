import { displayDecimalsBy, isFormattedZero } from "./constants";

export function displayFormattedString(input: string): string {
  const [rawInteger, rawFraction] = input.split(".");
  const negative = rawInteger.startsWith("-");
  const integer = negative ? rawInteger.slice(1) : rawInteger;
  const withCommas = integer.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  const fraction = rawFraction;
  const prefix = negative ? "-" : "";

  if (fraction === undefined) {
    return `${prefix}${withCommas}`;
  }

  return `${prefix}${withCommas}.${fraction}`;
}

export function displayWithDecimals(value: number, decimals: number): string {
  return displayFormattedString(value.toFixed(decimals));
}

function displayDecimals(absValue: number): number {
  return displayDecimalsBy((threshold) => absValue >= Number(threshold));
}

export function display(value: number): string {
  const formatted = value.toFixed(displayDecimals(Math.abs(value)));
  return isFormattedZero(formatted) ? "0" : displayFormattedString(formatted);
}

export function toDecimalValue(value: bigint, decimals: number): number {
  return Number(value) / 10 ** decimals;
}

/**
 * Format a number as a percentage with exactly 2 decimal places (truncated).
 *
 * When padding is true (default), always shows 2 decimal places (e.g. "12.30").
 * When false, trailing zeros are trimmed (e.g. "12.3").
 */
export function displayPct(value: number, padding?: boolean): string {
  const pad = padding ?? true;
  const truncated = Math.trunc(value * 100) / 100;

  if (pad) {
    return displayFormattedString(truncated.toFixed(2));
  } else {
    const formatted = truncated.toFixed(2);
    const trimmed = formatted.replace(/\.?0+$/, "");
    return displayFormattedString(trimmed);
  }
}

export function fromDecimalValue(value: number, decimals: number): bigint {
  return BigInt(Math.trunc(value * 10 ** decimals));
}
