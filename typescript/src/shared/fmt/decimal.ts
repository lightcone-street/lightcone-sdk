import Decimal from "decimal.js";
import { displayFormattedString } from "./num";

export function display(value: Decimal): string {
  if (value.isZero()) {
    return "0";
  }

  const abs = value.abs();
  if (abs.greaterThanOrEqualTo(100)) {
    return displayFormattedString(value.toDecimalPlaces(0).toString());
  }
  if (abs.greaterThanOrEqualTo(1)) {
    return displayFormattedString(value.toDecimalPlaces(2).toString());
  }

  const asString = abs.toFixed(20).replace(/0+$/, "");
  const match = /^0\.(0+)(\d+)/.exec(asString);
  if (match && match[1].length > 5) {
    const sign = value.isNegative() ? "-" : "";
    const significant = match[2].slice(0, 4).replace(/0+$/, "");
    return `${sign}0.0(${match[1].length})${significant || "0"}`;
  }

  return displayFormattedString(value.toSignificantDigits(8).toString());
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
      return `${sign}${abs.div(threshold).toDecimalPlaces(digits).toString()}${suffix}`;
    }
  }

  return `${sign}${abs.toDecimalPlaces(digits).toString()}`;
}

export function toBaseUnits(value: Decimal, decimals: number): bigint | null {
  const scaled = value.mul(new Decimal(10).pow(decimals));
  if (!scaled.isInteger() || scaled.isNegative()) {
    return null;
  }
  return BigInt(scaled.toFixed(0));
}
