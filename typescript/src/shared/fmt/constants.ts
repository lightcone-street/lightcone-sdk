export const DEFAULT_DECIMALS = 2;
export const TINY_SIGNIFICANT_DIGITS = 3;
export const MAX_STANDARD_DECIMALS = 8;
export const SUBSCRIPT_SIGNIFICANT_DIGITS = 4;

export type DisplayFormat =
  | { kind: "standard"; decimals: number; trimTrailingZeros: boolean }
  | { kind: "subscript" };

export function displayFormat({
  isZero,
  roundsToDefaultNonzero,
  leadingZeros,
}: {
  isZero: boolean;
  roundsToDefaultNonzero: boolean;
  leadingZeros: number;
}): DisplayFormat {
  if (isZero || roundsToDefaultNonzero) {
    return { kind: "standard", decimals: DEFAULT_DECIMALS, trimTrailingZeros: false };
  }

  if (leadingZeros + 1 > MAX_STANDARD_DECIMALS) {
    return { kind: "subscript" };
  }

  return {
    kind: "standard",
    decimals: Math.min(leadingZeros + TINY_SIGNIFICANT_DIGITS, MAX_STANDARD_DECIMALS),
    trimTrailingZeros: true,
  };
}

export function trimTrailingFractionZeros(input: string): string {
  if (!input.includes(".")) {
    return input;
  }
  return input.replace(/0+$/, "").replace(/\.$/, "");
}
