# `WebCrypto` — binding to the Web Crypto global

ReScript binding to the global [`crypto`](https://developer.mozilla.org/en-US/docs/Web/API/Crypto)
object (the Web Crypto API — Node 19+, Bun, and browsers). Currently just `randomUUID`,
used for client-generated ids: the HTTP `x-request-id` (`Http`) and order salts (`Order`).
Keeping it here means `src/` holds no inline `crypto` `@val external`.

> Not to be confused with `bindings/noble-hashes` (keccak256) or `bindings/solana-kit`
> `SolanaKitKeys` (ed25519 via `crypto.subtle`) — this is only the top-level `crypto`
> helpers the SDK needs.

## Setup

Part of the SDK package (no extra `rescript.json` dependency — `crypto` is a platform
global). Reference directly: `WebCrypto.randomUUID()`. No `open` needed.

## What it covers

| Binding | JS | Notes |
|---|---|---|
| `randomUUID() => string` | `crypto.randomUUID()` | a random v4 UUID string (36 chars) |

See [`tests/`](./tests) for the runnable suite.
