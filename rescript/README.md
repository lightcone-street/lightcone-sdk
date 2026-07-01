# Lightcone ReScript SDK

An idiomatic **ReScript** SDK for the [Lightcone](https://lightcone.xyz) protocol, authored in
ReScript and exported to **TypeScript** via [gentype](https://rescript-lang.org/docs/manual/latest/typescript-integration).
Behavior mirrors the Rust SDK (the source of truth).

It is consumable from three surfaces, all generated from one ReScript codebase:

| Surface | What you import | Error handling |
|---|---|---|
| **ReScript** | the domain modules (`Market`, `Trade`, `Auth`, …) | `promise<result<_, SdkError.t>>` (idiomatic) |
| **JavaScript** | the compiled `*.res.mjs` (esm) | same runtime as ReScript |
| **TypeScript** | the namespaced gentype API `TypeScriptApi.gen.ts` (package entry) | throwing `Promise<T>`, grouped: `Markets.featured`, `Orders.submitLimit`, … |

## Stack

- **ReScript 12.3** + **bun** for dev, npm-publishable ESM output (`.res.mjs`, in-source).
- [`@mununki/ppx-spice`](https://github.com/green-labs/ppx_spice) generates JSON encoders/decoders for
  every wire/domain type; **gentype** emits `.gen.ts` for TS consumers.
- [`@solana/kit`](https://solanakit.com) for the Solana/crypto layer (ed25519 signing, PDAs,
  transactions, RPC) + [`@noble/hashes`](https://github.com/paulmillr/noble-hashes) for keccak256 +
  [`@solana-program/token`](https://www.npmjs.com/package/@solana-program/token) for ATAs. Bindings to
  `decimal.js` and `sorted-btree`. HTTP is the platform-global **Fetch** (no axios); the app WebSocket
  is the platform-global `WebSocket`. House-style bindings live in [`bindings/`](bindings/).

## Layout

```
bindings/   per-library dirs (solana-kit, noble-hashes, solana-program-token, decimal,
            sorted-btree, fetch, websocket) — each with its own README + runtime tests
src/
  Shared      enums/newtypes (Side, TimeInForce, Resolution, DepositSource, …)
  SdkError    error tree + ApiResponse decoding + the result→throwing facade helper
  Env, Http   environment config + Fetch transport (retry, cookies, x-request-id)
  Client      the client handle (HTTP, env, program id, RPC, signer, deposit source, nonce)
  Auth        nonce + ed25519 signed-message login, session, logout
  domain/     Market, Orderbook, Trade, Position, PriceHistory, Metrics, Notification, Referral, Faucet, Order
  domain/     OrderbookState, PriceHistoryState, DepositPriceState — WS "apply events → live view" reducers
              (live sorted book via sorted-btree; rolling candle series); consumed from a Ws onMessage closure
  program/    Constants, Pda, OrderPayload (keccak256 + ed25519 order hashing), Scaling, Instructions,
              Envelope (limit-order build/sign/submit), PositionBuilders (deposit/withdraw/merge/redeem), Accounts
  Rpc         on-chain reads over @solana/kit (latest blockhash, account data, user nonce)
  ws/         Ws (connect/reconnect/heartbeat), Subscriptions, Messages
  TypeScriptApi  the gentype-exported, namespaced TypeScript API (package entry; `@genType module`s)
examples/     ReScript (.res) + TypeScript (.ts) examples, co-located; the compiled .res.mjs is the JS
              example. Each is a `<Name>__Example.res` / `__Example.ts` entry point — the `__Example`
              suffix keeps the module name a single valid identifier (editor/LSP-friendly) and distinct
              from the SDK/domain modules (`Orderbook__Example` doesn't shadow the `Orderbook` domain).
              Shared helper: `Common__Example.res`. TypeScript examples import only from the generated
              `.gen.ts` — never the compiled .res.mjs.
tests/        runtime tests (bun): bindings, codecs, order-hash & scaling golden vectors, PDAs
```

## Usage

**ReScript** (idiomatic `result`):

```rescript
let client = Client.make(~env=Staging, ())
switch await Market.featured(client) {
| Ok(markets) => Console.log(Array.length(markets))
| Error(error) => Console.error(SdkError.toMessage(error))
}
```

**TypeScript** (throwing, namespaced, via gentype — imported from the package entry):

```ts
import { makeForEnv, Markets, login } from "@lightconexyz/lightcone-sdk-rescript";
const client = makeForEnv("staging");
const markets = await Markets.featured(client); // Promise<marketSearchResult[]>
await login(client, undefined);            // throws on failure
```

See [`examples/`](examples/) for both surfaces (and `examples/*.res.mjs` for the compiled JS).

## Develop

```bash
bun install
bun run build          # rescript compile + gentype
bun test ./tests/*Test.res.mjs ./bindings/*/tests/*Test.res.mjs
bun x tsc --noEmit -p tsconfig.json   # typecheck the gentype output + TS examples
```

## Conventions (for contributors)

- **String enums**: set BOTH `@as("wire")` and `@spice.as("wire")` so the ReScript runtime value, the
  JSON wire value, and the gentype TS union all match (e.g. `Side` → `"bid" | "ask"`).
- **Decimals** (prices/sizes) stay as wire **strings** in domain types (no precision loss, gentype-clean);
  wrap in `Decimal` for math. Timestamps/ids/counts are `float`; on-chain amounts are `bigint`.
- **Optional fields** use `field?: T`; the HTTP layer strips JSON `null` before decoding so absent and
  `null` both map to `None` (spice's `?`-field decoder rejects an explicit `null` otherwise).
- **Internally/adjacently-tagged unions** (serde `#[serde(tag=…)]`) are hand-decoded (spice can't derive
  them) — see `Auth.userIdentityDecode` / `SdkError.parseApiResponse`.
- No `namespace` in `rescript.json`: it makes gentype route cross-module types through an un-generated
  bundle. Without it, `.gen.ts` files import types directly and resolve under `tsc`.
