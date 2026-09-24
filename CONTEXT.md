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

**Request-Scoped Relay Adapter**:
The native Rust `RelayContext` and `Relayed<T>` operations are a server adapter for the Dioxus web app, which holds one API Key while forwarding each browser visitor's own cookies and verified country. They are not shared-client authentication state and are not mirrored as Python or TypeScript client methods. This is a deliberate language-specific integration seam, not a difference in the backend wire contract. Python and TypeScript direct clients still use their own API Keys and backend-observed request context; a future multi-visitor server integration in those languages needs its own request-scoped relay design.

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
