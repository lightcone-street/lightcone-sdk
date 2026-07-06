// Fmt — human-readable display formatting.
// Pure namespace: each alias re-exports its `Fmt__*` implementation file, whose
// `.resi` is the real interface. `Fmt__Constants` (the shared tier table) is
// namespace-internal and deliberately not re-exported.

module Decimal = Fmt__Decimal
module Num = Fmt__Num
module Str = Fmt__Str
