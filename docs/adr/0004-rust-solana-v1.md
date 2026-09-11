# ADR 0004: Rust Solana v1 transaction cutover

- Status: Accepted for the Rust admin SDK integration
- Date: 2026-09-11

Rust consumers and the Rust-only admin SDK share one validated v1 transaction
type. Compilation requires explicit resources and a blockhash/expiry context.
Import rejects legacy/v0, malformed or oversized wire bytes, and mismatched
contexts. Signing and wallet-response validation preserve the exact message.
Canonical wincode encoding replaces transaction bincode. Every send verifies
signatures, checks v1 activation, simulates without blockhash replacement, and
sends once with preflight. Unknown sends retain their signature and expiry.

The SOL reserve, integer units, instruction ordering, canonical WSOL lifecycle,
and temporary seed preimage remain governed by ADR 0001. Every SOL plan now
retains its final blockhash expiry; changing budgets requires a fresh plan and
signatures. No resource defaults are silently selected by the SDK.

Parity review: TypeScript and Python already have the same upgraded program ABI,
including eleven-maker u16 masks and the 176-byte orderbook. Their transaction
declarations, exports, docs, and tests still use web3.js/solders legacy outer
transactions. This Rust-only implementation does not certify those transaction
paths for v1. They require separate dependency and wallet migrations before use
in a v1-only application. Their existing instruction and SOL lifecycle fixtures
remain the comparison baseline for the unchanged business semantics.
