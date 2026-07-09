# `SortedBtree` — bindings to `sorted-btree`

ReScript bindings to [`sorted-btree`](https://github.com/qwertie/btree-typescript), a
**sorted (ordered) map** backed by a B+ tree. The SDK uses it to keep order-book bid/ask
price levels in key order (mirrors the Rust SDK's sorted book), so snapshots come out
already sorted and `minKey`/`maxKey` give best bid/ask in O(log n).

Written for someone who knows `sorted-btree` but is new to ReScript bindings. We bind the
constructor plus the map/lookup/ordering surface the SDK needs; the upstream `BTree<K,V>`
class has much more (range scans, cursors, bulk ops).

> Compatibility: `sorted-btree` **v2**. The default export is the `BTree<K, V>` class. The
> binding is generic — `SortedBtree.t<'k, 'v>` — so keys and values keep their ReScript
> types. `get` / `minKey` / `maxKey` return JS `undefined` when absent, which the binding
> surfaces as `option`.
>
> See [`tests/`](./tests) for the runnable suite and [`tests/README.md`](./tests/README.md)
> for the coverage matrix.

## Setup

The module is part of the SDK package (no extra `rescript.json` dependency). `sorted-btree`
must be installed (it is an SDK dependency). Reference it directly — `SortedBtree.make`,
`SortedBtree.set`, etc. No `open` is needed (and there is no namespace prefix).

**Trailing-`()` calling convention.** `make` takes two *optional* labeled arguments
(`~entries`, `~compare`) followed by a positional `unit`. Because the last argument is
positional `unit`, you always end the call with `()`:

```rescript
SortedBtree.make(~compare=(a, b) => a - b, ())   // comparator only
SortedBtree.make(~entries=[(1, "a")], ~compare=(a, b) => a - b, ())   // seed + comparator
SortedBtree.make()   // no args — default comparison (only safe for primitive keys)
```

Always pass `~compare` for deterministic ordering of non-trivial keys. For `int` keys,
`(a, b) => a - b` is a valid comparator (negative / zero / positive). Note ReScript's
`Int.compare` returns `Ordering.t` (a `float`), so it does **not** fit the `('k, 'k) => int`
comparator type — write the `int`-returning lambda shown above.

## How to read returned values

`SortedBtree.t<'k, 'v>` is an **opaque, mutable handle** (the `BTree`). You mutate it with
`set` / `delete` / `clear` and read content back through accessors. The shapes:

- **`option<'a>` — the one to watch.** `get` returns `option<'v>`; `minKey` / `maxKey`
  return `option<'k>`. A missing key (JS `undefined`) becomes `None`; a present value
  becomes `Some(value)`. Read it with `switch`, or `Option.getOr` / `Option.mapOr`:
  ```rescript
  switch book->SortedBtree.get(2) {
  | Some(value) => value
  | None => "default"
  }
  // or: book->SortedBtree.get(2)->Option.getOr("default")
  ```
- **`bool`** — `set` returns `true` if a **new** key was inserted, `false` if it overwrote
  an existing one; `has` is membership; `delete` returns `true` if a key was actually
  removed.
- **`int`** — `size` (a property, not a call: `book->SortedBtree.size`) and `forEachPair`
  (returns the number of pairs visited).
- **`unit`** — `clear` (empties in place).
- **`array<('k, 'v)>` / `array<'k>` / `array<'v>`** — `toArray` / `keysArray` /
  `valuesArray` return **ascending-by-key** snapshots. Each `toArray` entry is a 2-tuple
  `(key, value)`; destructure it:
  ```rescript
  book->SortedBtree.toArray->Array.forEach(((key, value)) => Console.log2(key, value))
  ```
- **callback** — `forEachPair(book, (key, value) => …)` runs the callback in ascending key
  order and returns the count.

## Quick start

```rescript
// int-keyed map; comparator returns an int
let book = SortedBtree.make(~compare=(a, b) => a - b, ())
let _ = book->SortedBtree.set(3, "c")   // inserted out of order
let _ = book->SortedBtree.set(1, "a")
let _ = book->SortedBtree.set(2, "b")

let keys = book->SortedBtree.keysArray      // [1, 2, 3] — ascending, regardless of insert order
let two = book->SortedBtree.get(2)          // Some("b")
let absent = book->SortedBtree.get(9)       // None
let lowest = book->SortedBtree.minKey       // Some(1)
let highest = book->SortedBtree.maxKey      // Some(3)
let count = book->SortedBtree.size          // 3
```

## Reference

### Construction

| Binding | Signature | Notes |
|---|---|---|
| `make` | `(~entries: array<('k, 'v)>=?, ~compare: ('k, 'k) => int=?, unit) => t<'k, 'v>` | `new BTree(entries?, compare?)`. Trailing `()` required (see Setup). `~entries` seeds the tree (sorted on insert); `~compare` must return an `int`. |

### Mutation — `=> bool` / `=> unit`

| Binding | Signature | Notes |
|---|---|---|
| `set` | `(t<'k, 'v>, 'k, 'v) => bool` | `true` if a **new** key was inserted, `false` if it overwrote an existing key. |
| `delete` | `(t<'k, 'v>, 'k) => bool` | `true` if the key existed and was removed. |
| `clear` | `t<'k, 'v> => unit` | Empties the tree in place. |

### Lookup — `=> option` / `=> bool` / `=> int`

| Binding | Signature | Notes |
|---|---|---|
| `get` | `(t<'k, 'v>, 'k) => option<'v>` | `Some(value)` / `None`. `undefined` → `None`. |
| `has` | `(t<'k, 'v>, 'k) => bool` | Membership test. |
| `size` | `t<'k, 'v> => int` | Property accessor (`@get`) — `book->SortedBtree.size`, not a call. |
| `minKey` | `t<'k, 'v> => option<'k>` | Smallest key, or `None` when empty. |
| `maxKey` | `t<'k, 'v> => option<'k>` | Largest key, or `None` when empty. |

### Ordered snapshots — ascending by key

| Binding | Signature | Notes |
|---|---|---|
| `toArray` | `t<'k, 'v> => array<('k, 'v)>` | Entries as `(key, value)` tuples, ascending. Destructure: `((key, value)) => …`. |
| `keysArray` | `t<'k, 'v> => array<'k>` | Keys, ascending. |
| `valuesArray` | `t<'k, 'v> => array<'v>` | Values in key order. |
| `forEachPair` | `(t<'k, 'v>, ('k, 'v) => unit) => int` | Runs the callback in key order; returns the number of pairs visited. |

## Escape hatches

- **Range scans / cursors / bulk ops** (`forRange`, `getRange`, `nextHigherKey`,
  `editRange`, …): add an ad-hoc `@send external` next to the existing ones, mirroring
  their shape. Return `undefined`-able results as `option<_>` (as `get`/`minKey` do).
- **A different default-export shape** (some bundlers expose `BTree.default`): the binding
  imports the package default; if interop ever changes, fall back to
  `@module("sorted-btree") external …` against the concrete export.
