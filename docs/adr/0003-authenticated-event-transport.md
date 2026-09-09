# ADR 0003: Authenticated Event Transport Trailer

- Status: Accepted
- Date: 2026-09-02

## Context

The Rust, Python, and TypeScript SDKs must encode the same program wire contract.
The [program integration contract](https://github.com/lightcone-street/docs/blob/0886e2356c69e8d59b2dca953331f63d7ecd9619/api-reference/program-integration.mdx) owns event transport behavior,
invocation rules, position replay semantics, and program limits. This ADR records
how the SDKs expose that contract and preserve their public APIs.

## Decision

Every SDK instruction builder appends the event-authority and program trailer
through one private constructor. A table-driven test checks the trailer for every
public builder. The SDKs expose the event-authority seed and PDA helper, reserved
event-batch discriminator, and deposit-mint limits. Their program error types mirror
codes 68 through 75, and market decoders read `deposit_mint_count` at byte offset 148.
The SDKs neither build nor decode event batches, and builders do not add
compute-budget instructions.

`ExtendPositionTokens` parameters name the signer `payer`. Fluent builders retain
a deprecated `operator()` alias that forwards to `payer()`. Raw builder signatures
are otherwise preserved. Already-fallible position initialization and extension
paths validate beneficiaries and mint-list bounds before serialization. Rust's
infallible `build_init_position_tokens_ix` and `Positions::init_position_tokens_ix`
defer these checks to the program; its fluent builder and transaction helper
validate them locally.

## Considered Options

An optional trailer would create two incompatible instruction layouts behind the
same SDK API. Builders therefore always append it. Changing infallible Rust raw
builders to return `Result` was rejected to preserve existing public signatures.

## Consequences

Consumers that inspect instruction accounts must account for the two trailing
entries. Code constructing `ExtendPositionTokens` parameters uses `payer`; the
fluent alias supports migration from `operator()`. Callers select the SDK version
that matches their program deployment and budget compute for the final self-CPI.
