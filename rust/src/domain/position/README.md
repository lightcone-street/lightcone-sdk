# Positions

User portfolio, token balances, and market positions.

[Back to SDK root](../../../README.md)

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

Fetch all positions for a user across all markets. Returns the full portfolio including wallet holdings and conditional token positions.

### `get_for_market`

```rust
async fn get_for_market(
    &self,
    user_pubkey: &str,
    market_pubkey: &str,
) -> Result<MarketPositionsResponse, SdkError>
```

Fetch positions for a user in a specific market.

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
