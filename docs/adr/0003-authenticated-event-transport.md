# ADR 0003: Authenticated Event Transport Trailer

- Status: Accepted
- Date: 2026-09-02

## Context

The Lightcone program (lightcone-pinnochio ticket LIG-957) replaced runtime-log accounting events with one PDA-authenticated self-CPI event batch per successful public instruction. Every public instruction now ends with two read-only, non-signer trailer accounts: the event-authority PDA derived from the seed `__event_authority` and the executable program account. The program pops both before dispatch and rejects a missing, wrong, or writable trailer before any state change, so legacy instruction builders fail closed. The same release makes `AddDepositMint` write the market account, adds `Market.deposit_mint_count` at byte offset 148, caps deposit mints at eight per market, makes `ExtendPositionTokens` permissionless, and makes `InitPositionTokens` idempotent. The three SDKs must present one contract for this ABI.

## Decision

Every SDK instruction builder appends the trailer unconditionally through a single private constructor, so no builder can omit it and one table-driven test proves the invariant for every public builder. The SDK exposes the seed, the event-authority PDA helper, the reserved private discriminator, and the deposit-mint limits as documented constants, and mirrors on-chain error codes 68 through 75 in its program error type. The SDK does not build or decode event batches.

`ExtendPositionTokens` parameters name the signer `payer` rather than `operator`; the fluent builder keeps a deprecated `operator()` alias that forwards to `payer()`. Raw builder signatures are otherwise preserved; the per-instruction deposit-mint limit is validated only on already-fallible paths. The SDK does not attach a compute-budget instruction; callers budget for the program's final self-CPI themselves.

The Rust SDK implements this decision first. Python and TypeScript must reach parity under the AGENTS.md cross-language rule before the program upgrade is activated in an environment they target.

## Considered Options

A builder flag that omits the trailer for older program deployments was rejected: the previous program parses trailing accounts as remaining accounts for init, extend, cleanup, and match instructions, so no single instruction shape works against both program versions and the cutover must be coordinated per environment. Keeping the `operator` name with corrected documentation was rejected because the name would misstate the authorization the program enforces. Making infallible raw builders return `Result` to validate the deposit-mint limit was rejected to preserve public signatures.

## Consequences

Transactions built by an upgraded SDK gain one static account key and two account indexes per instruction and are rejected by the previous program; transactions built by an older SDK are rejected by the upgraded program. Each environment therefore upgrades the program and the SDKs together. Direct construction of `ExtendPositionTokensParams` with the old field name no longer compiles. Consumers that enumerate instruction accounts must expect the two trailing entries, and consumers that wrap Lightcone instructions in their own program's CPI can no longer do so.
