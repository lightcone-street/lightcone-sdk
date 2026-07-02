// Fmt__Constants — the shared display-precision tier table (fmt/constants.rs).
// Internal to the Fmt namespace.

// Display decimal places for a magnitude, via a caller-supplied tier test:
// ≥10000→0, ≥1000→1, ≥100→2, ≥10→3, ≥0.1→4, else 5.
let displayDecimalsBy: (string => bool) => int
