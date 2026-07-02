// Fmt — human-readable display formatting, the ReScript counterpart of
// rust/src/shared/fmt. Namespace shortcuts over the per-concern implementation
// files (the stdlib `Stdlib.res` → `Stdlib_Array` pattern): `Fmt.Decimal`
// (fmt/decimal.rs, Decimal-string inputs), `Fmt.Num` (fmt/num.rs, float
// inputs), `Fmt.Str` (fmt/str.rs). The shared magnitude tier table lives in
// `Fmt__Constants` (fmt/constants.rs) and stays internal to the namespace.

module Decimal = Fmt__Decimal
module Num = Fmt__Num
module Str = Fmt__Str
