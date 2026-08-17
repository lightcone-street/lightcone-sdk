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

### Wallet Deposit Balances and SOL Conversion

`deposit_token_balances` returns a required exact nine-decimal `native_sol_balance` alongside the mint-keyed SPL `balances`. Initialize `WalletDepositBalancesState` either with `apply_rest_snapshot(wallet, snapshot)` or a complete WebSocket `Snapshot`, then apply typed events from the matching wallet channel. Complete snapshots replace state even when their lower cross-component slot trails a prior update; component events are absolute and only apply to initialized matching-wallet state.

`WalletDepositBalancesState::combined_sol_balance()` returns exact native SOL plus the separately stored canonical WSOL balance. `Positions::wrap_sol(amount, state)` accepts exact no-rounding SOL input, creates the canonical Tokenkeg WSOL ATA idempotently, transfers, syncs, waits for confirmation, and returns the transaction signature. Preflight does not guess a fee or ATA-rent reserve, so wrapping the full cached native balance can still fail on-chain. `Positions::unwrap_wsol(state)` returns a confirmed transaction signature after closing that canonical ATA and crediting its full token balance plus account rent to the wallet. Both methods require live matching credentials, authoritative state, and a signing strategy that controls the wallet; neither mutates state optimistically. A confirmation error does not prove the transaction was rolled back, so refresh authoritative balances before retrying.

The `deposit_token_balances` example is manual-only and runs with `LIGHTCONE_ENV=local` or `staging` only when `SDK_API_URL`, `SDK_WS_URL`, `SDK_RPC_URL`, and `SDK_PROGRAM_ID` are all unset. It uses the SDK-selected WebSocket endpoint to initialize and refresh state, wraps `0.1` SOL, then closes the full canonical WSOL account after observing its exact 0.1 SOL increase. Running it moves funds and closes any pre-existing canonical WSOL balance as well. If it fails after submission, inspect authoritative balances before retrying because funds may already have moved.

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

Build an InitPositionTokens instruction/transaction — create a position account and associated token accounts for all outcomes.

#### `extend_position_tokens_ix` / `extend_position_tokens_tx`

```rust
fn extend_position_tokens_ix(&self, params: &ExtendPositionTokensParams, num_outcomes: u8) -> Result<Instruction, SdkError>
fn extend_position_tokens_tx(&self, params: ExtendPositionTokensParams, num_outcomes: u8) -> Result<Transaction, SdkError>
```

Build an ExtendPositionTokens instruction/transaction — extend a position's lookup table with additional token accounts.

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
