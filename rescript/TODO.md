# TODO — ReScript SDK

Remaining deferred work and verification caveats. Complete already: the request/response,
auth, order-signing, on-chain transaction-build, RPC-failover, and WS state-container paths.
Source of truth for everything below is the Rust SDK (`../rust/src`).

---

## 1. WebSocket user `snapshot` / `order` wire-tree typing — `src/ws/Messages.res`

**Status:** the last remaining WS piece. The three stateful containers (`OrderbookState`,
`PriceHistoryState`, `DepositPriceState` / `LatestDepositPrice`) are done; what remains is
typing the two largest user payload trees.

- **WS user `snapshot` / `order` events** (`ws/Messages.res` → `UserUpdate.Snapshot` /
  `Order`): carry the largest nested wire trees (open orders, trigger orders, fills). We
  decode + dispatch them but carry the (null-stripped) **raw `JSON.t`** (surfaced to TS as
  `unknown`) instead of fully-typed records.
  *The high-value user events — `BalanceUpdate`, `GlobalDepositUpdate`, `NonceUpdate`,
  `NotificationPush` — are fully typed.*

**To finish:** port the `UserSnapshotOrder` / fill wire trees from Rust `order/state.rs` +
`order/wire.rs`, then replace the two `UserUpdate` arms' raw `JSON.t` with real records
(they are already `@genType`-exported — as `unknown` today).

---

## 2. Position — less-common builders — `src/domain/Position.res`

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
- The WS balance-index / Decimal-math conversions (needed alongside the user snapshot/order
  typing in §1): `From<ConditionalBalanceDelta>`, `ConditionalBalanceDelta`,
  `UserMarketBalanceIndex`, `DepositAssetMetadata`, `TokenBalance::computed_base` / `computed_quote`.

---

## 3. Verification caveats (not code TODOs — confirm against a live chain)

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
