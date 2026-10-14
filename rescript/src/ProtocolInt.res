// Exact protocol integers. Unsafe JSON numbers are rejected; ExactJson turns
// their original integer tokens into decimal text before this boundary.
let decode = (json, minimum, maximum) => {
  let text = switch json {
  | JSON.String(text) => Some(text)
  | JSON.Number(value)
    if Float.isFinite(value) &&
    Math.floor(value) == value &&
    Math.abs(value) <= 9007199254740991.0 =>
    Some(Float.toString(value))
  | _ => None
  }
  switch text {
  | Some(text) if String.match(text, /^-?\d+$/)->Option.isSome =>
    switch BigInt.fromString(text) {
    | Some(value) if value >= minimum && value <= maximum => Ok(value)
    | _ => Spice.error("integer out of range", json)
    }
  | _ => Spice.error("expected an exact integer", json)
  }
}
module U64 = {
  type t = bigint
  let t_decode = json => decode(json, 0n, 18446744073709551615n)
  let t_encode = ExactJson.integer
  let t_encodeJson = ExactJson.integer
}
module I64 = {
  type t = bigint
  let t_decode = json => decode(json, -9223372036854775808n, 9223372036854775807n)
  let t_encode = ExactJson.integer
  let t_encodeJson = ExactJson.integer
}
