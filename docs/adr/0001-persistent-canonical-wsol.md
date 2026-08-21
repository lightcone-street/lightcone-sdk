# ADR 0001: Persistent Canonical WSOL

- Status: Accepted
- Date: 2026-08-19

## Context

The Trading Wallet can hold SOL in two separately authoritative components: native lamports in its system account and wrapped lamports in its canonical Tokenkeg associated token account. Applications present their sum as one SOL asset, but fund-moving operations must preserve the distinction. Closing the canonical account to obtain native SOL destroys persistent wallet state, can unwrap more than the requested amount, and makes split, merge, redeem, and withdrawal behavior diverge across SDK languages.

SOL actions also need live transaction funding. Account existence, rent exemption, and message fees can change independently of cached wallet balances. A floor without those RPC results is not authoritative enough to enable an action.

## Decision

Rust, TypeScript, and Python expose equivalent SOL component, cost, availability, action-plan, and expected-delta contracts.

- Native SOL Balance is the Trading Wallet system account's exact lamports.
- Canonical wSOL Balance is the exact token amount in its canonical Tokenkeg native-mint ATA.
- Displayed SOL Balance is Native SOL Balance plus Canonical wSOL Balance.
- SOL Transaction Reserve is zero only for an explicitly sponsored action. Otherwise it is the greater of live fee plus required up-front rent and 0.0035 SOL when creating the canonical ATA, or 0.001 SOL otherwise.
- Spendable SOL Balance is Displayed SOL Balance minus SOL Transaction Reserve, provided native SOL can fund the reserve.

Every planner requires initialized matching-wallet state and live RPC results. It returns an unsigned transaction, exact costs and availability, and separate expected native/canonical component deltas. Callers rebuild the plan at their final account-operation boundary, submit through the slot-bearing confirmed API, freeze one projection from the final plan, and restore authority only from a complete snapshot covering the confirmation slot.

## Instruction Ownership

The SDK owns canonical mint/account derivation and instruction order.

- Split consumes canonical WSOL first. Only a shortfall adds idempotent canonical ATA creation when required, native transfer, and `SyncNative` before the market deposit instruction in the same transaction.
- Merge and redeem create the canonical ATA when required and leave all returned WSOL there.
- Native withdrawal transfers directly when native SOL covers the requested amount and reserve.
- A withdrawal requiring canonical WSOL creates a temporary seeded Tokenkeg account, initializes it for the native mint and Trading Wallet owner, transfers only the shortfall from the canonical ATA, closes only the temporary account back to the Trading Wallet, and then transfers the exact requested native lamports to the recipient. These five instructions are one Solana transaction: any instruction failure rolls back the account creation, WSOL transfer, close, and native transfer atomically.
- The canonical WSOL ATA is never closed by a SOL action planner.

## Temporary Account Seed

The preimage is, in order:

1. ASCII `lightcone:wsol-withdraw:v1`
2. One `0x00` terminator byte
3. Raw 32-byte recent blockhash
4. Raw 32-byte Trading Wallet public key
5. Raw 32-byte recipient public key
6. Requested lamports as unsigned 64-bit big-endian bytes
7. Attempt as one unsigned byte

The seed is the first 16 SHA-256 digest bytes encoded as exactly 32 lowercase hexadecimal ASCII characters. Attempts `0` through `7` are checked for account absence; eight collisions return a typed exhaustion error. The wallet is the `CreateAccountWithSeed` base and Tokenkeg is the owner.

## Cross-Language Parity

The three SDKs use the same units, reserve formulas, instruction order, deltas, seed fixture, and eight-attempt bound. Tests decode transactions rather than relying only on instruction counts. The shared seed fixture produces:

- Seed: `4dce744c636478f024df5aefd987f933`
- Tokenkeg address: `71S4MLz9scZhY8BomAjfTkVn6HhFo8yFU7G6tSLto5g6`

## Consequences

Applications can show one SOL asset while retaining component authority, preserve canonical account identity across actions, withdraw an exact native amount to an arbitrary recipient, and fail closed when transaction funding is unknown. Temporary-account rent is required up front even though closing the account refunds it.

Planning does not mutate cached balance state and does not submit transactions. Atomic execution does not eliminate submission uncertainty: a confirmation transport error is not proof that the transaction rolled back or landed, so callers must inspect authoritative chain-backed state before retrying.

## Non-Goals

This decision does not enable gas sponsorship or an explicit unwrap-all operation. Sponsorship policy and authorization remain application concerns, and the current Lightcone Web application enables sponsorship only when both its build capability and a Privy embedded-wallet session permit it. An intentional self-custody action that closes the canonical account is deferred separate work; no planner closes it implicitly. Current application builds pass `sponsored = false`.
