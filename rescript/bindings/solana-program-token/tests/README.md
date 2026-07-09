# `SolanaProgramToken` binding tests

Runtime tests for the `@solana-program/token` binding. They exercise the **actual binding** (as
a ReScript consumer would) and run the compiled output under **Bun** — catching both type errors
(`rescript build`) and runtime errors (wrong JS export name / arg order / return shape, at
`bun test`).

## Run

```bash
# from the rescript SDK root, build first, then run (note the ./ prefix)
./node_modules/.bin/rescript build
bun test ./bindings/solana-program-token/tests/SolanaProgramTokenTest.res.mjs
```

The `./` prefix is required: `bun test` treats a bare path as a name filter.

## Coverage matrix

**Behaviorally tested:**
- `tokenProgramAddress` / `associatedTokenProgramAddress` — asserted to be valid base58 addresses
  (decoded string length > 30), proving the `TOKEN_PROGRAM_ADDRESS` / `ASSOCIATED_TOKEN_PROGRAM_ADDRESS`
  exports resolve to real addresses.
- `findAssociatedTokenPda` — asserted to return a valid ATA address and a bump in `0..255` for a
  known owner+mint, **and** to match the canonical ATA (`DHe62eeQVEnNK7vg5xUpDkJm7tuqHadjhvmPRFBG9UPo`,
  bump `254`) — which additionally pins the seed-record field order (owner vs mint) and determinism.

**Smoke only:** none.

**Not runtime-tested (reason):** none — all three bindings are covered.

`SolanaProgramTokenReadmeChecks.res` compile-guards every README snippet against the real
signatures, so the docs cannot silently drift from the API.
