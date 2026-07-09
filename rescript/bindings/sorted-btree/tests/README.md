# `SortedBtree` binding tests

Runtime tests for the `sorted-btree` binding. They exercise the **actual binding** (as a
ReScript consumer would) and run the compiled output under **Bun** — catching both type
errors (`rescript build`) and runtime errors (wrong JS method name / arg order / return
shape, at `bun test`). `sorted-btree` is a pure utility, so there is no driver: each test
builds a tree and asserts the return value.

## Run

```bash
# from the rescript SDK root, build first, then run (note the ./ prefix)
./node_modules/.bin/rescript build
bun test ./bindings/sorted-btree/tests/SortedBtreeTest.res.mjs
```

The `./` prefix is required: `bun test` treats a bare path as a name filter.

## Library-specific gotchas (encoded in the tests)

- **Comparator must return `int`.** ReScript's `Int.compare` returns `Ordering.t`
  (a `float`), which does not fit `('k, 'k) => int`. The suite uses
  `(left, right) => left - right` for `int` keys.
- **`make` needs a trailing `()`.** Its labeled args are optional, so the call ends with
  the positional `unit`: `SortedBtree.make(~compare, ())`.
- **The tree is mutable.** Each test builds a **fresh** tree (`emptyTree()` / `seededTree()`)
  so mutation in one test can't leak into another.
- **`undefined` → `None`.** `get` / `minKey` / `maxKey` surface absence as `None`; the tests
  assert both the `Some(_)` and `None` paths.

## Coverage matrix

**Behaviorally tested** (asserted observable return value):

- Construction: `make` (with `~compare`; and with `~entries` seeding, asserting the seed is
  sorted).
- Mutation: `set` (true for new key / false on overwrite, plus the overwrite is visible),
  `delete` (true when removed / false when absent), `clear`.
- Lookup: `get` (`Some` and `None`), `has` (present / absent), `size`, `minKey` / `maxKey`
  (`Some` and the empty-tree `None`).
- Ordered snapshots: `keysArray` (ascending despite out-of-order inserts), `valuesArray`,
  `toArray` (tuple entries in key order), `forEachPair` (ascending visit order + returned
  count).

**Smoke only:** none.

**Not runtime-tested (reason):** none — every public binding is behaviorally covered.

`SortedBtreeReadmeChecks.res` compile-guards every README snippet against the real
signatures.
