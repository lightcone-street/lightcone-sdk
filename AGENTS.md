# Lightcone SDK Agent Guidance

## Guidance Layers

Use the shared agent baseline from the private
[`lightcone-street/conetext`](https://github.com/lightcone-street/conetext)
repository at the minimum release declared in `.conetext-version`. This file owns
SDK-specific compatibility, parity, and validation guidance.

Developer instructions override both layers. Follow the override, briefly disclose its
consequence, and persist an exception only when requested.

## Context Ownership

Read `CONTEXT.md` and the relevant ADR before changing SDK terminology or contracts.
Published product and API behavior belongs in `lightcone-street/docs`; point to its owning
page instead of copying public definitions here.

## Compatibility

- Treat Rust, Python, and TypeScript as language-idiomatic views of one semantic SDK
  contract. Keep behavior, exact units, wire fields, error conditions, and exports aligned.
- Preserve public names, signatures, return shapes, serialization, and feature behavior
  unless the task explicitly authorizes a breaking change.
- Keep fund-moving values exact. Rust uses integer and `Decimal` boundaries, Python uses
  integer and exact-decimal boundaries, and TypeScript uses `bigint` and decimal strings.
  Document the unit at every public numeric boundary.
- Keep wire models compatible with the backend payload. Domain models may add validation or
  presentation, but must not silently reinterpret API fields.
- A contract change in one SDK requires a parity review of the corresponding declarations,
  exports, docs, and tests in the other two SDKs.

## Tests

- Keep Rust unit tests beside their owning modules. Run
  `cargo fmt --manifest-path rust/Cargo.toml --all` and
  `cargo test --manifest-path rust/Cargo.toml --features native`. Also use
  `native,trigger_orders` when touching trigger-order surfaces.
- Keep Python tests under `python/tests`. From `python`, run Black and Ruff on the changed
  Python files, then run `uv run pytest`. Do not fold repository-wide formatting or lint
  cleanup into an unrelated task.
- Keep TypeScript tests under `typescript/tests`. From `typescript`, run `npm run lint`,
  `npm run typecheck`, `npm run typecheck:examples`, and `npm test`.
- For cross-language contracts, run the nearest equivalent tests in all three SDKs. Assert
  decoded instructions, exact amounts, and failure cases rather than instruction counts alone.
- Examples require the matching backend environment. The fund-moving
  `deposit_token_balances` example is manual-only and restricted to built-in local or staging
  routing.
