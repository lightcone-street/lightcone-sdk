# ADR 0003: Authenticated Event Transport Trailer

- Status: Accepted
- Date: 2026-09-02

## Context

The Rust, Python, and TypeScript SDKs must encode the same program wire contract.
The [program integration contract](https://github.com/lightcone-street/docs/blob/0886e2356c69e8d59b2dca953331f63d7ecd9619/api-reference/program-integration.mdx) owns event transport behavior and invocation rules. The [program source at `db552338`](https://github.com/lightcone-street/lightcone-pinnochio/tree/db552338404263b17b6af5e39a99477ee16a1934/src) defines the current binary interfaces. This ADR records the SDK exposure and compatibility boundary.

## Decision

Every SDK instruction builder appends the event-authority and program trailer
through one private constructor. A table-driven test checks the trailer for every
public builder. The SDKs expose the event-authority seed and PDA helper, reserved
event-batch discriminator, and deposit-mint limits. Their program error types expose the retained event-transport failures and collateral errors 76 and 77. Removed error numbers remain unassigned. Market decoders read `deposit_mint_count` at byte offset 148.
The SDKs neither build nor decode event batches, and builders do not add
compute-budget instructions.

The SDKs target the program ABI at `db552338` as a hard cutover. The program emits event schema 2; this event payload version is independent of the outer Solana transaction version. The SDKs expose the 176-byte Orderbook provenance and require both collateral identities for trading. They reject the legacy account layout and remove the program's retired address lookup table (ALT) operations, parameters, and helpers.

`InitPositionTokens` prepares every requested collateral group without a slot or lookup table. Callers supply groups in increasing global registration index order. The same instruction prepares missing accounts and additional groups and validates repeated requests. Fallible preparation paths reject invalid beneficiaries, outcome counts, and mint lists before serialization. Rust's infallible `build_init_position_tokens_ix` and `Positions::init_position_tokens_ix` retain their return types. Its fluent builder and transaction helper validate locally.

The eleven-maker limit applies to instruction encoding. Existing transaction helpers retain their legacy transaction types and packet limits. Native transaction-v1 compilation and submission require a separate transport contract. Program instructions contain no transaction resource configuration.

## Considered Options

An optional trailer would create two incompatible instruction layouts behind the
same SDK API. Builders therefore always append it. Changing infallible Rust raw
builders to return `Result` was rejected to preserve existing public signatures.

## Consequences

Consumers that inspect instruction accounts must account for the two trailing
entries. Callers select the SDK version that matches their program deployment and budget compute for the final self-CPI. The SDKs provide no legacy Orderbook conversion or ALT recovery path.

Backend REST and WebSocket models remain separate from authenticated event batches.
