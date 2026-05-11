import { DEFAULT_DECIMALS, MAX_STANDARD_DECIMALS, TINY_SIGNIFICANT_DIGITS } from "./constants";

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

function trimTrailingFractionZeros(input: string): string {
  if (!input.includes(".")) {
    return input;
  }
  return input.replace(/0+$/, "").replace(/\.$/, "");
}

function leadingZeroCount(value: number): number {
  const exponent = Math.floor(Math.log10(Math.abs(value)));
  return Math.max(-exponent - 1, 0);
}

function displaySubscript(value: number, leadingZeros: number): string {
  const sign = value < 0 ? "-" : "";
  const scaled = Math.abs(value) * 10 ** (leadingZeros + 1);
  const significant = trimTrailingFractionZeros(scaled.toFixed(3)).replace(".", "");
  return `${sign}0.0(${leadingZeros})${significant}`;
}

export function display(value: number): string {
  const abs = Math.abs(value);
  if (abs !== 0 && abs < 0.005) {
    const leadingZeros = leadingZeroCount(abs);
    if (leadingZeros + 1 > MAX_STANDARD_DECIMALS) {
      return displaySubscript(value, leadingZeros);
    }

    const decimals = Math.min(leadingZeros + TINY_SIGNIFICANT_DIGITS, MAX_STANDARD_DECIMALS);
    return displayFormattedString(trimTrailingFractionZeros(value.toFixed(decimals)));
  }

  return displayWithDecimals(value, DEFAULT_DECIMALS);
}

export function toDecimalValue(value: bigint, decimals: number): number {
  return Number(value) / 10 ** decimals;
}

export function fromDecimalValue(value: number, decimals: number): bigint {
  return BigInt(Math.trunc(value * 10 ** decimals));
}
