# `NobleHashes` — bindings to `@noble/hashes`

ReScript bindings to [`@noble/hashes`](https://github.com/paulmillr/noble-hashes), used
for **keccak256** — the one hash primitive `@solana/kit` does not ship. The Lightcone
program hashes the 169-byte order message with Keccak256 (see `program/OrderPayload.res`).

Written for someone who knows `@noble/hashes` but is new to ReScript bindings. We bind
only what the SDK needs (one function); the upstream library has much more.

> Compatibility: `@noble/hashes` **v2** — the sha3 family ships from `@noble/hashes/sha3.js`
> (v1 used `@noble/hashes/sha3`). `keccak_256` is a callable `CHash`.
>
> See [`tests/`](./tests) for the runnable suite and [`tests/README.md`](./tests/README.md)
> for the coverage matrix.

## Setup

The module is part of the SDK package (no extra `rescript.json` dependency). `@noble/hashes`
must be installed (it is an SDK dependency). Reference it directly — `NobleHashes.keccak256`.

## How to read returned values

`keccak256` takes a `Uint8Array.t` and returns a `Uint8Array.t` (the 32-byte digest). To
turn bytes into a value:

- **bytes → hex string**: use `@solana/kit`'s base16 *decoder*: `SolanaKit.getBase16Decoder()->SolanaKit.decode(digest)` (lowercase, matching Rust's `hex::encode`).
- **string → bytes** (to hash a string): `SolanaKit.getUtf8Encoder()->SolanaKit.encode("abc")`.

## Quick start

```rescript
let digest = SolanaKit.encode(SolanaKit.getUtf8Encoder(), "abc")->NobleHashes.keccak256
let hex = SolanaKit.decode(SolanaKit.getBase16Decoder(), digest)
// hex == "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45"
```

## Reference

### `NobleHashes`

| Binding | Signature | Notes |
|---|---|---|
| `keccak256` | `Uint8Array.t => Uint8Array.t` | One-shot Keccak256. Returns the 32-byte digest. Imported from `@noble/hashes/sha3.js` as `keccak_256` (a callable `CHash`). |

## Escape hatches

For other hash functions (sha256, blake3, …), add an ad-hoc `@module("@noble/hashes/<algo>.js")`
external next to `keccak256`, mirroring its shape.
