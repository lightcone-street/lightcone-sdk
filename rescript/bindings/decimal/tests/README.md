# `Decimal` binding tests

Runtime tests for the `decimal.js` binding. They exercise the **actual binding** (as a
ReScript consumer would) and run the compiled output under **Bun** — catching both type
errors (`rescript build`) and runtime errors (wrong JS method name / arg order / return
shape, at `bun test`). `decimal.js` is a pure utility, so there is no driver: each binding
is called directly and its return value asserted.

## Run

```bash
# from the rescript SDK root, build first, then run (note the ./ prefix)
./node_modules/.bin/rescript build
bun test ./bindings/decimal/tests/DecimalTest.res.mjs
```

The `./` prefix is required: `bun test` treats a bare path as a name filter.

## Coverage matrix

**Behaviorally tested** (asserted observable return value):

- Construction: `fromString`, `fromInt`, `fromFloat`.
- Arithmetic: `plus` (incl. the exact `0.1 + 0.2 == 0.3` case), `minus`, `times`, `div`,
  `pow`, `powInt`.
- Rounding / sign: `abs`, `floor`, `ceil`, `round` (both half-up directions),
  `toSignificantDigits`, `toDecimalPlaces` with `roundDown` (truncation).
- Comparison / predicates: `cmp` (-1 / 0 / 1), `eq`, `gt`, `gte`, `lt`, `lte`, `isZero`,
  `isNeg`.
- Terminal accessors: `toFixed` (padding), `toString`, `toNumber`.
- Constant: `roundDown` — exercised via `toDecimalPlaces`.

**Smoke only:** none.

**Not runtime-tested (reason):** none — every public binding is behaviorally covered.

`DecimalReadmeChecks.res` compile-guards every README snippet against the real
signatures.
