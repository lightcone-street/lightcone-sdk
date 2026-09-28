# ADR 0005: Sponsored External Submission

- Status: Accepted for the Rust SDK
- Date: 2026-09-25

## Context

Lightcone Web signs in-app Trading Wallet transactions through a Privy embedded wallet. Privy can pay a Solana network fee for that wallet. To do so, Privy receives the prepared transaction and submits it with sponsorship. Privy may replace the fee payer and the recent blockhash before it sends the transaction. Privy then returns only the transaction signature.

ADR 0001 and ADR 0004 assume that the SDK holds the exact signed message. The SDK simulates that message, keeps its blockhash expiry, and rejects a changed message. Those assumptions cannot hold when an external wallet service rewrites and sends the transaction. Before this decision, every SOL planner rejected `sponsored = true`, and no sponsored send path existed.

## Decision

### Scope

Only the Rust SDK supports sponsored submission and sponsored SOL planning. The only consumer is Lightcone Web, which runs the Rust SDK in WebAssembly with an active Privy browser session. The TypeScript and Python SDKs keep rejecting sponsored SOL planning with a validation error. This is an explicit exception to Cross-Language Parity. Adding sponsorship to TypeScript or Python requires a new decision.

### Submission

1. A client built with `transaction_sponsorship(true)` and an external signer submits through `ExternalSigner::send_sponsored_transaction`.
2. The signer's `wallet_address()` must equal the prepared transaction's fee payer. Otherwise the send fails with a validation error before any request.
3. The SDK calls the signer once. It does not sign locally, does not simulate, and does not send through Solana RPC.
4. The default trait method returns `SponsoredSubmissionError::Rejected`. An ordinary adapter therefore never falls back to an unsponsored send.
5. The signer returns the submitted signature. A signature that does not parse counts as an unknown outcome.

### Outcomes

| Signer result | Meaning | SDK error |
| --- | --- | --- |
| `Ok(signature)` | The wallet service accepted and sent the transaction. | Confirmation continues. |
| `Rejected(reason)` | The wallet service accepted no request. | `SdkError::Signing(reason)` |
| `Unknown` | A request may have landed without a trustworthy response. | `SdkError::SponsoredSubmissionUnknown` |

The SDK never replays a sponsored transaction. After `SponsoredSubmissionUnknown`, callers must check wallet activity and authoritative balances before another attempt.

### Confirmation

The SDK does not know the blockhash that the wallet service submitted. Confirmation therefore has no expiry bound. Only the poll limit ends the wait. An unconfirmed signature at that limit returns `SdkError::ConfirmationTimeout`, which is an unresolved outcome, not a failure.

### Funding

For a sponsored plan, the SOL Transaction Reserve equals the up-front rent that the plan's instructions charge to the Trading Wallet. It has no network fee and no safety floor. The action principal stays wallet-funded.

| Action | Rent counted in a sponsored reserve |
| --- | --- |
| Split | The canonical wSOL account when absent. The Position PDA and each conditional token account when they hold no data. |
| Merge, redeem | The canonical wSOL account when absent. |
| Native withdrawal through canonical wSOL | The temporary seeded account. The same transaction closes it and refunds its rent. |
| Direct native withdrawal | None. |

The program funds only the missing rent for an address that already holds lamports but no data. `account_creation_top_up` counts that gap only. A native balance below the sponsored reserve returns `SdkError::InsufficientSolForAccountRent` instead of the fee error.

Whether a sponsor can pay these rent amounts is not proven. The rule above assumes that the Trading Wallet pays every account that an instruction names it to fund. A controlled sponsored transaction must confirm this rule before production enables sponsorship. Change this table when execution evidence shows a different payer.

## Superseded Statements

This decision replaces these statements for sponsored submission only:

- ADR 0001, Decision: "SOL Transaction Reserve is zero only for an explicitly sponsored action."
- ADR 0001, Decision: planners reject sponsored requests.
- ADR 0001, Action Flow: signing may not replace the fee payer or blockhash, and every v1 transaction retains its blockhash expiry.
- ADR 0001, Cross-Language Parity: equal reserve formulas across all three SDKs.
- ADR 0001, Non-Goals: "enable gas sponsorship".
- ADR 0004, Decision: every send simulates the signed message without blockhash replacement.
- ADR 0004, Compatibility: every SOL plan retains its final blockhash expiry.
- ADR 0004, Decision: a v1-capable external signer that returns signed bytes is the only supported wallet integration.

Unsponsored submission keeps every contract in ADR 0001 and ADR 0004.

## Consequences

An eligible Privy Trading Wallet can run actions that need no wallet-funded rent with zero native SOL. Callers must present `SponsoredSubmissionUnknown` and `ConfirmationTimeout` as unresolved outcomes. The SDK cannot detect a sponsored transaction that landed after an unknown result. Durable recovery across reloads and tabs is outside this decision.
