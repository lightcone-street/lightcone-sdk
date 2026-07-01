// Binding to sorted-btree — an ordered map used by OrderbookState to keep bid/ask
// price levels sorted (mirrors the Rust SDK's sorted book). The default export is
// the `BTree<K, V>` class. `get`/`minKey`/`maxKey` return `undefined` when absent,
// which we surface as `option`.

type t<'k, 'v>

// sorted-btree ships as CommonJS (`exports.default = BTree`, `__esModule: true`). Under
// Node's ESM interop a bare default import yields the module *namespace* rather than the
// `BTree` class, so resolve the constructor through `.default` (Bun already unwraps it)
// before `new`. Only the constructor needs this; the `@send` methods below are instance
// calls with no import interop.
%%raw(`import SortedBtreeDefault from "sorted-btree"`)
let makeWith: (option<array<('k, 'v)>>, option<('k, 'k) => int>) => t<'k, 'v> = %raw(`
  function (entries, compare) {
    // Node: the default import is module.exports, so the class is at .default; Bun already
    // unwraps to the class (its .default is undefined), so fall back to the import itself.
    const BTree = SortedBtreeDefault.default || SortedBtreeDefault;
    return new BTree(entries, compare);
  }
`)

// new BTree(entries?, compare?) — pass a comparator for deterministic ordering.
let make = (~entries: option<array<('k, 'v)>>=?, ~compare: option<('k, 'k) => int>=?, ()): t<'k, 'v> =>
  makeWith(entries, compare)

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
