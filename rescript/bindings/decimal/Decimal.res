// Binding to decimal.js — arbitrary-precision decimals for price/size scaling and
// display formatting (mirrors the Rust SDK's rust_decimal usage). The default
// export is the `Decimal` class; `Decimal.Value` accepts string | number | bigint
// | Decimal, so we expose a few constructors and the operations the SDK needs.

type t

@new @module("decimal.js") external fromString: string => t = "default"
@new @module("decimal.js") external fromInt: int => t = "default"
@new @module("decimal.js") external fromFloat: float => t = "default"

@send external plus: (t, t) => t = "plus"
@send external minus: (t, t) => t = "minus"
@send external times: (t, t) => t = "times"
@send external div: (t, t) => t = "div"
@send external pow: (t, t) => t = "pow"
// Integer exponent convenience (e.g. 10 ^ token_decimals).
@send external powInt: (t, int) => t = "pow"

@send external abs: t => t = "abs"
@send external floor: t => t = "floor"
@send external ceil: t => t = "ceil"
@send external round: t => t = "round"

// comparedTo / cmp returns -1 | 0 | 1.
@send external cmp: (t, t) => int = "cmp"
@send external eq: (t, t) => bool = "eq"
@send external gt: (t, t) => bool = "gt"
@send external gte: (t, t) => bool = "gte"
@send external lt: (t, t) => bool = "lt"
@send external lte: (t, t) => bool = "lte"
@send external isZero: t => bool = "isZero"
@send external isNeg: t => bool = "isNeg"

// toDecimalPlaces(dp, roundingMode). `roundDown` truncates toward zero.
@send external toDecimalPlaces: (t, int, int) => t = "toDecimalPlaces"
let roundDown = 1

@send external toFixed: (t, int) => string = "toFixed"
@send external toString: t => string = "toString"
@send external toNumber: t => float = "toNumber"
@send external toSignificantDigits: (t, int) => t = "toSignificantDigits"
