// Fmt__Constants — the shared display-precision tier table. Internal to the
// Fmt namespace: consumed by `Fmt__Decimal` and `Fmt__Num`, not re-exported
// through `Fmt`.

// Display decimal places for a magnitude, via a caller-supplied tier test:
// ≥10000→0, ≥1000→1, ≥100→2, ≥10→3, ≥0.1→4, else 5.
let displayDecimalsBy = (gteThreshold: string => bool): int =>
  if gteThreshold("10000") {
    0
  } else if gteThreshold("1000") {
    1
  } else if gteThreshold("100") {
    2
  } else if gteThreshold("10") {
    3
  } else if gteThreshold("0.1") {
    4
  } else {
    5
  }
