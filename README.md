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

## SOL Account Lifecycle

All three SDKs model native SOL and canonical Tokenkeg WSOL separately while
presenting their sum as one SOL balance. Split wraps only a shortfall; merge and
redeem retain proceeds in the persistent canonical account; native withdrawal
uses native SOL directly or converts only its shortfall through a temporary
seeded account. The conversion instructions share one Solana transaction, so an
instruction failure rolls the entire conversion back atomically. Ordinary
planners never close the canonical account implicitly. Native-keypair users can
instead plan an exact standalone wrap or an explicit no-amount unwrap-all that
returns the account's complete lamports, including rent, to the same Trading
Wallet. The per-language `wsol_conversion` examples rebuild before
signing, submit prepared transactions with confirmed slots, retain each frozen
projection until a complete snapshot covers that slot, and warn that a later SOL
action may recreate the closed account and pay rent. An uncertain submission or
confirmation is never retried automatically; inspect authoritative balances
before planning another action. See the [persistent canonical WSOL
ADR](docs/adr/0001-persistent-canonical-wsol.md).

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) toolchain (via rustup)
- [Node.js](https://nodejs.org/) 22+ and npm
- [Python](https://www.python.org/) 3.12+ and [uv](https://docs.astral.sh/uv/)
- [Solana CLI](https://docs.solanalabs.com/cli/install) (`solana-keygen` for wallet generation)

### Per-SDK Setup

```bash
# TypeScript
cd typescript && npm install

# Python
cd python && uv sync
```

Rust requires no extra setup beyond rustup.

### Git Hooks

After cloning, enable the shared git hooks:

```bash
git config core.hooksPath .githooks
```

The pre-commit hook runs `cargo fmt` on staged Rust files and prompts you to verify SDK examples pass before committing.

### Local Backend (required for examples)

The Lightcone backend must be running locally for SDK examples to work. At minimum you need:

- **backend/api** — REST API server
- **backend/engine** — order matching engine

Set `LIGHTCONE_ENV=local` to point the SDK at the local backend (`https://api.local.lightcone.xyz`).

For Caddy + mkcert TLS setup and running the full local stack, refer to the [web-app repo](https://github.com/lightcone-street/lightcone) setup instructions.

### Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `LIGHTCONE_ENV` | Yes | Target environment: `local`, `staging`, or `prod` |
| `SDK_RPC_URL` | Optional | Solana RPC URL. Use a private devnet RPC (e.g. [Helius](https://www.helius.dev/)) to avoid 429 rate-limit errors from the public `api.devnet.solana.com`. Local `wsol_conversion` runs and eligible staging-CI runs may retain it; other fund-moving examples document stricter guards. |
| `LIGHTCONE_WALLET_PATH` | Yes | Path to Solana keypair JSON for Rust examples |
| `LIGHTCONE_WALLET_PATH_TS` | Yes | Path to Solana keypair JSON for TypeScript examples |
| `LIGHTCONE_WALLET_PATH_PYTHON` | Yes | Path to Solana keypair JSON for Python examples |

The fund-moving `deposit_token_balances` example remains excluded from
`scripts/run-examples.sh`; run it manually with `LIGHTCONE_ENV=local` or
`staging` and all endpoint/program overrides unset. It uses the three existing
wallet paths as a funding cycle (`Rust -> TypeScript -> Python -> Rust`).

`wsol_conversion` runs automatically for every SDK in the local aggregate suite
and is included when the stateful example workflow is enabled for staging CI.
That workflow's global `backend-ready` gate currently disables all stateful CI
jobs. Each language uses its own wallet, wraps a small exact amount, warns that it
will close the complete canonical WSOL account, and unwraps all without a prompt.
The local runner preserves an optional paid RPC while clearing API, WebSocket,
and program overrides so application and program identity remain built in. An
enabled staging-CI run may supply its managed endpoints, but a program-ID
override still fails.
Production skips the example, and the example itself refuses production.

For the routine automatic example suite, add these to your shell profile
(`.bashrc` / `.zshrc`):

```bash
export LIGHTCONE_ENV=local
export SDK_RPC_URL=https://devnet.helius-rpc.com/?api-key=YOUR_KEY
export LIGHTCONE_WALLET_PATH=~/.config/solana/lightcone-sdk-rs.json
export LIGHTCONE_WALLET_PATH_TS=~/.config/solana/lightcone-sdk-ts.json
export LIGHTCONE_WALLET_PATH_PYTHON=~/.config/solana/lightcone-sdk-py.json
```

Before directly running `deposit_token_balances`, use a clean shell or run `unset
SDK_API_URL SDK_WS_URL SDK_RPC_URL SDK_PROGRAM_ID`. Direct local
`wsol_conversion` runs may retain `SDK_RPC_URL` but must unset `SDK_API_URL`,
`SDK_WS_URL`, and `SDK_PROGRAM_ID`; the aggregate local runner performs that
three-variable cleanup in its subprocess.

### Wallet Setup

Three separate keypairs are required so that all SDK examples can run in parallel without race conditions from shared on-chain state. Each wallet needs devnet SOL (for transaction fees) and 100 Lightcone USDC.

**Quick setup** (creates keypairs, airdrops devnet SOL, prints public keys):

```bash
./scripts/setup-wallets.sh
```

The script creates keypairs at `~/.config/solana/lightcone-sdk-{rs,ts,py}.json`, airdrops 2 devnet SOL to the first wallet and transfers 0.5 SOL to each of the others. If the airdrop fails (devnet rate limits), you'll need to send devnet SOL manually.

After the script completes, send 100 Lightcone USDC to each wallet's public key.

**Manual setup:**

```bash
solana-keygen new --no-passphrase -o ~/.config/solana/lightcone-sdk-rs.json
solana-keygen new --no-passphrase -o ~/.config/solana/lightcone-sdk-ts.json
solana-keygen new --no-passphrase -o ~/.config/solana/lightcone-sdk-py.json
```

Then send devnet SOL and 100 Lightcone USDC to each wallet's public key (viewable with `solana-keygen pubkey <path>`).

### Running Examples

Use the shared example runner script:

```bash
# Run all SDKs in parallel
./scripts/run-examples.sh

# Run a single SDK
./scripts/run-examples.sh --sdk rs
./scripts/run-examples.sh --sdk ts
./scripts/run-examples.sh --sdk py

# See all options
./scripts/run-examples.sh --help
```

### Supply Chain Security

All three SDKs enforce a 7-day minimum release age on dependencies to guard against supply chain attacks. Install `cargo-cooldown` so that Rust dependency updates respect the cooldown window:

```bash
cargo install --locked cargo-cooldown
```

Then use `cargo cooldown` in place of `cargo` when updating dependencies (e.g. `cargo cooldown update`). The npm and uv configs are applied automatically via `.npmrc` and `pyproject.toml`.

| SDK | Mechanism | Config |
|-----|-----------|--------|
| Rust | [`cargo-cooldown`](https://crates.io/crates/cargo-cooldown) | `cooldown.toml` — `cooldown_minutes = 10080` |
| TypeScript | npm `min-release-age` | `.npmrc` — `min-release-age=7` |
| Python | uv `exclude-newer` | `pyproject.toml` — `exclude-newer = "7 days"` |
