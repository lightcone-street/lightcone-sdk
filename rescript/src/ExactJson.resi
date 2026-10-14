// Encode protocol integers without converting them to JavaScript numbers.
let integer: bigint => JSON.t
let stringify: JSON.t => string
// Safe numbers remain numbers. Unsafe integers become exact decimal strings.
let parse: string => result<JSON.t, string>
