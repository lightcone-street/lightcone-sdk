# Bindings

House-style ReScript bindings to the JS libraries the SDK consumes, hand-finished per the
[`rescript-bindgen`](https://github.com/…) skill conventions. Each binding is its own
directory with a consumer-facing `README.md`, a runtime test suite (`tests/`, run under Bun),
a `ReadmeChecks` compile-guard, and a coverage matrix (`tests/README.md`).

| Binding | Library | What it covers |
|---|---|---|
| [`solana-kit`](solana-kit/) | `@solana/kit` | base58 addresses, base16/base58/u64 codecs, ed25519 keys & signing, PDAs, transactions, RPC |
| [`noble-hashes`](noble-hashes/) | `@noble/hashes` | keccak256 — the order-hash primitive `@solana/kit` lacks |
| [`solana-program-token`](solana-program-token/) | `@solana-program/token` | associated-token-account derivation + token / ATA program ids |
| [`decimal`](decimal/) | `decimal.js` | arbitrary-precision decimals (price/size scaling, formatting) |
| [`sorted-btree`](sorted-btree/) | `sorted-btree` | ordered map backing the orderbook state |
| [`fetch`](fetch/) | global `fetch` | the HTTP transport (no axios) |
| [`websocket`](websocket/) | global `WebSocket` | the app-level WebSocket |

These are internal modules of the one SDK package (no per-binding `package.json`/namespace) —
the skill's "single project sources" option — so SDK code references them directly
(`SolanaKit.encode`, `NobleHashes.keccak256`, …). Run all binding tests:

```bash
./node_modules/.bin/rescript build && bun test ./bindings/*/tests/*Test.res.mjs
```
