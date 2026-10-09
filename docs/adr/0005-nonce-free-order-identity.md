# ADR 0005: Nonce-free order identity

- Status: Accepted for Rust; TypeScript and Python pending
- Date: 2026-10-09

## Decision

The SDK follows the nonce-free program and backend submit contract
([lightcone-pinnochio `f1092ae7`](https://github.com/lightcone-street/lightcone-pinnochio/tree/f1092ae7cc13910528437a8c33bb53c893687a32/src),
ABI unchanged from `9702f231`). An order's identity is its signed fields alone,
with the `u64` salt as the only distinguishing value. The per-user nonce, the
`UserNonce` account, and the `IncrementNonce` instruction are removed. The order
preimage is 161 bytes, the signed order 225 bytes, and the compact order 33
bytes. The wallet signs the lowercase hex of the preimage's Keccak-256 hash.

Salts are drawn from the full `u64` range, and an envelope keeps its salt once
drawn, so one envelope always yields one order. Resubmitting an identical signed
request is the only safe retry after an unknown submission outcome: the backend
answers a repeat with `DUPLICATE_ORDER` or `ALREADY_EXISTS`. Re-signing with a
new salt creates a different order.

Local preflight mirrors the program: a taker fill rounds up
(`mul_div_up`), an order remains valid during its expiration second, and signed
or compact orders decode only at their exact lengths.

## Compatibility

This is a breaking change to the order structs, size constants, signing
preimage, request body, and on-chain helpers; the Rust crate releases it as a
new minor version. TypeScript and Python keep the nonce until they are ported,
and their orders are rejected by a nonce-free backend.

## Validation

Inline known-answer tests pin the backend's order-signing vectors and the
instruction bytes produced by the program team's `lightcone-client` at the
referenced commit. They replace the removed fixture files.
