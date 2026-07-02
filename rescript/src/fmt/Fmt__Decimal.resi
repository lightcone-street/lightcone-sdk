// Fmt__Decimal — display formatting over decimal.js values (fmt/decimal.rs).
// Reached as `Fmt.Decimal`.

// Tier-based decimal places (≥10000→0, ≥1000→1, ≥100→2, ≥10→3, ≥0.1→4, else 5),
// ties away from zero, comma-grouped thousands; values that round to zero
// display as "0".
let display: Decimal.t => string

// Abbreviate with k/m/b/t suffixes at `digits` precision (ties to even).
// `~showSign=false` drops a leading minus.
let abbrNumber: (Decimal.t, ~digits: int=?, ~showSign: bool=?) => string

// Percentage with exactly 2 decimal places, truncated; `~padding=false` trims
// trailing zeros.
let displayPct: (Decimal.t, ~padding: bool=?) => string

// Human-readable Decimal → token base units (10^decimals, truncated). `None`
// when negative or over u64::MAX.
let toBaseUnits: (Decimal.t, ~decimals: int) => option<bigint>
