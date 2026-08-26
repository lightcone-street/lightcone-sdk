# ADR 0002: Best-Effort Transaction Fee Funding Preflight

- Status: Accepted
- Date: 2026-08-25

## Context

Every SDK-owned on-chain transaction needs Native SOL in its declared fee payer unless an external sponsor pays the network fee. The SOL action planners already own action-specific fee, rent, transfer, and reserve checks, but generic transaction builders can reach RPC simulation with an unfunded fee payer and expose an ambiguous low-level error. The three SDKs need one user-instructive funding contract without blocking all transaction attempts when fee evidence is temporarily unavailable.

## Decision

Rust, TypeScript, and Python expose equivalent typed insufficient-transaction-fee errors with available and required lamports and the message `Insufficient SOL for transaction fees. Deposit SOL to your wallet and try again.` Before an unsponsored SDK-owned on-chain transaction reaches its signer, the shared submission path makes a best-effort check of the exact message fee and declared fee-payer Native SOL Balance. Proven insufficiency returns the typed error; a fee or balance lookup failure after normal RPC behavior continues through the existing signing and submission path because unknown funding is not insufficient funding.

SOL action planners retain their stronger fail-closed cost and freshness rules and use the same typed error when Native SOL cannot cover their action-specific transaction reserve. Asset-amount shortages remain validation errors. A client-wide Transaction Sponsorship Capability defaults to disabled and bypasses the generic fee-funding check when enabled; local-keypair signing rejects that capability, while external and Privy signing treat it as a trusted application assertion. This decision does not implement sponsorship, infer it from a wallet provider, cover off-chain order-message signatures, or cover examples that bypass SDK submission APIs.

## Considered Options

A fixed reserve for every transaction was rejected because generic transactions do not share the SOL planners action costs and valid transactions would be blocked. Translating `AccountNotFound` after submission was rejected because that error does not identify the missing account as the fee payer. Failing closed when generic fee evidence is unavailable was rejected in favor of preserving existing submission availability.

## Consequences

Unsponsored shared submissions add best-effort fee and fee-payer balance reads before signing. A balance change after the check or unavailable evidence can still allow the underlying RPC error to surface. The typed error and sponsorship capability become public cross-language SDK contracts; planner-owned actions continue to require authoritative live cost evidence.
