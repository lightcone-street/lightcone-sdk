# ReScript staging parity

The `0.9.0-rc.1` contract targets staging
`6df479dd401c03c71fab63037b7b0f05f88422f1`, starting from ReScript branch
`c46df9f454805113556d1f5f88d6f626629722cd`. The audit covered 92 non-merge Rust-path
commits and the final changes across 80 Rust files. Unmerged engine-refactor work
is excluded. Nonces and ALO remain supported; trigger orders, jurisdiction/relay
clients, and obsolete ALT operations are excluded from the final staging surface.

| Work package | ReScript implementation and evidence |
|---|---|
| Baseline and consumers | Integrated pinned staging; `LightconeSdk` namespace; production PPX dependency; `scripts/check-consumers.mjs` compiles a packed SDK in a separate app with a colliding `Client` module |
| Exact values and errors | `ExactJson`, `ProtocolInt`, `ExactDecimal`, `Scaling`, `ProgramError`, `SdkError`; exact display and integer-boundary fixtures |
| HTTP and auth | `Http`, `Auth`, browser/server entries; concurrent restoration, SSR isolation, stable request IDs/bodies, Privy outage, logout, registration and preference tests |
| Discovery | Market metadata/outcome validation, favorites and exact cursors, referral codes, environment defaults; `ReadContracts.test.mjs` |
| Program ABI | Current accounts, event suffixes, conditional withdrawal, permissionless initialization, eleven-maker u16 matching; `ProgramTest.res` and `AbiParity.test.mjs` |
| Transactions | Explicit resources, immutable prepared bytes, signature verification, simulation, single-send taxonomy, fee preflight and expiry-aware confirmation; `V1Transaction.test.mjs` and `Transactions.test.mjs` |
| Orders and books | Exact cached rules, validation before signing/direct submission, cryptographic salts, replacement/generation-aware books; `OrderAdmission.test.mjs`, order and book suites |
| Wallet balances | Complete exact snapshots, wallet stream events, matching-wallet reducer and minimum snapshot floors; `WalletState.test.mjs` |
| SOL planning | Six action planners, live canonical-account inspection, fee/rent authority, temporary withdrawal and distinct deltas; `SolPlanning.test.mjs` |
| WebSocket lifecycle | Queueing, stale-generation fencing, stop/restart, private subscription cleanup and native token handshakes; lifecycle and server integration suites |
| Distribution | Browser root and `/server`, clean native/JS/TS consumers, Chromium signing/socket tests, independent PR checks, packaging/RC/release jobs and version promotion |

The three v1 digest vectors and the temporary withdrawal seed/address vector are
pinned to independent expected values in Rust. Corresponding Rust, TypeScript and
Python transaction/position tests provide additional reference coverage. Round-trip
checks are supplemental; they are not the sole byte-compatibility evidence.

ReScript is the app-facing SDK. The backend and on-chain program continue to own
wire semantics. Future contract changes require a recorded disposition for all
four SDKs and equivalent contract fixtures. Language-specific APIs may remain
idiomatic: ReScript returns results; its JS/TS facade throws structured errors.

## App migration

1. Replace global SDK module references with `LightconeSdk` imports.
2. Configure explicit v1 resources and a matching wallet signer.
3. Synchronize Privy registration and install the app's credential restorer.
4. Replace tick-based admission with validated orderbook rules and exact strings.
5. Replace bare wallet maps with complete snapshots and wallet state reducers.
6. Rebuild SOL plans at the account-operation boundary and hold projections until
   a complete snapshot covers the confirmed slot.
7. Tear down account-owned state and subscriptions when the active account changes.
8. Handle uncertain transaction outcomes through reconciliation, never blind retry.

The new package remains a prerelease until the rewritten app validates its own
wallet integrations and deployed-environment flows. Offline tests do not establish
that a particular third-party wallet or RPC deployment supports v1.

## Implementation validation

Validated locally on 2026-10-13:

| Check | Result |
|---|---|
| Frozen Bun installation, SDK build and TypeScript checks | Passed |
| ReScript unit, binding and contract suites | 229 tests passed, 794 assertions |
| Chromium integration | 3 tests passed: browser signing/credentials, socket lifecycle, worker isolation |
| Packed consumers outside the repository | Native ReScript, JavaScript and strict TypeScript passed, including server-to-root socket compatibility |
| Rust v1 transaction and position reference suites | 9 and 58 tests passed |
| TypeScript transaction/SOL/position reference suites | 62 tests passed |
| Python v1 transaction and position reference suites | 93 tests passed |
| Release version promotion and example-runner boundary tests | Passed; runner tests use mocked commands |
| Workflow YAML and Git whitespace checks | Passed |

No funded transactions, live authenticated examples, package publication, or app
rewrite were performed. The pinned staging merge is resolved and remains
uncommitted with the implementation for review.
