// Fmt__Decimal — display formatting over decimal.js values parsed from the
// SDK's Decimal wire strings. Reached as `Fmt.Decimal`; the string plumbing
// comes from `Fmt__Num` and the tier table from `Fmt__Constants`.

let thousand = Decimal.fromInt(1000)
let million = Decimal.fromInt(1000000)
let billion = Decimal.fromString("1000000000")
let trillion = Decimal.fromString("1000000000000")

let displayDecimals = (absValue: Decimal.t): int =>
  Fmt__Constants.displayDecimalsBy(threshold => Decimal.gte(absValue, Decimal.fromString(threshold)))

// Format for display: tier-based decimal places, ties away from zero,
// comma-grouped thousands; values that round to zero display as "0".
let display = (value: Decimal.t): string => {
  let decimals = displayDecimals(Decimal.abs(value))
  value
  ->Decimal.toDecimalPlaces(decimals, Decimal.roundHalfUp)
  ->Decimal.toFixed(decimals)
  ->Fmt__Num.displayDefaultFormattedString
}

// Abbreviate with k/m/b/t suffixes at `digits` precision (ties round to
// even). `~showSign=false` drops a leading minus.
let abbrNumber = (amount: Decimal.t, ~digits: int=2, ~showSign: bool=true): string => {
  let sign = showSign && Decimal.isNeg(amount) ? "-" : ""
  let absAmount = Decimal.abs(amount)
  let scaled = (value, divisor, suffix) =>
    `${sign}${Decimal.div(value, divisor)->Decimal.toFixedWithRounding(digits, Decimal.roundHalfEven)}${suffix}`
  if Decimal.gte(absAmount, trillion) {
    scaled(absAmount, trillion, "t")
  } else if Decimal.gte(absAmount, billion) {
    scaled(absAmount, billion, "b")
  } else if Decimal.gte(absAmount, million) {
    scaled(absAmount, million, "m")
  } else if Decimal.gte(absAmount, thousand) {
    scaled(absAmount, thousand, "k")
  } else {
    `${sign}${absAmount->Decimal.toFixedWithRounding(digits, Decimal.roundHalfEven)}`
  }
}

// Percentage with exactly 2 decimal places, TRUNCATED (not rounded). With
// `~padding=true` (default) always two places ("12.30"); otherwise trailing
// zeros are trimmed ("12.3").
let displayPct = (value: Decimal.t, ~padding: bool=true): string => {
  let truncated = value->Decimal.toDecimalPlaces(2, Decimal.roundDown)
  padding
    ? Fmt__Num.displayFormattedString(truncated->Decimal.toFixed(2))
    : Fmt__Num.displayFormattedString(truncated->Decimal.toString)
}

let u64Max = Decimal.fromString("18446744073709551615")

// Human-readable Decimal → token base units (10^decimals, truncated), e.g.
// 10.5 USDC (6 decimals) → 10_500_000n. `None` when negative or over u64::MAX.
let toBaseUnits = (value: Decimal.t, ~decimals: int): option<bigint> => {
  let scaled = Decimal.times(value, Decimal.powInt(Decimal.fromInt(10), decimals))
  if Decimal.isNeg(scaled) || Decimal.gt(scaled, u64Max) {
    None
  } else {
    // BigInt.fromString is already option-returning (Some for this integral string).
    BigInt.fromString(scaled->Decimal.toFixedWithRounding(0, Decimal.roundDown))
  }
}
