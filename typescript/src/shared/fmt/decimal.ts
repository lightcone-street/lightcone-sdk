import Decimal from "decimal.js";
import {
  DEFAULT_DECIMALS,
  SUBSCRIPT_SIGNIFICANT_DIGITS,
  displayFormat,
  trimTrailingFractionZeros,
} from "./constants";
import { displayFormattedString } from "./num";

function tinyParts(value: Decimal): { leadingZeros: number; significant: string } {
  const [, fraction = ""] = value.toFixed().split(".");
  const leadingZeros = /^0*/.exec(fraction)?.[0].length ?? 0;
  const significant =
    fraction.slice(leadingZeros, leadingZeros + SUBSCRIPT_SIGNIFICANT_DIGITS).replace(/0+$/, "") ||
    "0";
  return { leadingZeros, significant };
}

export function display(value: Decimal): string {
  const abs = value.abs();
  const { leadingZeros, significant } = tinyParts(abs);
  const format = displayFormat({
    isZero: value.isZero(),
    roundsToDefaultNonzero: !abs.toDecimalPlaces(DEFAULT_DECIMALS).isZero(),
    leadingZeros,
  });

  if (format.kind === "subscript") {
    const sign = value.isNegative() ? "-" : "";
    return `${sign}0.0(${leadingZeros})${significant}`;
  }

  const formatted = value.toFixed(format.decimals);
  return displayFormattedString(
    format.trimTrailingZeros ? trimTrailingFractionZeros(formatted) : formatted,
  );
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
