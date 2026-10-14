// Lossless integer wire boundary. Unsafe response integers become decimal text,
// while outgoing bigint values retain JSON integer tokens rather than quoted text.
type codec = {
  parse: string => JSON.t,
  stringify: JSON.t => string,
}
@module("json-bigint") external make: {"storeAsString": bool, "strict": bool} => codec = "default"
let codec = make({"storeAsString": true, "strict": true})

// This opaque bridge is accepted only by the lossless serializer below.
external integer: bigint => JSON.t = "%identity"
let stringify = (value: JSON.t): string => codec.stringify(value)
let parse = (text: string): result<JSON.t, string> =>
  switch codec.parse(text) {
  | json => Ok(json)
  | exception _ => Error("invalid JSON response")
  }
