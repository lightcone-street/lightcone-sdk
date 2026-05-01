/**
 * Resolve icon URL variants with fallback logic.
 *
 * Returns `null` when **all three** variants are missing (undefined/null/empty),
 * which signals a validation error. When at least one variant is available the
 * missing ones are filled in using a priority chain:
 *
 * - `low`  : try medium, then high
 * - `medium`: try low, then high
 * - `high` : try medium, then low
 */
export function resolveIconUrls(
  low: string | undefined | null,
  medium: string | undefined | null,
  high: string | undefined | null,
): { low: string; medium: string; high: string } | null {
  const any = low || medium || high;
  if (!any) return null;
  return {
    low: low || medium || high || any,
    medium: medium || low || high || any,
    high: high || medium || low || any,
  };
}
