// Compile-guard for README.md — a compile-only file (no test blocks). If a README
// snippet drifts from the actual binding signature, `rescript build` fails here.

// Quick start
let _quickStart = () => {
  let _formatted: string =
    Decimal.fromString("1.5")->Decimal.times(Decimal.fromInt(2))->Decimal.toFixed(1)
  let _exact: string =
    Decimal.fromString("0.1")->Decimal.plus(Decimal.fromString("0.2"))->Decimal.toFixed(1)
}

// Construction
let _fromString = (): Decimal.t => Decimal.fromString("1.5")
let _fromInt = (): Decimal.t => Decimal.fromInt(42)
let _fromFloat = (): Decimal.t => Decimal.fromFloat(2.5)

// Arithmetic — (t, t) => t, chainable
let _arithmetic = (left: Decimal.t, right: Decimal.t) => {
  let _plus: Decimal.t = left->Decimal.plus(right)
  let _minus: Decimal.t = left->Decimal.minus(right)
  let _times: Decimal.t = left->Decimal.times(right)
  let _div: Decimal.t = left->Decimal.div(right)
  let _pow: Decimal.t = left->Decimal.pow(right)
  let _powInt: Decimal.t = left->Decimal.powInt(3)
}

// Rounding / sign — t => t, chainable
let _rounding = (value: Decimal.t) => {
  let _abs: Decimal.t = value->Decimal.abs
  let _floor: Decimal.t = value->Decimal.floor
  let _ceil: Decimal.t = value->Decimal.ceil
  let _round: Decimal.t = value->Decimal.round
  let _sig: Decimal.t = value->Decimal.toSignificantDigits(2)
  let _dp: Decimal.t = value->Decimal.toDecimalPlaces(2, Decimal.roundDown)
}

// Comparison / predicates — => int / => bool
let _comparison = (left: Decimal.t, right: Decimal.t) => {
  let _cmp: int = left->Decimal.cmp(right)
  let _eq: bool = left->Decimal.eq(right)
  let _gt: bool = left->Decimal.gt(right)
  let _gte: bool = left->Decimal.gte(right)
  let _lt: bool = left->Decimal.lt(right)
  let _lte: bool = left->Decimal.lte(right)
  let _isZero: bool = left->Decimal.isZero
  let _isNeg: bool = left->Decimal.isNeg
}

// Terminal accessors — => string / => float
let _terminal = (value: Decimal.t) => {
  let _toFixed: string = value->Decimal.toFixed(3)
  let _toString: string = value->Decimal.toString
  let _toNumber: float = value->Decimal.toNumber
}

// Exported constant
let _roundDown: int = Decimal.roundDown
