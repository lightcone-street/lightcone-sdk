import { displayDecimals, isFormattedZero } from "./constants";

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

export function display(value: number): string {
  const formatted = value.toFixed(displayDecimals(Math.abs(value)));
  return isFormattedZero(formatted) ? "0" : displayFormattedString(formatted);
}

export function toDecimalValue(value: bigint, decimals: number): number {
  return Number(value) / 10 ** decimals;
}

export function fromDecimalValue(value: number, decimals: number): bigint {
  return BigInt(Math.trunc(value * 10 ** decimals));
}
