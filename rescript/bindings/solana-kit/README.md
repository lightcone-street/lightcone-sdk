# `@lightcone-sdk/solana-kit` — bindings to `@solana/kit`

ReScript bindings to [`@solana/kit`](https://solanakit.com) (modular web3.js v2) — the
Solana/crypto layer of the SDK. Written for someone who knows `@solana/kit` but is new to
ReScript bindings. We bind only the surface the SDK needs.

The surface is split by concern across the package's modules (namespace `SolanaKit`):

| Module | Covers |
|---|---|
| `SolanaKit` (main) | the shared types + base58 addresses (`address`, `isAddress`, `addressToString`), instruction types, `Role` |
| `SolanaKit.Codec` | byte encoders/decoders (`getU64Encoder`, `getBase16Decoder`, `getAddressEncoder`, `encode`, `decode`, …) |
| `SolanaKit.Keys` | ed25519 keypairs + signing (`createKeyPairFromBytes`, `signBytes`, `verifySignature`, `getAddressFromPublicKey`) |
| `SolanaKit.Pda` | `getProgramDerivedAddress` |
| `SolanaKit.Tx` | transaction build / sign / serialize |
| `SolanaKit.Rpc` | Solana JSON-RPC (`createSolanaRpc` + lazy `.send()`) |

> Compatibility: `@solana/kit` 6.x. ESM-only. Native ed25519 on Node ≥18.4 / Bun ≥1.2.6
> (no polyfill needed). All keypair/sign/PDA ops are **async** (WebCrypto). See
> [`tests/`](./tests) and the coverage matrix in [`tests/README.md`](./tests/README.md).

## Setup

Reference the package's modules directly (`SolanaKit`, `SolanaKit.Codec`, …) — a consuming
`rescript.json` lists `@lightcone-sdk/solana-kit` in `dependencies`.

## How to read returned values

- **`address`** is an opaque branded base58 string — `address("…")` validates (throws on
  invalid), `addressToString(a)` reads it back (no runtime call; it *is* a string).
- **Codecs**: `encode(encoder, value) => Uint8Array.t`; `decode(decoder, bytes) => value`.
  base16/base58 treat the hex/base58 **string** as the value, so **bytes → hex** is
  `decode(getBase16Decoder(), bytes)`.
- **Keys**: `cryptoKeyPair` is a record `{privateKey, publicKey}`. `signBytes`/`verifySignature`
  return a `promise`.
- **PDAs / `getAddressFromPublicKey`**: return a `promise` (async); PDA derivation returns the
  tuple `(address, bump)` — destructure it.
- **RPC**: build a request then `await SolanaKit.Rpc.send(request)` (`JSON.t` you decode).

## Quick start

```rescript
let programId = SolanaKit.address("11111111111111111111111111111112")
let (pda, _bump) = await SolanaKit.Pda.getProgramDerivedAddress({
  programAddress: programId,
  seeds: [SolanaKit.Codec.encode(SolanaKit.Codec.getUtf8Encoder(), "central_state")],
})
```

## Escape hatches

For kit functions not bound here, add an ad-hoc `@module("@solana/kit")` external in the
relevant module (`Codec`/`Keys`/`Tx`/`Rpc`), mirroring the existing shapes.
