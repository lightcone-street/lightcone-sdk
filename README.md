# Lightcone SDK

Official SDKs for the [Lightcone](https://lightcone.xyz) impact market protocol.

## SDKs

| Language | Package | Install |
|----------|---------|---------|
| **Rust** | [`lightcone`](rust/) | `cargo add lightcone` |
| **TypeScript** | [`@lightconexyz/lightcone-sdk`](typescript/) | `npm install @lightconexyz/lightcone-sdk` |
| **Python** | [`lightcone-sdk`](python/) | `pip install git+https://github.com/lightcone-street/lightcone-sdk.git@prod#subdirectory=python` |

All three SDKs expose the same interface and capabilities.

## Features

- **REST API** - Markets, orderbooks, orders, positions, trades, price history
- **WebSocket streaming** - Real-time orderbook updates, trades, tickers, user events
- **Order signing** - `LimitOrderEnvelope` with human-readable price/size and auto-scaling
- **On-chain operations** - Mint/merge complete sets, increment nonce, PDA derivations
- **Authentication** - Session-based ED25519 signed message flow

## Development Setup

After cloning, enable the shared git hooks:

```bash
git config core.hooksPath .githooks
```

This enables a pre-commit hook that checks Rust formatting via `cargo fmt --check`. If the check fails, run `cargo fmt --manifest-path rust/Cargo.toml` and re-stage.