// Price — decimal-string ↔ float helpers (mirrors rust/src/shared/price.rs).
// Lossy float conversions; use `Decimal` for precision-safe math.

// Strict parse of a decimal string to float (whole string must be numeric).
let parseDecimal: string => option<float>

// Format a float as a decimal string with the given precision.
let formatDecimal: (float, ~precision: int) => string
