/**
 * Hyperliquid-style book aggregation parameters.
 *
 * `{}` (both fields absent) is full precision. `nSigFigs` must be 2, 3, 4,
 * or 5; `mantissa` must be 1, 2, or 5 and is only valid with `nSigFigs: 5`.
 * `{ nSigFigs: 5 }` normalizes to `{ nSigFigs: 5, mantissa: 1 }` — the
 * backend treats them as the same subscription, so all key/matching logic
 * compares normalized values.
 *
 * Property names use the wire spelling shared with subscribe params and REST
 * query keys (camelCase `nSigFigs`). Incoming `book_update` frames tag their
 * view with snake_case `n_sig_figs`/`mantissa` (omitted = full precision);
 * use {@link aggregationFromFrame} to key per-`(orderbook, aggregation)`
 * state from those tags.
 */
export interface BookAggregation {
  nSigFigs?: number;
  mantissa?: number;
}

/** Full precision (no aggregation). */
export const FULL_PRECISION: BookAggregation = {};

/**
 * Validate aggregation params against the backend's contract, returning the
 * normalized form. Throws on invalid combinations — the server rejects them
 * with `INVALID_ORDERBOOK_SUBSCRIPTION` (WS) or HTTP 400 (REST), so validate
 * before sending.
 */
export function validateAggregation(aggregation: BookAggregation): BookAggregation {
  const { nSigFigs, mantissa } = aggregation;
  if (nSigFigs === undefined) {
    if (mantissa !== undefined) {
      throw new Error("mantissa is only valid when nSigFigs is 5");
    }
    return {};
  }
  if (nSigFigs >= 2 && nSigFigs <= 4) {
    if (mantissa !== undefined) {
      throw new Error("mantissa is only valid when nSigFigs is 5");
    }
    return { nSigFigs };
  }
  if (nSigFigs === 5) {
    if (mantissa === undefined) {
      return { nSigFigs: 5, mantissa: 1 };
    }
    if (mantissa === 1 || mantissa === 2 || mantissa === 5) {
      return { nSigFigs: 5, mantissa };
    }
    throw new Error("mantissa must be 1, 2, or 5");
  }
  throw new Error("nSigFigs must be 2, 3, 4, 5, or omitted");
}

/**
 * Normalized form: `{ nSigFigs: 5 }` becomes `{ nSigFigs: 5, mantissa: 1 }`;
 * everything else is unchanged. Lenient — never throws. Use
 * {@link validateAggregation} to reject invalid combinations before sending.
 */
export function normalizeAggregation(aggregation: BookAggregation): BookAggregation {
  if (aggregation.nSigFigs === 5 && aggregation.mantissa === undefined) {
    return { nSigFigs: 5, mantissa: 1 };
  }
  return aggregation;
}

/**
 * Aggregation identified by an incoming frame's snake_case tags. Untagged
 * frames (both fields absent) are full precision. Lenient — never throws.
 */
export function aggregationFromFrame(nSigFigs?: number, mantissa?: number): BookAggregation {
  return normalizeAggregation({ nSigFigs, mantissa });
}

/** Whether this is the full-precision (no aggregation) view. */
export function isFullPrecision(aggregation: BookAggregation): boolean {
  const normalized = normalizeAggregation(aggregation);
  return normalized.nSigFigs === undefined && normalized.mantissa === undefined;
}

/** Compare two aggregations as the backend does: by normalized value. */
export function aggregationsEqual(a: BookAggregation, b: BookAggregation): boolean {
  const normalizedA = normalizeAggregation(a);
  const normalizedB = normalizeAggregation(b);
  return (
    normalizedA.nSigFigs === normalizedB.nSigFigs &&
    normalizedA.mantissa === normalizedB.mantissa
  );
}

/**
 * Stable suffix for subscription keys: `"full"`, `"sig2"`..`"sig4"`, or
 * `"sig5m1"`/`"sig5m2"`/`"sig5m5"`. Matches the backend's subscription-key
 * vocabulary so keys are comparable across normalized spellings.
 */
export function aggregationKeySuffix(aggregation: BookAggregation): string {
  const { nSigFigs, mantissa } = normalizeAggregation(aggregation);
  if (nSigFigs === undefined) {
    return mantissa === undefined ? "full" : "invalid";
  }
  return mantissa === undefined ? `sig${nSigFigs}` : `sig${nSigFigs}m${mantissa}`;
}
