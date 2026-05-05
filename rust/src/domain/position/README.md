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

Fetch all positions for the **authenticated** user across every market. The wallet is resolved server-side from the `auth_token` cookie. See [the Authentication section](../../../README.md#authentication) for the cookie / `_with_auth` story.

### `positions_for_market`

```rust
async fn positions_for_market(
    &self,
    market_pubkey: &str,
) -> Result<MarketPositionsResponse, SdkError>
```

Fetch positions for the authenticated user in a specific market.

### `positions_with_auth` / `positions_for_market_with_auth` / `deposit_token_balances_with_auth`

Same as the no-arg authed variants above, but accept an explicit `auth_token: &str`. For SSR / Dioxus server-function callers that need to forward the per-request cookie. See [the Authentication section](../../../README.md#authentication).

### On-Chain Instruction & Transaction Builders

Each operation has an `_ix` method returning an `Instruction` (or `Result<Instruction, SdkError>` for fallible builders) and a `_tx` convenience method returning `Result<Transaction, SdkError>`.

#### `redeem_winnings_ix` / `redeem_winnings_tx`

```rust
fn redeem_winnings_ix(&self, params: &RedeemWinningsParams, outcome_index: u8) -> Instruction
fn redeem_winnings_tx(&self, params: RedeemWinningsParams, outcome_index: u8) -> Result<Transaction, SdkError>
```

Build a RedeemWinnings instruction/transaction — redeem conditional tokens for collateral after market resolution.

#### `withdraw_from_position_ix` / `withdraw_from_position_tx`

```rust
fn withdraw_from_position_ix(&self, params: &WithdrawFromPositionParams, is_token_2022: bool) -> Instruction
fn withdraw_from_position_tx(&self, params: WithdrawFromPositionParams, is_token_2022: bool) -> Result<Transaction, SdkError>
```

Build a WithdrawFromPosition instruction/transaction — withdraw conditional tokens from a position account to the user's wallet.

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

For market withdrawals, use `.with_market_deposit_source(&market)` to set the deposit source and market at once, then chain `.outcome_index()` and `.token_2022()` as needed.

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
    pub balance: String,
}
```

This differs from the WebSocket user snapshot shape in `domain::order::wire`, which uses `{ mint, balance }`.

---

[← Overview](../../../README.md#positions)
