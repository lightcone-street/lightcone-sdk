# Lightcone Pinocchio Rust SDK

Rust SDK for interacting with the Lightcone Pinocchio program on Solana.

## Features

- **Full instruction coverage**: All 14 program instructions supported
- **Type-safe**: Full Rust type safety with comprehensive error handling
- **Efficient order matching**: Cross-instruction Ed25519 signature verification for minimal transaction size
- **Complete set operations**: Mint and merge conditional tokens
- **Order management**: Create, sign, hash, and cancel orders
- **Position management**: Deposit, withdraw, and track positions
- **Async client**: Built on Tokio for async/await support

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
lightcone-pinocchio-sdk = { git = "https://github.com/lightcone-street/lightcone-sdk/rust" }
```

## Quick Start

```rust
use lightcone_pinocchio_sdk::prelude::*;
use solana_sdk::signer::{keypair::Keypair, Signer};

#[tokio::main]
async fn main() -> Result<(), SdkError> {
    // Initialize client
    let client = LightconePinocchioClient::new("https://api.devnet.solana.com");

    // Load your wallet
    let wallet = Keypair::new(); // Or load from file

    // Get market info (example pubkeys - replace with actual values)
    let market_pda = client.get_market_pda(0);
    let yes_token_mint = Pubkey::new_unique(); // Your YES token mint
    let no_token_mint = Pubkey::new_unique();  // Your NO token mint

    // Get user's current nonce for replay protection
    let nonce = client.get_next_nonce(&wallet.pubkey()).await?;

    // Create and sign an order (expiration 0 = no expiration)
    let mut order = client.create_bid_order(BidOrderParams {
        nonce,
        maker: wallet.pubkey(),
        market: market_pda,
        base_mint: yes_token_mint,
        quote_mint: no_token_mint,
        maker_amount: 100_000,
        taker_amount: 100_000,
        expiration: 0, // Or use a Unix timestamp for expiration
    });
    order.sign(&wallet);

    Ok(())
}
```

## Program Instructions

The SDK supports all 14 Lightcone Pinocchio program instructions:

| # | Instruction | Description |
|---|-------------|-------------|
| 0 | `initialize` | Initialize the exchange (one-time setup) |
| 1 | `create_market` | Create a new market |
| 2 | `add_deposit_mint` | Configure collateral token and create conditional mints |
| 3 | `mint_complete_set` | Deposit collateral to mint YES + NO tokens |
| 4 | `merge_complete_set` | Burn YES + NO tokens to withdraw collateral |
| 5 | `cancel_order` | Cancel an open order |
| 6 | `increment_nonce` | Increment user nonce (invalidates old orders) |
| 7 | `settle_market` | Settle market with winning outcome |
| 8 | `redeem_winnings` | Redeem winning tokens for collateral |
| 9 | `set_paused` | Pause/unpause the exchange (admin) |
| 10 | `set_operator` | Change the operator address (admin) |
| 11 | `withdraw_from_position` | Withdraw tokens from position to wallet |
| 12 | `activate_market` | Activate a pending market |
| 13 | `match_orders_multi` | Match taker against up to 5 makers |

## Client Methods

### Exchange Management

```rust
use lightcone_pinocchio_sdk::prelude::*;

// Initialize exchange
let tx = client.initialize(&authority_pubkey).await?;

// Set paused state
let tx = client.set_paused(&authority_pubkey, true).await?;

// Change operator
let tx = client.set_operator(&authority_pubkey, &new_operator_pubkey).await?;
```

### Market Operations

```rust
// Create market
let tx = client.create_market(CreateMarketParams {
    authority: authority_pubkey,
    num_outcomes: 2,
    oracle: oracle_pubkey,
    question_id: question_id_bytes, // [u8; 32]
}).await?;

// Add deposit mint
let (market_pda, _) = get_market_pda(0, &PROGRAM_ID);
let tx = client.add_deposit_mint(
    AddDepositMintParams {
        payer: authority_pubkey,
        market_id: 0,
        deposit_mint: usdc_mint,
        outcome_metadata: vec![
            OutcomeMetadata { name: "YES".to_string(), symbol: "YES".to_string(), uri: "https://...".to_string() },
            OutcomeMetadata { name: "NO".to_string(), symbol: "NO".to_string(), uri: "https://...".to_string() },
        ],
    },
    &market_pda,
    2, // num_outcomes
).await?;

// Activate market
let tx = client.activate_market(ActivateMarketParams {
    authority: authority_pubkey,
    market_id: 0,
}).await?;

// Settle market
let tx = client.settle_market(SettleMarketParams {
    oracle: oracle_pubkey,
    market_id: 0,
    winning_outcome: 0,
}).await?;
```

### Token Operations

```rust
// Mint complete set (deposit collateral, receive YES + NO)
let tx = client.mint_complete_set(
    MintCompleteSetParams {
        user: user_pubkey,
        market: market_pda,
        deposit_mint: usdc_mint,
        amount: 1_000_000,
    },
    2, // num_outcomes
).await?;

// Merge complete set (burn YES + NO, receive collateral)
let tx = client.merge_complete_set(
    MergeCompleteSetParams {
        user: user_pubkey,
        market: market_pda,
        deposit_mint: usdc_mint,
        amount: 500_000,
    },
    2, // num_outcomes
).await?;

// Withdraw from position
let tx = client.withdraw_from_position(
    WithdrawFromPositionParams {
        user: user_pubkey,
        market: market_pda,
        mint: conditional_mint,
        amount: 100_000,
    },
    false, // is_token_2022
).await?;

// Redeem winnings after settlement
let tx = client.redeem_winnings(
    RedeemWinningsParams {
        user: user_pubkey,
        market: market_pda,
        deposit_mint: usdc_mint,
        amount: 1_000_000,
    },
    0, // winning_outcome
).await?;
```

### Order Management

```rust
use lightcone_pinocchio_sdk::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

// Create and sign orders
let expiration = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs() as i64 + 3600; // 1 hour from now

let mut bid_order = client.create_bid_order(BidOrderParams {
    nonce: client.get_next_nonce(&buyer.pubkey()).await?,
    maker: buyer.pubkey(),
    market: market_pda,
    base_mint: yes_token,
    quote_mint: no_token,
    maker_amount: 100_000,  // NO tokens to give
    taker_amount: 100_000,  // YES tokens to receive
    expiration,
});
bid_order.sign(&buyer);

// Cancel order
let tx = client.cancel_order(&buyer.pubkey(), &bid_order).await?;

// Increment nonce (invalidates all pending orders with old nonce)
let tx = client.increment_nonce(&buyer.pubkey()).await?;
```

### Order Matching

```rust
// Match orders with Ed25519 verification
let tx = client.match_orders_multi_with_verify(MatchOrdersMultiParams {
    operator: operator_pubkey,
    market: market_pda,
    base_mint: yes_token,
    quote_mint: no_token,
    taker_order: bid_order,
    maker_orders: vec![ask_order],
    fill_amounts: vec![100_000],
}).await?;
```

## PDA Derivation

```rust
use lightcone_pinocchio_sdk::pda::*;

// Core PDAs
let (exchange, _) = get_exchange_pda(&program_id);
let (market, _) = get_market_pda(market_id, &program_id);
let (position, _) = get_position_pda(&user, &market, &program_id);
let (conditional_mint, _) = get_conditional_mint_pda(&market, &deposit_mint, outcome_index, &program_id);
let (order_status, _) = get_order_status_pda(&order_hash, &program_id);
let (user_nonce, _) = get_user_nonce_pda(&user, &program_id);

// Additional PDAs
let (vault, _) = get_vault_pda(&deposit_mint, &market, &program_id);
let (mint_authority, _) = get_mint_authority_pda(&market, &program_id);
let all_mints = get_all_conditional_mint_pdas(&market, &deposit_mint, num_outcomes, &program_id);
```

## Account Deserialization

```rust
use lightcone_pinocchio_sdk::accounts::*;

let exchange_data = rpc_client.get_account_data(&exchange_pda)?;
let exchange = Exchange::deserialize(&exchange_data)?;

let market_data = rpc_client.get_account_data(&market_pda)?;
let market = Market::deserialize(&market_data)?;

let position_data = rpc_client.get_account_data(&position_pda)?;
let position = Position::deserialize(&position_data)?;
```

## Order Utilities

```rust
use lightcone_pinocchio_sdk::orders::*;

// Create unsigned order
let mut order = FullOrder::new_bid(BidOrderParams { ... });

// Sign order
order.sign(&keypair);

// Hash order (for order status lookups)
let hash = order.hash();

// Check if orders can match
let can_match = orders_can_cross(&bid_order, &ask_order);

// Calculate fill amount
let fill_amount = calculate_taker_fill(&maker_order, maker_fill_amount)?;

// Verify signature
let is_valid = order.verify_signature()?;
```

## Ed25519 Verification Strategies

The SDK supports three Ed25519 verification strategies:

```rust
use lightcone_pinocchio_sdk::ed25519::*;

// Strategy 1: Individual instructions (simplest)
let ix = create_ed25519_verify_instruction(&Ed25519VerifyParams::from_order(&order));

// Strategy 2: Batch verification (more efficient)
let ix = create_batch_ed25519_verify_instruction(&[
    Ed25519VerifyParams::from_order(&order1),
    Ed25519VerifyParams::from_order(&order2),
]);

// Strategy 3: Cross-instruction references (most efficient - recommended)
let ixs = create_cross_ref_ed25519_instructions(num_makers, match_ix_index);
```

## Client Utility Methods

```rust
// Get all conditional mint addresses for a market
let mints = client.get_conditional_mints(&market, &deposit_mint, num_outcomes);

// Derive condition ID
let condition_id = client.derive_condition_id(&oracle, &question_id, num_outcomes);

// Get user's current nonce (for order creation)
let nonce = client.get_next_nonce(&user_pubkey).await?;

// Get next market ID
let next_id = client.get_next_market_id().await?;
```

## Testing

```bash
# Run unit tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_order_hash_consistency

# Run devnet integration tests (requires funded wallet)
cargo test --test devnet_integration -- --ignored --nocapture
```

## Configuration

Create a `.env` file:

```env
RPC_URL=https://your-rpc-endpoint.com
```

## Program ID

**Devnet**: `Aumw7EC9nnxDjQFzr1fhvXvnG3Rn3Bb5E3kbcbLrBdEk`

## Architecture

The Lightcone Pinocchio program is a high-performance market CLOB (Central Limit Order Book) built with [Pinocchio](https://github.com/anza-xyz/pinocchio) - a zero-dependency, zero-copy Solana program framework.

### Key Design Decisions

1. **Ed25519 Signature Verification**: Orders are signed off-chain and verified on-chain using the Ed25519 program
2. **Cross-instruction References**: Signature data is only stored once in the match instruction, with Ed25519 verify instructions referencing offsets
3. **Position Accounts**: User tokens are held in Position PDAs, enabling atomic matching
4. **Complete Sets**: Users can only mint/burn complete sets of conditional tokens, ensuring market integrity

### Transaction Size Optimization

The SDK uses cross-instruction Ed25519 references to minimize transaction size:

| Approach | Size | Status |
|----------|------|--------|
| Embedded Ed25519 data | ~1,386 bytes | Over limit |
| **Cross-instruction refs** | ~1,040 bytes | Under 1,232 limit |

## Module Structure

```
src/
├── lib.rs              # Public API exports
├── constants.rs        # Program IDs, seeds, discriminators, sizes
├── types.rs            # Enums (MarketStatus, OrderSide) and parameter structs
├── accounts.rs         # Account structs and deserialization
├── pda.rs              # PDA derivation functions
├── orders.rs           # Order types, serialization, hashing, signing
├── instructions.rs     # Instruction builders for all 14 instructions
├── ed25519.rs          # Ed25519 verification instruction helpers
├── client.rs           # Async client with account fetchers and tx builders
├── utils.rs            # Buffer helpers, ATA derivation, validation
└── error.rs            # Custom error types
```

## License

MIT
