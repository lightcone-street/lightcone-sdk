export function displayFormattedString(input: string): string {
  const [rawInteger, rawFraction] = input.split(".");
  const negative = rawInteger.startsWith("-");
  const integer = negative ? rawInteger.slice(1) : rawInteger;
  const withCommas = integer.replace(/\B(?=(\d{3})+(?!\d))/g, ",");
  const fraction = rawFraction?.replace(/0+$/, "");
  const prefix = negative ? "-" : "";

  if (!fraction) {
    return `${prefix}${withCommas}`;
  }

  return `${prefix}${withCommas}.${fraction}`;
}

export function displayWithDecimals(value: number, decimals: number): string {
  return displayFormattedString(value.toFixed(decimals));
}

export function display(value: number): string {
  if (Math.abs(value) >= 100) {
    return displayWithDecimals(value, 0);
  }
  if (Math.abs(value) >= 1) {
    return displayWithDecimals(value, 2);
  }
  if (value === 0) {
    return "0";
  }

  const abs = Math.abs(value);
  const exponent = Math.floor(Math.log10(abs));
  const decimals = Math.min(Math.max(Math.abs(exponent) + 2, 2), 8);
  return displayWithDecimals(value, decimals);
}

export function toDecimalValue(value: bigint, decimals: number): number {
  return Number(value) / 10 ** decimals;
}

export function fromDecimalValue(value: number, decimals: number): bigint {
  return BigInt(Math.trunc(value * 10 ** decimals));
}
