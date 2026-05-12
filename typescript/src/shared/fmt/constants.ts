export const DISPLAY_DECIMAL_TIERS = [
  [10_000, 0],
  [1_000, 1],
  [100, 2],
  [10, 3],
  [0.1, 4],
] as const;

export const SMALL_VALUE_DECIMALS = 5;

export function displayDecimalsBy(matchesTier: (threshold: number) => boolean): number {
  for (const [threshold, decimals] of DISPLAY_DECIMAL_TIERS) {
    if (matchesTier(threshold)) {
      return decimals;
    }
  }

  return SMALL_VALUE_DECIMALS;
}

export function isFormattedZero(input: string): boolean {
  return /^-?0(?:\.0+)?$/.test(input);
}
