# TODO — ReScript SDK

Deferred work and verification caveats. None of these touch the request/response,
auth, order-signing, or on-chain transaction-build paths — those are complete.
Source of truth for everything below is the Rust SDK (`../rust/src`).

---

## 1. RPC failover — `src/Rpc.res`

**Status:** functional gap. We use the **primary** `client.rpc` only.

**What Rust does** (`../rust/src/rpc_failover.rs`): the client holds a primary + a
backup RPC URL. On an **infrastructure** error (connection refused, 5xx, timeout —
*not* a logical "account not found"), it flips the active endpoint to the backup,
retries, and applies a **cooldown** before probing the primary again. `active_rpc()`
reports which is live. The `rpc_failover` example proves it: call #1 hits a dead
primary → transparently retries on the backup → succeeds; call #2 goes straight to
the backup with no retry delay.

**What we have:** `Rpc.res` builds the kit RPC from the primary only;
`getLatestBlockhash` / `getAccountData` hit it and, on failure, return an `SdkError`
with no automatic backup retry. Every read still works against the primary.

**To finish:**
- Error classification (infrastructure vs logical).
- Mutable `activeRpc` + cooldown state on `Client.t` / `Rpc`.
- A second kit RPC instance for the backup URL.
- A try-primary → flip-on-infra-error → retry wrapper around each RPC call.

`examples/LCEX__RpcFailover.res` already configures primary + backup and will work
transparently once this lands.

---

## 2. WebSocket stateful containers — `src/domain/Orderbook.res`, `src/domain/PriceHistory.res`, `src/ws/Messages.res`

**Status:** largest deferred piece — one cohesive subsystem. The WS **transport,
reconnect/heartbeat, subscription serialization, and message decoding are done**;
what remains is the stateful "apply WS deltas to maintain a live view" layer from
Rust's `*/state.rs`, plus the largest payload trees.

- **`OrderbookState`** (`../rust/src/domain/orderbook/state.rs`): a live book built by
  applying `book_update` deltas — bids/asks in a sorted structure (the reason
  `sorted-btree` is bound) with `bestBid` / `bestAsk` / `midPrice` / depth helpers.
  *The one-shot REST depth snapshot (`Orderbook.get`) is ported.*
- **`PriceHistoryState` / `DepositPriceState` / `LatestDepositPrice`**: rolling candle
  series maintained from `price_history` snapshot/update/heartbeat events (and the
  deposit-price equivalent), with apply/snapshot/update helpers.
  *The REST `PriceHistory.get` / `getLineData` / `getDepositAssetPricesSnapshot` are ported.*
- **WS user `snapshot` / `order` events** (`ws/Messages.res` → `UserUpdate.Snapshot` /
  `Order`): carry the largest nested wire trees (open orders, trigger orders, fills).
  We decode + dispatch them but carry the (null-stripped) **raw `JSON.t`** instead of
  fully-typed records, and leave those two arms off `@genType`.
  *The high-value user events — `BalanceUpdate`, `GlobalDepositUpdate`, `NonceUpdate`,
  `NotificationPush` — are fully typed.*

**To finish:** port the `*/state.rs` reducers (using the already-installed
`sorted-btree` + `decimal` bindings) and the full `UserSnapshotOrder` / fill wire
trees; then type the two `UserUpdate` arms and `@genType`-export them.

---

## 3. Position — less-common builders — `src/domain/Position.res`

**Done:** the core builders (`depositToGlobal`, `withdrawFromGlobal`,
`globalToMarketDeposit`, `merge`, `redeemWinnings` — each builds + signs + sends a
Solana tx, in `program/PositionBuilders.res`), the `initPositionTokens` /
`incrementNonce` instructions (`program/Instructions.res`), and the on-chain reads
(`getExchange` / `getMarket` / `getOrderbook` / `getPosition` in `Rpc.res`).

**Deferred:**
- Market-level direct `deposit` / `withdraw`.
- `extendPositionTokens`.
- `closePositionAlt` / `closePositionTokenAccounts` / `withdrawFromPosition`.
- The low-level `_ix` / `_tx` variants (return an unsigned instruction/transaction for
  the caller to assemble, vs the high-level build+sign+send we ported).
- The WS balance-index / Decimal-math conversions (part of the WS state work in §2):
  `From<ConditionalBalanceDelta>`, `ConditionalBalanceDelta`, `UserMarketBalanceIndex`,
  `DepositAssetMetadata`, `TokenBalance::computed_base` / `computed_quote`.

---

## 4. Verification caveats (not code TODOs — confirm against a live chain)

These compile and are verified against the Rust source, but compile-time cannot prove
the on-chain program accepts them. The CI on-chain examples (against staging) are the
real test.

- **Blockhash-lifetime projection** (`program/PositionBuilders.res`): the tx lifetime is
  wired by projecting `getLatestBlockhash().value` into kit's
  `setTransactionMessageLifetimeUsingBlockhash` via a one-line `%raw`. It matches kit's
  expected shape but is the one piece not directly mirrored from Rust — wants a real
  send against staging to confirm.
- **Instruction / account byte layouts** (`program/Instructions.res`,
  `program/Accounts.res`): account orders, 1-byte opcodes, little-endian data packing,
  and decoder byte-offsets are verified **byte-for-byte against the Rust source**.

---

## Done — formerly TODO, now resolved (kept for context)

- **`getNonce`** (`src/domain/Order.res`) — reads the on-chain UserNonce PDA via
  `Rpc.getNonce`. The envelope still defaults nonce to 0 when the caller doesn't supply
  one; fetch a fresh value via `getNonce` for live submission.
- **Core program layer** — order keccak256 + ed25519 signing, PDAs, scaling, the order
  submit/cancel path, the position transaction builders (§3), and the on-chain account
  reads are all ported and tested.
