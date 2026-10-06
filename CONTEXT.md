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

**Trading API Key**:
Server-side clients attach an opaque API Key only to submit, cancel, and cancel-all POST requests on their configured API origin. The key identifies an API Consumer, not an Account, and does not replace user authentication or order signatures. Browser callers use their existing Privy session directly. Public reads, authentication, and WebSocket connections do not carry the key.

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

## Temporary Privy Verification Failure

The exact HTTP `503 PRIVY_VERIFICATION_UNAVAILABLE` rejection bypasses SDK retry scheduling, including idempotent and custom policies. Applications choose when to retry. Session checks retain cached credentials because an unavailable authority does not prove that the credentials are invalid.

Structured rejections expose the response status, stable code, and optional retry delay in milliseconds. The delay starts at response receipt. Missing, negative, malformed, nonfinite, or out-of-range guidance remains absent. The SDK rounds fractional milliseconds upward and never schedules a retry for this rejection. Genuine 401 restoration and retry-policy selection retain their existing behavior. Shared delay parsing rejects malformed numeric values and supports HTTP dates for other responses too.
