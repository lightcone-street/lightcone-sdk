// Binding to sorted-btree — an ordered map used by OrderbookState to keep bid/ask
// price levels sorted (mirrors the Rust SDK's sorted book). The default export is
// the `BTree<K, V>` class. `get`/`minKey`/`maxKey` return `undefined` when absent,
// which we surface as `option`.

type t<'k, 'v>

// new BTree(entries?, compare?) — pass a comparator for deterministic ordering.
@new @module("sorted-btree")
external make: (~entries: array<('k, 'v)>=?, ~compare: ('k, 'k) => int=?, unit) => t<'k, 'v> =
  "default"

@send external set: (t<'k, 'v>, 'k, 'v) => bool = "set"
@send external get: (t<'k, 'v>, 'k) => option<'v> = "get"
@send external has: (t<'k, 'v>, 'k) => bool = "has"
@send external delete: (t<'k, 'v>, 'k) => bool = "delete"
@send external clear: t<'k, 'v> => unit = "clear"
@get external size: t<'k, 'v> => int = "size"

@send external minKey: t<'k, 'v> => option<'k> = "minKey"
@send external maxKey: t<'k, 'v> => option<'k> = "maxKey"

// Ordered snapshots (ascending key order).
@send external toArray: t<'k, 'v> => array<('k, 'v)> = "toArray"
@send external keysArray: t<'k, 'v> => array<'k> = "keysArray"
@send external valuesArray: t<'k, 'v> => array<'v> = "valuesArray"
@send external forEachPair: (t<'k, 'v>, ('k, 'v) => unit) => int = "forEachPair"
