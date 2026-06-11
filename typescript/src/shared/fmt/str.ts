/**
 * Shorten a string to its first and last `qty / 2` characters joined by an
 * ellipsis — e.g. `shorten("FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR", 8)`
 * → `"FRGk...WcPR"`. Strings of `qty` characters or fewer are returned
 * unchanged.
 */
export function shorten(value: string, qty: number): string {
  if (value.length > qty) {
    const charsToShow = Math.floor(qty / 2);
    return `${value.slice(0, charsToShow)}...${value.slice(value.length - charsToShow)}`;
  }
  return value;
}
