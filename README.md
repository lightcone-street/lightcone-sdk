# Lightcone SDK

Official SDKs for the [Lightcone](https://lightcone.xyz) impact market protocol on Solana.

## SDKs

| Language | Package | Install |
|----------|---------|---------|
| **TypeScript** | [`@lightconexyz/lightcone-sdk`](typescript/) | `npm install @lightconexyz/lightcone-sdk` |
| **Rust** | [`lightcone`](rust/) | `lightcone = { version = "0.3.21", features = ["native"] }` |
| **Python** | [`lightcone-sdk`](python/) | `pip install lightcone-sdk` |

All three SDKs expose the same interface and capabilities.

## Features

- **REST API** - Markets, orderbooks, orders, positions, trades, price history
- **WebSocket streaming** - Real-time orderbook updates, trades, tickers, user events
- **Order signing** - `LimitOrderEnvelope` with human-readable price/size and auto-scaling
- **On-chain operations** - Mint/merge complete sets, increment nonce, PDA derivations
- **Authentication** - Session-based ED25519 signed message flow