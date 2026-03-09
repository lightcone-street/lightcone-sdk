/**
 * Parse a decimal string to a number.
 */
export function parseDecimal(value: string): number {
  const parsed = Number.parseFloat(value);
  if (Number.isNaN(parsed)) {
    throw new Error(`Invalid decimal string: ${value}`);
  }
  return parsed;
}

/**
 * Format a number to a fixed decimal string.
 */
export function formatDecimal(value: number, precision: number): string {
  return value.toFixed(precision);
}

/**
 * Whether a decimal string represents numeric zero.
 */
export function isZero(value: string): boolean {
  const parsed = Number.parseFloat(value);
  return !Number.isNaN(parsed) && parsed === 0;
}
