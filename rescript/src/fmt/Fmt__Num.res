// Fmt__Num — float display formatting + base-unit conversions. Reached as
// `Fmt.Num`; the string plumbing (`displayFormattedString` and friends) lives
// here, with `Fmt__Decimal` consuming it cross-file.

// Insert thousands separators into a fixed-format number string, preserving the
// sign and fractional digits.
let displayFormattedString = (formatted: string): string => {
  let (integer, fraction) = switch formatted->String.split(".") {
  | [integer, fraction] => (integer, Some(fraction))
  | _ => (formatted, None)
  }
  let (sign, digits) = integer->String.startsWith("-")
    ? ("-", integer->String.slice(~start=1))
    : ("", integer)
  let grouped = []
  let length = String.length(digits)
  for index in 0 to length - 1 {
    if index > 0 && mod(length - index, 3) == 0 {
      grouped->Array.push(",")
    }
    grouped->Array.push(digits->String.slice(~start=index, ~end=index + 1))
  }
  let integerPart = grouped->Array.join("")
  switch fraction {
  | Some(fraction) => `${sign}${integerPart}.${fraction}`
  | None => `${sign}${integerPart}`
  }
}

// Every digit is zero (so "-0.000", "0", "0.00" all count as zero).
let isFormattedZero = (formatted: string): bool =>
  formatted
  ->String.split("")
  ->Array.every(char => char == "0" || char == "." || char == "-")

// Values that round to zero display as plain "0"; everything else gets grouped.
// (Fmt-internal: consumed by `Fmt__Decimal`.)
let displayDefaultFormattedString = (formatted: string): string =>
  isFormattedZero(formatted) ? "0" : displayFormattedString(formatted)

let displayDecimals = (absValue: float): int =>
  Fmt__Constants.displayDecimalsBy(threshold =>
    absValue >= Float.fromString(threshold)->Option.getOr(0.0)
  )

// Format for display with tier-based decimal places (see Fmt__Constants).
let display = (amount: float): string =>
  Float.toFixed(amount, ~digits=displayDecimals(Math.abs(amount)))->displayDefaultFormattedString

// Format with explicit decimal places (no zero-collapse).
let displayWithDecimals = (amount: float, ~decimals: int): string =>
  Float.toFixed(amount, ~digits=decimals)->displayFormattedString

// Percentage with exactly 2 decimal places, truncated; `~padding=false` trims
// trailing zeros.
let displayPct = (value: float, ~padding: bool=true): string => {
  let truncated = Math.trunc(value *. 100.0) /. 100.0
  if padding {
    displayFormattedString(Float.toFixed(truncated, ~digits=2))
  } else {
    let formatted = Float.toFixed(truncated, ~digits=2)
    let trimmed = if formatted->String.includes(".") {
      let noZeros = formatted->String.replaceRegExp(%re("/0+$/"), "")
      noZeros->String.endsWith(".") ? noZeros->String.slice(~start=0, ~end=-1) : noZeros
    } else {
      formatted
    }
    displayFormattedString(trimmed)
  }
}

// On-chain base units → human-readable float, e.g. (1_500_000_000n, 9) → 1.5.
let toDecimalValue = (value: bigint, ~decimals: int): float =>
  BigInt.toFloat(value) /. Math.pow(10.0, ~exp=Int.toFloat(decimals))

// Human-readable float → on-chain base units, e.g. (1.5, 9) → 1_500_000_000n.
// Truncation toward zero; negatives/NaN → 0.
let fromDecimalValue = (value: float, ~decimals: int): bigint => {
  let scaled = Math.trunc(value *. Math.pow(10.0, ~exp=Int.toFloat(decimals)))
  BigInt.fromFloat(Math.max(scaled, 0.0))->Option.getOr(0n)
}
