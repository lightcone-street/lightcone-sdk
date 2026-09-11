# ADR 0004: Solana v1 transaction cutover

- Status: Accepted across Rust, TypeScript, Python, and the Rust admin SDK
- Date: 2026-09-11

## Decision

Each language exposes a validated v1 transaction type. Compilation requires
caller-selected compute units, loaded account bytes, total priority fee lamports,
optional heap bytes, and a blockhash with its last valid block height. No resource
defaults are silently selected by the SDK. Import rejects legacy/v0, malformed or
oversized wire bytes, ComputeBudget instructions, and mismatched contexts.

Signing and wallet-response validation preserve the complete prepared message
and verify every required signature. Every send checks v1 activation, simulates
the signed message without blockhash replacement, and sends once with preflight.
Ambiguous submissions retain the original signature and expiry for reconciliation;
the SDK never automatically rebuilds, re-signs, or resubmits them.

The existing Privy backend transaction endpoint returns only a transaction hash,
so it cannot meet the signed-message validation contract. Its transaction helpers
fail before sending any request. A v1-capable external signer returning signed
bytes is the supported wallet integration. Privy off-chain order signing remains
available. Unsponsored external signers must expose the fee-payer identity.

## Compatibility

The program ABI from SDK PR #164 remains unchanged, including eleven-maker u16
masks, canonical GlobalDepositToken accounts, and the 176-byte orderbook. This
decision changes the outer Solana transaction format and the APIs that own it.
Rust uses the upstream Solana compiler and wincode codec. TypeScript matches
Rust's account privilege merging and raw-address ordering while using Solana Kit
for canonical encoding and decoding; Kit's compiler rejects writable invoked
program accounts that Rust permits. Python uses solders. Python requires 3.11 or newer for the compatible
solana-py/solders dependency pair. Package dependencies pin the supported codec
versions; legacy transport libraries are retained only for compatible instruction,
address, and RPC helpers.

The SOL reserve, integer units, instruction ordering, canonical WSOL lifecycle,
and temporary seed preimage remain governed by ADR 0001. Every SOL plan retains
its final blockhash expiry. Changing budgets requires a fresh plan, fee estimate,
and signatures. Generic fee-funding preflight remains best-effort under ADR 0002;
SOL planners keep their stricter live fee, rent, and reserve requirements.

## Parity validation

All three SDKs test the same canonical vectors in
`rust/src/program/fixtures/solana_v1_transactions.json`. The vectors were generated
with the pinned Rust compiler and codec and cover zero and maximum priority fees,
optional heap encoding, signer/account ordering, merged privileges and repeated
references, writable invoked program accounts, 64 inline addresses, and the
4,096-byte signed transaction limit.
Each suite checks message bytes, unsigned and signed transaction bytes, required
signers, signatures, and immutable wallet acceptance. Language-local tests cover
invalid imports, changed messages, resource bounds, funding, and submission errors.
