// Exact decimal text admission. Exponents and insignificant trailing zeroes are
// accepted, but scaling never rounds. Bounds match the protocol's Rust parser.
let isInteger: int => bool = %raw(`Number.isInteger`)
let scaledInteger = (value: string, decimals: int): result<bigint, string> => {
  let text = value->String.trim->String.toLowerCase
  if !isInteger(decimals) || decimals < 0 || decimals > 255 {
    Error("decimal scale must fit u8")
  } else if !RegExp.test(/^\+?(?:[0-9]+\.?[0-9]*|\.[0-9]+)(?:e[+-]?[0-9]+)?$/, text) {
    Error("expected non-negative base-10 decimal syntax")
  } else {
    let parts = String.split(text, "e")
    let mantissa = parts->Array.get(0)->Option.getOr("")->String.replaceRegExp(/^\+/, "")
    let exponentText = parts->Array.get(1)->Option.getOr("0")
    switch Float.fromString(exponentText) {
    | None => Error("invalid exponent")
    | Some(exponent) if !Float.isFinite(exponent) || Math.abs(exponent) > 10000.0 =>
      Error("invalid or unsupported exponent")
    | Some(exponent) =>
      let decimalParts = String.split(mantissa, ".")
      let whole = decimalParts->Array.get(0)->Option.getOr("")
      let fraction = decimalParts->Array.get(1)->Option.getOr("")
      let coefficient = (whole ++ fraction)->String.replaceRegExp(/^0+/, "")
      let shift = decimals + Float.toInt(exponent) - String.length(fraction)
      let digits = if coefficient == "" {
        Ok("0")
      } else if shift >= 0 {
        if String.length(coefficient) + shift > 10000 {
          Error("scaled value is too large")
        } else {
          Ok(coefficient ++ String.repeat("0", shift))
        }
      } else {
        let split = String.length(coefficient) + shift
        if (
          split < 0 ||
            !RegExp.test(
              /^0*$/,
              String.slice(coefficient, ~start=split, ~end=String.length(coefficient)),
            )
        ) {
          Error("cannot be represented exactly at this scale")
        } else {
          Ok(String.slice(coefficient, ~start=0, ~end=split))
        }
      }
      switch digits {
      | Error(error) => Error(error)
      | Ok(digits) =>
        switch BigInt.fromString(digits) {
        | Some(integer) => Ok(integer)
        | None => Error("invalid integer coefficient")
        }
      }
    }
  }
}

// Exact lamport/token formatting with a fixed number of fractional digits.
let formatScaled = (value: bigint, decimals: int): result<string, string> =>
  if value < 0n || !isInteger(decimals) || decimals < 0 || decimals > 255 {
    Error("invalid unsigned amount or decimal scale")
  } else if decimals == 0 {
    Ok(BigInt.toString(value))
  } else {
    let text = BigInt.toString(value)->String.padStart(decimals + 1, "0")
    let split = String.length(text) - decimals
    Ok(
      String.slice(text, ~start=0, ~end=split) ++
      "." ++
      String.slice(text, ~start=split, ~end=String.length(text)),
    )
  }
