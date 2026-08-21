# Lightcone SDK Context

This repository owns language-specific SDK contracts and their cross-language parity.
Published Lightcone behavior and terminology remain owned by `lightcone-street/docs`; this
file only names SDK-local concepts and ownership.

## Ownership

**SDK Contract**:
The public Rust, Python, and TypeScript declarations, exports, exact units, errors, and wire
representations maintained in this repository.

**Wire Contract**:
The backend owns payload meaning. This repository owns compatible decoding, validation, and
language-specific representation of those payloads.

**Cross-Language Parity**:
Equivalent observable behavior across all three SDKs, expressed with each language's naming,
numeric, and error conventions.
_Avoid_: Identical APIs

## SDK Terms

**SOL Action Plan**:
An unsigned, fee-prepared transaction plus the costs, availability, and component projection
used to authorize that exact message. The account lifecycle is owned by
`docs/adr/0001-persistent-canonical-wsol.md`.

**Prepared Transaction**:
A transaction whose message, including its fee payer and recent blockhash, was used for fee
estimation. Submission may add signatures but may not replace message fields.

**Canonical WSOL Account**:
The persistent Tokenkeg associated token account referenced by SOL planning contracts. Use the
ADR for its lifecycle rather than restating that definition elsewhere.
