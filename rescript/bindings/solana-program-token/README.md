# `SolanaProgramToken` — bindings to `@solana-program/token`

ReScript bindings to [`@solana-program/token`](https://github.com/solana-program/token) (the
Codama-generated SPL Token client), used for **associated-token-account (ATA) derivation** and
the **well-known program addresses** that `@solana/kit` does not bundle.

Written for someone who knows `@solana-program/token` but is new to ReScript bindings. We bind
only what the SDK needs (two address constants + one PDA finder); the upstream client ships the
full SPL Token instruction set.

> Compatibility: peer-compatible with `@solana/kit` **6.x**. Every address here is a
> `SolanaKit.address` — the same branded base58 string type used across the SDK — so values flow
> straight into kit's PDA/instruction APIs without conversion.
>
> See [`tests/`](./tests) for the runnable suite and [`tests/README.md`](./tests/README.md) for
> the coverage matrix.

## Setup

This package depends on the sibling [`@lightcone-sdk/solana-kit`](../solana-kit) package — its
`rescript.json` lists `@lightcone-sdk/solana-kit` (and its `package.json` pins it `workspace:*`) —
so every `SolanaKit.address` value flows straight in. The npm package `@solana-program/token` must
be installed (a package dependency). Reference it directly — `SolanaProgramToken.tokenProgramAddress`.

No trailing-`()` calling convention is needed here: no binding takes a trailing optional argument
(`findAssociatedTokenPda` takes a single record).

## How to read returned values

- **`tokenProgramAddress` / `associatedTokenProgramAddress` are `SolanaKit.address`** — a branded
  base58 string. An address IS a string at runtime, so get the string back with the zero-cost
  `SolanaKit.addressToString(addr)`. Build one from a string with `SolanaKit.address("...")` (it
  validates and brands; it **throws** on an invalid address — catch at the SDK layer).
- **`findAssociatedTokenPda` returns `promise<(SolanaKit.address, int)>`** — a 2-tuple of
  `(ata, bump)`. `await` it and destructure: `let (ata, bump) = await SolanaProgramToken.findAssociatedTokenPda(seeds)`.
  `ata` is the derived ATA address; `bump` is the PDA bump seed (an `int`, 0..255).
- **The input is a record `associatedTokenSeeds`** — build it with all three fields
  (`owner`, `tokenProgram`, `mint`); none are optional, and each is a `SolanaKit.address`.

## Quick start

```rescript
let seeds: SolanaProgramToken.associatedTokenSeeds = {
  owner: SolanaKit.address("So11111111111111111111111111111111111111112"),
  tokenProgram: SolanaProgramToken.tokenProgramAddress,
  mint: SolanaKit.address("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
}
let (ata, bump) = await SolanaProgramToken.findAssociatedTokenPda(seeds)
// SolanaKit.addressToString(ata) == "DHe62eeQVEnNK7vg5xUpDkJm7tuqHadjhvmPRFBG9UPo"
// bump == 254
```

## Reference

### `SolanaProgramToken`

| Binding | Signature | Notes |
|---|---|---|
| `tokenProgramAddress` | `SolanaKit.address` | The SPL Token program address (`TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA`). Imported as `TOKEN_PROGRAM_ADDRESS`. Read as a string with `SolanaKit.addressToString`. |
| `associatedTokenProgramAddress` | `SolanaKit.address` | The Associated Token program address (`ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL`). Imported as `ASSOCIATED_TOKEN_PROGRAM_ADDRESS`. |
| `findAssociatedTokenPda` | `associatedTokenSeeds => promise<(SolanaKit.address, int)>` | Async (SHA-256 PDA). Returns `(ata, bump)`; destructure: `let (ata, bump) = await …`. |

### `associatedTokenSeeds` (input record)

Construct it with `{owner, tokenProgram, mint}` — all three fields are required.

| Field | Type | Notes |
|---|---|---|
| `owner` | `SolanaKit.address` | Wallet that owns the token account. |
| `tokenProgram` | `SolanaKit.address` | Usually `SolanaProgramToken.tokenProgramAddress` (pass the Token-2022 program address instead for a Token-2022 mint). |
| `mint` | `SolanaKit.address` | The token mint. |

## Escape hatches

For other SPL Token surface (mint/transfer instructions, account decoders, …) add ad-hoc
`@module("@solana-program/token")` externals next to these, mirroring their shape. To go from a
`SolanaKit.address` back to a raw `string`, use `SolanaKit.addressToString`; to build one from a
`string`, use `SolanaKit.address`.
