// Fmt — human-readable display formatting. Namespace shortcuts over the
// per-concern implementation files (the stdlib `Stdlib.res` → `Stdlib_Array`
// pattern): `Fmt.Decimal` (Decimal-string inputs), `Fmt.Num` (float inputs),
// `Fmt.Str` (string helpers). The shared magnitude tier table lives in
// `Fmt__Constants` and stays internal to the namespace.

module Decimal = Fmt__Decimal
module Num = Fmt__Num
module Str = Fmt__Str
