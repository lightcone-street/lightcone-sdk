import Decimal from "decimal.js";
import { DISPLAY_DECIMAL_TIERS, SMALL_VALUE_DECIMALS, isFormattedZero } from "./constants";
import { displayFormattedString } from "./num";

function displayDecimals(value: Decimal): number {
  for (const [threshold, decimals] of DISPLAY_DECIMAL_TIERS) {
    if (value.greaterThanOrEqualTo(threshold)) {
      return decimals;
    }
  }

  return SMALL_VALUE_DECIMALS;
}

export function display(value: Decimal): string {
  const decimals = displayDecimals(value.abs());
  const formatted = value.toFixed(decimals);
  return isFormattedZero(formatted) ? "0" : displayFormattedString(formatted);
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
