// Price — decimal-string ↔ float helpers (mirrors rust/src/shared/price.rs).
// The SDK keeps price/size/balance fields as wire strings to preserve exact
// decimal representation; these are the lossy float conversions for callers
// doing quick math or display. For precision-safe math use `Decimal`.

// `Number("1.5abc")` is NaN (unlike parseFloat), giving strict semantics.
let strictNumber: string => float = %raw(`(s) => Number(s)`)

// Strict parse of a decimal string to float — the whole string must be a valid
// number (mirrors Rust `str::parse::<f64>`, which rejects trailing garbage;
// plain `parseFloat` would accept "1.5abc").
let parseDecimal = (value: string): option<float> => {
  let trimmed = value->String.trim
  if trimmed == "" {
    None
  } else {
    let parsed = strictNumber(trimmed)
    Float.isNaN(parsed) ? None : Some(parsed)
  }
}

// Format a float as a decimal string with the given precision, e.g.
// formatDecimal(0.5, ~precision=6) → "0.500000".
let formatDecimal = (value: float, ~precision: int): string =>
  Float.toFixed(value, ~digits=precision)
