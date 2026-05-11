import {
  SUBSCRIPT_SIGNIFICANT_DIGITS,
  displayFormat,
  trimTrailingFractionZeros,
} from "./constants";

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

function leadingZeroCount(value: number): number {
  const exponent = Math.floor(Math.log10(Math.abs(value)));
  return Math.max(-exponent - 1, 0);
}

function displaySubscript(value: number, leadingZeros: number): string {
  const sign = value < 0 ? "-" : "";
  const scaled = Math.abs(value) * 10 ** (leadingZeros + 1);
  let significant = Math.trunc(scaled * 10 ** (SUBSCRIPT_SIGNIFICANT_DIGITS - 1));
  while (significant > 0 && significant % 10 === 0) {
    significant = Math.trunc(significant / 10);
  }
  return `${sign}0.0(${leadingZeros})${significant}`;
}

export function display(value: number): string {
  const abs = Math.abs(value);
  const leadingZeros = abs === 0 ? 0 : leadingZeroCount(abs);
  const format = displayFormat({
    isZero: abs === 0,
    roundsToDefaultNonzero: abs >= 0.005,
    leadingZeros,
  });

  if (format.kind === "subscript") {
    return displaySubscript(value, leadingZeros);
  }

  const formatted = value.toFixed(format.decimals);
  return displayFormattedString(
    format.trimTrailingZeros ? trimTrailingFractionZeros(formatted) : formatted,
  );
}

export function toDecimalValue(value: bigint, decimals: number): number {
  return Number(value) / 10 ** decimals;
}

export function fromDecimalValue(value: number, decimals: number): bigint {
  return BigInt(Math.trunc(value * 10 ** decimals));
}
