// Fmt__Num — float display formatting + base-unit conversions (fmt/num.rs).
// Reached as `Fmt.Num`. The zero-collapsing default formatter is exported for
// `Fmt__Decimal` (Rust `pub(super)`); the zero test stays private.

// Insert thousands separators into a fixed-format number string, preserving the
// sign and fractional digits.
let displayFormattedString: string => string

// Values that round to zero display as plain "0"; everything else gets grouped.
// Fmt-internal (consumed by `Fmt__Decimal`).
let displayDefaultFormattedString: string => string

// Tier-based decimal places over a float (≥10000→0, ≥1000→1, ≥100→2, ≥10→3,
// ≥0.1→4, else 5).
let display: float => string

// Explicit decimal places (no zero-collapse).
let displayWithDecimals: (float, ~decimals: int) => string

// Percentage with exactly 2 decimal places, truncated; `~padding=false` trims
// trailing zeros.
let displayPct: (float, ~padding: bool=?) => string

// On-chain base units → human-readable float, e.g. (1_500_000_000n, 9) → 1.5.
let toDecimalValue: (bigint, ~decimals: int) => float

// Human-readable float → on-chain base units (truncated toward zero;
// negatives/NaN → 0).
let fromDecimalValue: (float, ~decimals: int) => bigint
