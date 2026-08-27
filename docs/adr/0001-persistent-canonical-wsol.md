# ADR 0001: Persistent Canonical WSOL

- Status: Accepted
- Date: 2026-08-19

## Context

The Trading Wallet can hold SOL in two separately authoritative components: native lamports in its system account and wrapped lamports in its canonical Tokenkeg associated token account. Applications present their sum as one SOL asset, but fund-moving operations must preserve the distinction. Implicitly closing the canonical account to obtain native SOL destroys persistent wallet state, can unwrap more than the requested amount, and makes split, merge, redeem, and withdrawal behavior diverge across SDK languages.

SOL actions also need live transaction funding. Account existence, account lamports, rent exemption, and message fees can change independently of cached wallet balances. A floor without those RPC results is not authoritative enough to enable an action.

## Decision

Rust, TypeScript, and Python expose equivalent SOL component, cost, availability, action-plan, and expected-delta contracts.

- Native SOL Balance is the exact lamport balance in the Trading Wallet system account.
- Canonical wSOL Balance is the exact token amount in the canonical Tokenkeg native-mint ATA for the Trading Wallet.
- Displayed SOL Balance is Native SOL Balance plus Canonical wSOL Balance.
- SOL Transaction Reserve is zero only for an explicitly sponsored action. Otherwise it is the greater of live fee plus required up-front rent and 0.0035 SOL when creating the canonical ATA, or 0.001 SOL otherwise.
- Spendable SOL Balance is Displayed SOL Balance minus SOL Transaction Reserve, provided native SOL can fund the reserve.

Every planner requires initialized matching-wallet state and live RPC results. It returns an unsigned transaction, exact costs and availability, and separate expected native/canonical component deltas. Callers rebuild the plan at their final account-operation boundary, submit through the slot-bearing confirmed API, freeze one projection from the final plan, and restore authority only from a complete snapshot covering the confirmation slot.

## Instruction Ownership

The SDK owns canonical mint/account derivation and instruction order.

- Split consumes canonical WSOL first. Only a shortfall adds idempotent canonical ATA creation when required, native transfer, and `SyncNative` before the market deposit instruction in the same transaction. `SyncNative` is the Token Program instruction that recalculates the WSOL token amount from account lamports minus the native-account rent reserve.
- Merge and redeem create the canonical ATA when required and leave all returned WSOL there.
- Native withdrawal transfers directly when native SOL covers the requested amount and reserve.
- A withdrawal requiring canonical WSOL creates a temporary seeded Tokenkeg account, initializes it for the native mint and Trading Wallet owner, transfers only the shortfall from the canonical ATA, closes only the temporary account back to the Trading Wallet, and then transfers the exact requested native lamports to the recipient. These five instructions are one Solana transaction: any instruction failure rolls back the account creation, WSOL transfer, close, and native transfer atomically.
- Split, merge, redeem, native withdrawal, orders, and other ordinary SOL actions never call a standalone conversion planner and never close the canonical WSOL ATA.

## Explicit Native Conversion

The three SDKs expose equivalent, standalone native-keypair planners for two explicit conversions. They are never composed into ordinary SOL actions.

- Exact-amount wrap accepts positive integer lamports, creates or reuses only the canonical Tokenkeg ATA for the authenticated Trading Wallet, transfers the exact amount, and synchronizes the native mint. Admission preserves the standard SOL Transaction Reserve floors and includes live fee plus required account rent. Its component delta includes the wrapped amount, fee, and any new account rent.
- Unwrap-all accepts no amount, requires a positive authoritative Canonical wSOL Balance and a live validated canonical account, closes only that account to the same Trading Wallet, and does not support partial conversion. Admission requires Native SOL Balance to cover the freshly estimated fee only. Its native delta is the complete live lamport value of the account, including refunded rent, minus the fee; its canonical delta removes the full token balance.

Both planners reject wallet-adapter and Privy signing strategies and do not enable sponsored planning. Consuming applications invoke unwrap-all only through a clearly explicit user action, warn that the persistent canonical account will close and later SOL actions may recreate it and pay rent, rebuild immediately before signing, submit through the prepared slot-bearing confirmation API, retain the frozen projection from the final plan, and restore action authority only after a complete snapshot covers the confirmed slot. Signing, planning, submission, or confirmation failures never trigger automatic resubmission; uncertain outcomes require authoritative refresh before another attempt.

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

The three SDKs use the same units, reserve formulas, instruction order, deltas, signer restrictions, full-account semantics, and native conversion lifecycle. Tests decode transactions rather than relying only on instruction counts. Non-production examples may act on an existing canonical balance, print the exact close and future-rent warning, wrap a small exact amount, unwrap the complete resulting account without an interactive pause, and refresh complete state past each confirmed slot. The shared runner executes them for every SDK wallet in local aggregate runs and includes them in staging CI when the globally gated stateful example workflow is enabled; that workflow currently disables all stateful CI jobs. Production remains forbidden.

The shared temporary-withdraw seed fixture produces:

- Seed: `4dce744c636478f024df5aefd987f933`
- Tokenkeg address: `71S4MLz9scZhY8BomAjfTkVn6HhFo8yFU7G6tSLto5g6`

## Consequences

Applications can show one SOL asset while retaining component authority, preserve canonical account identity across ordinary actions, explicitly convert native SOL to canonical WSOL, explicitly recover the full canonical account as native SOL, withdraw an exact native amount to an arbitrary recipient, and fail closed when transaction funding is unknown. Temporary-account and canonical-account rent are exact component movements rather than hidden value.

Planning does not mutate cached balance state and does not submit transactions. Atomic execution does not eliminate submission uncertainty: a confirmation transport error is not proof that the transaction rolled back or landed, so callers must inspect authoritative chain-backed state before retrying.

## Non-Goals

This decision does not enable Web self-custody, expose standalone conversion in Lightcone Web, permit Privy or wallet-adapter conversion signing, enforce the backend authentication-method policy for native SDK users, support partial canonical unwrap, enable gas sponsorship, change protocol-held deposits or accounting, or restore implicit canonical-account closure. Broader native-user authentication enforcement remains separate future work.
