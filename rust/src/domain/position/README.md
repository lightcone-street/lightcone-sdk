# Positions

User portfolio, token balances, and market positions.

[← Overview](../../../README.md#positions)

## Table of Contents

- [Types](#types)
- [Client Methods](#client-methods)
- [Examples](#examples)
- [Wire Types](#wire-types)

## Types

### `Portfolio`

A user's full portfolio across all markets.

| Field | Type | Description |
|-------|------|-------------|
| `user_address` | `PubkeyStr` | User's wallet address |
| `wallet_holdings` | `Vec<WalletHolding>` | Non-conditional token balances (SOL, USDC, etc.) |
| `positions` | `Vec<Position>` | Per-market conditional token positions |
| `total_wallet_value` | `Decimal` | Total USD value of wallet holdings |
| `total_positions_value` | `Decimal` | Total USD value of all positions |

### `Position`

A user's position in a single market.

| Field | Type | Description |
|-------|------|-------------|
| `event_pubkey` | `PubkeyStr` | Market public key |
| `event_name` | `String` | Market name |
| `event_img_src` | `String` | Market image URL |
| `outcomes` | `Vec<PositionOutcome>` | Per-outcome token balances |
| `total_value` | `Decimal` | Total USD value of this position |
| `created_at` | `DateTime<Utc>` | When the position was opened |

### `PositionOutcome`

One outcome within a position.

| Field | Type | Description |
|-------|------|-------------|
| `condition_id` | `u8` | Outcome index |
| `condition_name` | `String` | Outcome name (e.g., "Yes") |
| `token_mint` | `PubkeyStr` | Conditional token mint |
| `amount` | `Decimal` | Token balance |
| `usd_value` | `Decimal` | USD value at current price |

### `WalletHolding`

A non-conditional token balance in the user's wallet.

| Field | Type | Description |
|-------|------|-------------|
| `token_mint` | `PubkeyStr` | Token mint address |
| `symbol` | `String` | Token symbol (e.g., "USDC") |
| `amount` | `Decimal` | Token balance |
| `decimals` | `u64` | Token decimals |
| `usd_value` | `Decimal` | USD value |
| `img_src` | `String` | Token icon URL |

## Client Methods

Access via `client.positions()`.

### `get`

```rust
async fn get(&self, user_pubkey: &str) -> Result<PositionsResponse, SdkError>
```

Fetch all positions for a user across all markets. Public path-based endpoint; no auth required. Returns the full portfolio including wallet holdings and conditional token positions.

### `get_for_market`

```rust
async fn get_for_market(
    &self,
    user_pubkey: &str,
    market_pubkey: &str,
) -> Result<MarketPositionsResponse, SdkError>
```

Fetch positions for a user in a specific market. Public path-based endpoint; no auth required.

### `positions`

```rust
async fn positions(&self) -> Result<PositionsResponse, SdkError>
```

Fetch all positions for the **authenticated** user across every market. The wallet is resolved server-side from the `auth_token` cookie. See [the Authentication section](../../../README.md#authentication) for the cookie / `_with_cookies` story.

### `positions_for_market`

```rust
async fn positions_for_market(
    &self,
    market_pubkey: &str,
) -> Result<MarketPositionsResponse, SdkError>
```

Fetch positions for the authenticated user in a specific market.

### `positions_with_cookies` / `positions_for_market_with_cookies` / `deposit_token_balances_with_cookies`

Same as the no-arg authed variants above, but accept a raw `Cookie` header containing `privy-token` and/or `lightcone-token`. For SSR / Dioxus server-function callers that need to forward the per-request cookie without writing it into shared client state. See [the Authentication section](../../../README.md#authentication).

### Wallet Deposit Balances and SOL Action Planning

`deposit_token_balances` returns a required exact nine-decimal `native_sol_balance` alongside the mint-keyed SPL `balances`. Initialize `WalletDepositBalancesState` either with `apply_rest_snapshot(wallet, snapshot)` or a complete WebSocket `Snapshot`, then apply typed events from the matching wallet channel. Existing methods preserve unconditional complete-snapshot replacement even when the lower shared snapshot slot trails a prior update. Confirmation-aware consumers can call `apply_rest_snapshot_with_minimum_snapshot_slot` or `apply_event_with_minimum_snapshot_slot`; only a complete snapshot below the supplied floor is ignored without mutation, while equal slots and all balance/status events retain their normal behavior. Balance update events are absolute and only apply to initialized matching-wallet state. A matching SPL update with a negative idle balance returns `Rejected` without changing balances or the context slot.

`WalletDepositBalancesState::combined_sol_balance()` returns exact native SOL plus the separately stored canonical WSOL balance. `sol_balance_breakdown()` narrows each balance to Solana's `u64` transaction range. `plan_sol_split`, `plan_sol_merge`, `plan_sol_redeem`, and `plan_native_sol_withdrawal` return an unsigned `SolActionPlan` containing live fee/rent costs, the action-specific reserve and spendable balance, and separate expected native/canonical deltas. Missing RPC estimates, an incomplete or wrong-wallet snapshot, insufficient native reserve, and sponsored requests fail closed; sponsorship remains unsupported until a concrete sponsor owns fees and account rent. An occupied canonical address is accepted only when it decodes as the wallet's initialized, unfrozen Tokenkeg native-mint account. `canonical_wsol_account_info` exposes that exact validated inspection as full account lamports (including excess), decoded token-amount lamports, and decoded native-reserve lamports, while `canonical_wsol_account_exists` preserves the boolean API by delegating to it.

Split plans consume canonical WSOL first and add idempotent ATA creation, native transfer, and `SyncNative` only for a shortfall. `SyncNative` is the Token Program instruction that recalculates the WSOL token amount from account lamports minus the native-account rent reserve. Merge and redeem plans create the canonical account when absent and always retain proceeds there. Native withdrawal transfers directly when native SOL covers amount plus reserve; otherwise it converts only the shortfall through a bounded, seeded temporary Tokenkeg account, closes that temporary account back to the Trading Wallet, and transfers the exact requested native amount to the recipient. The temporary account's create, initialize, WSOL transfer, close, and native transfer instructions share one Solana transaction, so an instruction failure rolls the entire conversion back atomically. These ordinary planners never close the persistent canonical account.

Native-keypair consumers have two explicit standalone conversion planners. `plan_wrap_sol(amount_lamports, state)` creates the canonical ATA only when absent, transfers the exact positive amount, and runs `SyncNative`; an existing account must have no unsynchronized donated lamports because `SyncNative` would otherwise increase WSOL by more than the requested amount. Standard reserve floors apply and its native delta includes only the amount, live fee, and newly funded rent. No-amount `plan_unwrap_wsol_all(state)` requires positive cached WSOL exactly matching the live token amount, then closes the canonical account to and under the authority of the same Trading Wallet. It accepts unsynchronized excess and returns every live account lamport, including rent and donated excess, after checking that the final native balance remains within `u64`. Its `SolActionCosts` fields are always unsponsored, zero upfront rent, and no account creation; its availability reserves only the live fee, which native SOL must fund before the refund is available. Browser wallet adapters and Privy signers are intentionally rejected only for these explicit conversion planners.

Submit the final rebuilt plan with `sign_and_submit_prepared_tx_confirmed_with_slot`; this preserves the fee-estimated message and rejects external-wallet message replacement. Atomic execution does not resolve uncertain submission or confirmation errors, so refresh authoritative balances instead of automatically retrying. Hold the balance projection until a complete snapshot covering the returned slot restores action authority. See [`docs/adr/0001-persistent-canonical-wsol.md`](../../../../docs/adr/0001-persistent-canonical-wsol.md).

The `deposit_token_balances` example is manual-only and runs with `LIGHTCONE_ENV=local` or `staging` only when `SDK_API_URL`, `SDK_WS_URL`, `SDK_RPC_URL`, and `SDK_PROGRAM_ID` are all unset. It sends 0.001 SOL to the TypeScript SDK wallet configured by `LIGHTCONE_WALLET_PATH_TS`, confirms it with a slot, waits for the wallet stream to cover that slot, and refreshes a complete snapshot at that slot. Running it moves funds. If it fails after submission, inspect authoritative balances before retrying because funds may already have moved.

The `wsol_conversion` example runs automatically for every SDK wallet in local aggregate runs and is included when the globally gated stateful example workflow is enabled for staging CI; that workflow currently disables all stateful CI jobs. Local runs may use a paid RPC while retaining built-in API, WebSocket, and program identity; an enabled staging-CI run may supply its managed endpoints. Direct staging runs remain override-free. The example requires a native keypair matching the authenticated Trading Wallet, previews and rebuilds an exact 0.001 SOL wrap, retains the confirmed projection through a covering refresh, then warns and rebuilds unwrap-all immediately before submission. Unwrap-all closes the full canonical account, including pre-existing WSOL and rent; the example exits without automatic resubmission on any uncertain error.

### On-Chain Instruction & Transaction Builders

Each operation has an `_ix` method returning an `Instruction` (or `Result<Instruction, SdkError>` for fallible builders) and a `_tx` convenience method returning `Result<Transaction, SdkError>`.

#### `redeem_winnings_ix` / `redeem_winnings_tx`

```rust
fn redeem_winnings_ix(&self, params: &RedeemWinningsParams, outcome_index: u8) -> Instruction
fn redeem_winnings_tx(&self, params: RedeemWinningsParams, outcome_index: u8) -> Result<Transaction, SdkError>
```

Build a RedeemWinnings instruction/transaction — redeem conditional tokens for collateral after market resolution.

#### `withdraw_conditional_from_position_ix` / `withdraw_conditional_from_position_tx`

```rust
fn withdraw_conditional_from_position_ix(&self, params: &WithdrawConditionalFromPositionParams) -> Instruction
fn withdraw_conditional_from_position_tx(&self, params: WithdrawConditionalFromPositionParams) -> Result<Transaction, SdkError>

// Compatibility wrappers:
fn withdraw_from_position_ix(&self, params: &WithdrawFromPositionParams) -> Instruction
fn withdraw_from_position_tx(&self, params: WithdrawFromPositionParams) -> Result<Transaction, SdkError>
```

Build a conditional-token withdrawal instruction/transaction. The params take the market's registered `deposit_mint`; the SDK derives the conditional mint from `(market, deposit_mint, outcome_index)` and withdraws from the position's canonical conditional-token ATA to the user's canonical ATA.

The fluent `withdraw_from_position()` and `withdraw_conditional_from_position()` builders only receive a market pubkey, so callers must pass `.num_outcomes(market.num_outcomes)` before building. Unified market deposit, merge, and withdrawal use `Market::num_outcomes` directly rather than the length of display outcome metadata.

#### `init_position_tokens_ix` / `init_position_tokens_tx`

```rust
fn init_position_tokens_ix(&self, params: &InitPositionTokensParams, num_outcomes: u8) -> Instruction
fn init_position_tokens_tx(&self, params: InitPositionTokensParams, num_outcomes: u8) -> Result<Transaction, SdkError>
```

Build an InitPositionTokens instruction/transaction — create a position account and associated token accounts for all outcomes. Permissionless and idempotent: replaying with the same `recent_slot` reuses the existing lookup table and skips groups already present, so retries must reuse the original slot. At most `MAX_DEPOSIT_MINTS_PER_IX` deposit mints per call.

#### `extend_position_tokens_ix` / `extend_position_tokens_tx`

```rust
fn extend_position_tokens_ix(&self, params: &ExtendPositionTokensParams, num_outcomes: u8) -> Result<Instruction, SdkError>
fn extend_position_tokens_tx(&self, params: ExtendPositionTokensParams, num_outcomes: u8) -> Result<Transaction, SdkError>
```

Build an ExtendPositionTokens instruction/transaction — extend a position's lookup table with additional token accounts. Permissionless: `params.payer` is any signer and pays for new accounts. Groups already present in the table are skipped on chain, so existing and new mints may be passed together.

#### `deposit_to_global_ix` / `deposit_to_global_tx`

```rust
fn deposit_to_global_ix(&self, params: &DepositToGlobalParams) -> Instruction
fn deposit_to_global_tx(&self, params: DepositToGlobalParams) -> Result<Transaction, SdkError>
```

Build a DepositToGlobal instruction/transaction — deposit collateral into the global deposit pool for cross-market use.

#### `global_to_market_deposit_ix` / `global_to_market_deposit_tx`

```rust
fn global_to_market_deposit_ix(&self, params: &GlobalToMarketDepositParams, num_outcomes: u8) -> Instruction
fn global_to_market_deposit_tx(&self, params: GlobalToMarketDepositParams, num_outcomes: u8) -> Result<Transaction, SdkError>
```

Build a GlobalToMarketDeposit instruction/transaction — move collateral from the global deposit pool into a specific market position.

#### `close_position_alt_ix` / `close_position_alt_tx`

```rust
fn close_position_alt_ix(&self, params: &ClosePositionAltParams) -> Instruction
fn close_position_alt_tx(&self, params: ClosePositionAltParams) -> Result<Transaction, SdkError>
```

Build a ClosePositionAlt instruction/transaction — deactivate or close a resolved position lookup table.

#### `close_position_token_accounts_ix` / `close_position_token_accounts_tx`

```rust
fn close_position_token_accounts_ix(&self, params: &ClosePositionTokenAccountsParams, num_outcomes: u8) -> Result<Instruction, SdkError>
fn close_position_token_accounts_tx(&self, params: ClosePositionTokenAccountsParams, num_outcomes: u8) -> Result<Transaction, SdkError>
```

Build a ClosePositionTokenAccounts instruction/transaction — close empty position-owned conditional token accounts.

### Deposit / Withdraw / Merge Builders

The preferred way to build deposit, withdraw, and merge instructions.

#### `deposit`

```rust
async fn deposit(&self) -> DepositBuilder<'a>
```

Create a `DepositBuilder` pre-seeded with the client's deposit source. Chain `.user()`, `.mint()`, `.amount()`, then call `.build_ix()` or `.build_tx()`.

For market deposits, use `.with_market_deposit_source(&market)` or `.market(&market)` if the client is already configured with `DepositSource::Market`.

#### `withdraw`

```rust
async fn withdraw(&self) -> WithdrawBuilder<'a>
```

Create a `WithdrawBuilder` pre-seeded with the client's deposit source. Chain `.user()`, `.mint()`, `.amount()`, then call `.build_ix()` or `.build_tx()`.

For market withdrawals, `.mint()` is the registered deposit mint and the SDK derives the conditional mint from `.outcome_index()`. Position withdrawals are conditional-token only; collateral exits through global withdrawal, complete-set merge, or winnings redemption.

#### `merge`

```rust
fn merge(&self) -> MergeBuilder<'a>
```

Create a `MergeBuilder` for burning a complete set of conditional tokens and releasing collateral. Chain `.user()`, `.market(&market)`, `.mint()`, `.amount()`, then call `.build_ix()` or `.build_tx()`.

## Examples

### Check portfolio across all markets

```rust
use lightcone::prelude::*;

async fn show_portfolio(client: &LightconeClient, wallet: &str) -> Result<(), SdkError> {
    let portfolio = client.positions().get(wallet).await?;

    println!("Wallet holdings: ${}", portfolio.total_wallet_value);
    for holding in &portfolio.wallet_holdings {
        println!("  {} {}: ${}", holding.amount, holding.symbol, holding.usd_value);
    }

    println!("\nPositions: ${}", portfolio.total_positions_value);
    for position in &portfolio.positions {
        println!("  {} (${}):", position.event_name, position.total_value);
        for outcome in &position.outcomes {
            println!("    {}: {} tokens (${:.2})", outcome.condition_name, outcome.amount, outcome.usd_value);
        }
    }

    Ok(())
}
```

### Check position in a specific market

```rust
use lightcone::prelude::*;

async fn check_market_position(
    client: &LightconeClient,
    wallet: &str,
    market_pubkey: &str,
) -> Result<(), SdkError> {
    let response = client.positions().get_for_market(wallet, market_pubkey).await?;
    println!("Position: {:?}", response);
    Ok(())
}
```

## Wire Types

Raw response types are available in `lightcone::domain::position::wire`, including `PositionsResponse` and `PositionResponse`.

`global_deposits` on positions REST responses uses the REST shape:

```rust
pub struct GlobalDeposit {
    pub deposit_mint: String,
    pub symbol: String,
    pub balance: Decimal,
}
```

This differs from the WebSocket user snapshot shape in `domain::order::wire`, which uses `{ mint, balance }`.

---

[← Overview](../../../README.md#positions)
