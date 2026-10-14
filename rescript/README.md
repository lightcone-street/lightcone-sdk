# Lightcone ReScript SDK

The app-facing ReScript SDK, with generated JavaScript and TypeScript APIs. Version
`0.9.0-rc.1` implements the contract at staging commit
`6df479dd401c03c71fab63037b7b0f05f88422f1`. It retains order nonces and ALO. Trigger
orders and ALT operations have been removed from this contract.

Use Node 24+ and Bun 1.3.14 for development. Browser applications use Web Crypto,
Fetch, and WebSocket. The browser entry does not load Node's `ws` transport.

## Install and import

ReScript applications add `@lightconexyz/lightcone-sdk-rescript` to both their
package dependencies and `rescript.json` dependencies. Import through the
`LightconeSdk` namespace; modules such as `Client` no longer occupy the app's global
namespace. The package ships source, interfaces, compiled ESM, and declarations.

```rescript
open LightconeSdk

let client = Client.make(~env=Staging, ())
switch await Market.Client.featured(client) {
| Ok(markets) => Console.log(markets)
| Error(error) => Console.error(SdkError.toMessage(error))
}
```

JavaScript and TypeScript use the same package entry. Its asynchronous facade
throws a `LightconeError` with the structured ReScript error in `.sdkError`.

```ts
import { makeForEnv, MarketClient } from '@lightconexyz/lightcone-sdk-rescript';
const client = makeForEnv('staging');
const markets = await MarketClient.featured(client);
```

The server entry adds native credentials, signed-message login, trading API keys,
and authenticated WebSocket handshakes:

```ts
import { makeForEnv } from '@lightconexyz/lightcone-sdk-rescript';
import { useNativeSigner, login, connect } from '@lightconexyz/lightcone-sdk-rescript/server';
const client = makeForEnv('staging');
// secretKey is the caller's 64-byte Solana keypair.
await useNativeSigner(client, secretKey);
await login(client, undefined);
const socket = connect(client, message => console.log(message), undefined, undefined);
```

`ServerApi` is the equivalent native ReScript surface. Browser signed-message login
and server API keys fail explicitly. Browser cookies and Privy logout belong to
the provider; SDK logout revokes the SDK session and clears cached credentials.

## Exact amounts and order entry

Human prices and sizes are decimal strings. Amounts, salts, nonces, slots, and
transaction lamports use `bigint`. Outgoing large JSON integers remain integer
tokens; incoming integers retain their exact value. Presentation-only float
helpers must not be used to authorize fund movement.

`Orderbook.Client.decimals` retrieves and caches validated `Scaling.OrderbookRules`.
Concurrent reads share a lookup. `invalidateDecimals` and `clearDecimalsCache`
explicitly remove cached rules. Offline `Envelope.buildLimitOrder` requires rules;
`Envelope.submitLimitOrderSigned` fetches them before asking the wallet to sign.
Neither path rounds a user's price or size. Direct signed submission validates the
same amount and field limits.

`Fmt.Decimal.displayExact` returns both `compact` and `expanded` strings. The compact
form can compress leading fractional zeros; the expanded form retains every digit.

## Session and discovery APIs

`Auth.Client.registerPrivy` accepts an attempted identity and always installs the
returned synchronized session. Cached credentials carry the trading wallet and
expiry. `Http.setCredentialRestorer` installs an app-owned callback: concurrent
eligible 401 responses share one bounded restoration attempt, and only retry-safe
requests replay once. Mutating order requests do not replay automatically.

The exact `PRIVY_VERIFICATION_UNAVAILABLE` 503 rejection requires manual retry and
preserves cached credentials. Logout clears local authentication even when server
revocation fails, while preserving that failure result. Explicit per-request
cookies neither replace shared credentials nor invoke restoration.

Markets include signed fee basis points, exact resolution deadlines, validated
outcome counts, optional artwork, and subcategories. `Market.Client.favoriteMarkets`,
`addFavoriteMarket`, and `removeFavoriteMarket` support explicit cookies.
`Market.depositSymbol` displays canonical WSOL as SOL while retaining its mint.

## Prepared v1 transactions

All SDK transaction construction uses v1. Configure explicit resources before
using convenience builders or SOL planners; there is no implicit budget or v0
fallback. Priority fees are **total lamports**, not micro-lamports per compute unit.

```rescript
let resources: LightconeSdk.V1Transaction.Resources.t = {
  computeUnitLimit: 200000,
  loadedAccountsDataSizeLimit: 1048576,
  priorityFeeLamports: 0n,
}
let client = LightconeSdk.Client.make(~env=Staging, ~transactionResources=resources, ())
```

Choose resources appropriate to the actual instruction workload. The values above
illustrate configuration, not a promise that every instruction fits that budget.

`V1Transaction.compile` builds offline using a paired blockhash/expiry context.
`Transactions.build` fetches that context. Fees are estimated against the exact
prepared message. External wallets receive unsigned v1 wire bytes and must return
the same message with every required signature. Altered messages and invalid
signatures fail before submission. Unsupported wallets must report failure.

`Transactions.signAndSubmitPrepared` checks funding, verifies v1 activation,
simulates the signed transaction, and sends it once. `signAndSubmitConfirmed`
additionally returns `{signature, slot}`. `confirm` can reconcile a known signature
using its retained expiry. `SubmissionUnknown`, `ConfirmationTimeout`, and
`TransactionExpired` require reconciliation before building another transaction;
the SDK never automatically rebuilds and resends them.

Unsigned program instructions remain available through `Instructions` and position
builders. Permissionless position initialization accepts a signing payer and a
separate on-curve beneficiary. Conditional withdrawals take the registered deposit
mint and validate the outcome against the on-chain market.

## Live state and SOL plans

The app owns state instances and account activation. `Orderbook.State` replaces both
sides on every accepted frame, retains exact quote notional and truncation metadata,
and gates equal/older sequences within a subscription generation. Call
`beginGeneration` when establishing a fresh subscription; visible levels survive.
A resync result asks the app to refresh its subscription without clearing the book.

`Position.WalletState` consumes complete REST snapshots and
`WalletDepositBalances` events. Complete snapshots replace state even if their slot
is lower than a component update; an optional minimum snapshot slot guards pending
confirmed operations. Component updates apply only to an initialized matching
wallet. Native SOL remains separate from SPL balances.

`Position.Sol` exposes `planSplit`, `planMerge`, `planRedeem`,
`planNativeWithdrawal`, `planWrap`, and `planUnwrapAll`. Plans include the unsigned
transaction, live fee/rent costs, reserve-aware availability, and separate native
and canonical WSOL deltas. They require complete state, valid authenticated wallet
credentials, and a matching signer. SOL planning fails when live costs are unknown.

Ordinary actions support browser wallets and preserve the canonical WSOL account.
Split wraps only its shortfall. Native withdrawal either transfers native SOL or
converts the shortfall through an atomic temporary account. Explicit wrap and
unwrap-all require a native keypair. Wrap rejects unsynchronized donated lamports;
unwrap-all returns the full live canonical account balance. Sponsored SOL planning
is unsupported in this staging contract.

Submit a freshly rebuilt plan through `Position.Sol.submit`, then retain its
projection until a **complete** wallet snapshot reaches the returned confirmation
slot. See [`examples/SolPlan__Example.res`](examples/SolPlan__Example.res).

`Ws.stop` clears queued sends, `restartConnection` permits an auth-driven restart,
and stale socket callbacks cannot publish into the new generation. Authenticated
cleanup removes both user and wallet-balance subscriptions. Native handshakes read
the current token each time; browser handshakes use browser-owned cookies.

## Develop and validate

```bash
bun install --frozen-lockfile
bun run test
bun run typecheck
bun run test:consumers
bunx playwright install chromium
bun run test:browser
```

Set `CHROMIUM_PATH` to use an installed Chromium executable. Browser tests use a
local HTTP/WebSocket server and deterministic signing keys. Offline tests require
no deployed backend and move no funds. Stateful examples remain environment-gated
through the root example runner and disabled in CI with the existing backend gate.

Keep `.res`, `.resi`, generated `.res.mjs`, gentype output, namespace aliases, and
examples synchronized. Tests include Rust-pinned v1 digests, ABI boundaries, exact
order admission, auth recovery, SOL projections, browser signing, and clean packed
consumers. See the [staging parity record](../docs/rescript-staging-parity.md).
