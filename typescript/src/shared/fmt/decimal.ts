import Decimal from "decimal.js";
import {
  DEFAULT_DECIMALS,
  MAX_STANDARD_DECIMALS,
  SUBSCRIPT_SIGNIFICANT_DIGITS,
  TINY_SIGNIFICANT_DIGITS,
} from "./constants";
import { displayFormattedString } from "./num";

function trimTrailingFractionZeros(input: string): string {
  if (!input.includes(".")) {
    return input;
  }
  return input.replace(/0+$/, "").replace(/\.$/, "");
}

function tinyParts(value: Decimal): { leadingZeros: number; significant: string } {
  const [coefficient, exponentText] = value
    .toExponential(SUBSCRIPT_SIGNIFICANT_DIGITS - 1)
    .split("e");
  const exponent = Number(exponentText);
  const leadingZeros = Math.max(-exponent - 1, 0);
  const significant = coefficient.replace(".", "").replace(/0+$/, "") || "0";
  return { leadingZeros, significant };
}

export function display(value: Decimal): string {
  if (value.isZero()) {
    return displayFormattedString(value.toFixed(DEFAULT_DECIMALS));
  }

  const abs = value.abs();
  if (!abs.toDecimalPlaces(DEFAULT_DECIMALS).isZero()) {
    return displayFormattedString(value.toFixed(DEFAULT_DECIMALS));
  }

  const { leadingZeros, significant } = tinyParts(abs);
  if (leadingZeros + 1 > MAX_STANDARD_DECIMALS) {
    const sign = value.isNegative() ? "-" : "";
    return `${sign}0.0(${leadingZeros})${significant}`;
  }

  const decimals = Math.min(leadingZeros + TINY_SIGNIFICANT_DIGITS, MAX_STANDARD_DECIMALS);
  return displayFormattedString(trimTrailingFractionZeros(value.toFixed(decimals)));
}

export function abbrNumber(value: Decimal, digits = 2, showSign = true): string {
  const sign = showSign && value.isNegative() ? "-" : "";
  const abs = value.abs();

  const units: Array<[Decimal, string]> = [
    [new Decimal("1000000000000"), "t"],
    [new Decimal("1000000000"), "b"],
    [new Decimal("1000000"), "m"],
    [new Decimal("1000"), "k"],
  ];

  for (const [threshold, suffix] of units) {
    if (abs.greaterThanOrEqualTo(threshold)) {
      return `${sign}${abs.div(threshold).toFixed(digits)}${suffix}`;
    }
  }

  return `${sign}${abs.toFixed(digits)}`;
}

export function toBaseUnits(value: Decimal, decimals: number): bigint | null {
  const scaled = value.mul(new Decimal(10).pow(decimals));
  if (!scaled.isInteger() || scaled.isNegative()) {
    return null;
  }
  return BigInt(scaled.toFixed(0));
}
