# `NobleHashes` binding tests

Runtime tests for the `@noble/hashes` binding. They exercise the **actual binding** (as
a ReScript consumer would) and run the compiled output under **Bun** — catching both
type errors (`rescript build`) and runtime errors (wrong JS export name / arg order /
return shape, at `bun test`).

## Run

```bash
# from the rescript SDK root, build first, then run (note the ./ prefix)
./node_modules/.bin/rescript build
bun test ./bindings/noble-hashes/tests/NobleHashesTest.res.mjs
```

The `./` prefix is required: `bun test` treats a bare path as a name filter.

## Coverage matrix

**Behaviorally tested:** `keccak256` — asserted against the canonical `keccak256("abc")`
vector, the 32-byte output length, and determinism.

**Smoke only:** none.

**Not runtime-tested (reason):** none — the binding has a single function and it is fully
covered.

`NobleHashesReadmeChecks.res` (in `Support`-style, here flat) compile-guards every README
snippet against the real signature.
