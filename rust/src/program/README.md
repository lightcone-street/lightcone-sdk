# Program Module

On-chain Solana smart contract interaction for Lightcone markets.

[← Overview](../../README.md#on-chain-program)

## Overview

The program module provides:
- Account type definitions with serialization/deserialization
- Transaction builders for all current on-chain instructions
- PDA derivation utilities
- Order creation, signing, and verification
- Ed25519 order hashing and signature helpers

## Account Types

### Exchange (120 bytes)

Singleton account storing global exchange state.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| discriminator | 0 | 8 | `[u8; 8]` | `EXCHANGE_DISCRIMINATOR` |
| authority | 8 | 32 | `Pubkey` | Can pause exchange and rotate privileged roles |
| operator | 40 | 32 | `Pubkey` | Can match orders and run operational cleanup |
| manager | 72 | 32 | `Pubkey` | Can create and activate markets/orderbooks |
| market_count | 104 | 8 | `u64` | Number of markets created |
| paused | 112 | 1 | `bool` | Whether exchange is paused |
| bump | 113 | 1 | `u8` | PDA bump seed |
| deposit_token_count | 114 | 2 | `u16` | Number of whitelisted deposit tokens |
| _padding | 116 | 4 | - | Reserved |

### Market (148 bytes)

Individual market state.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| discriminator | 0 | 8 | `[u8; 8]` | `MARKET_DISCRIMINATOR` |
| market_id | 8 | 8 | `u64` | Sequential market identifier |
| num_outcomes | 16 | 1 | `u8` | Number of outcomes (2-6) |
| status | 17 | 1 | `MarketStatus` | Pending=0, Active=1, Resolved=2, Cancelled=3 |
| bump | 18 | 1 | `u8` | PDA bump seed |
| _padding | 19 | 5 | - | Reserved |
| oracle | 24 | 32 | `Pubkey` | Resolution authority |
| question_id | 56 | 32 | `[u8; 32]` | Unique question identifier |
| condition_id | 88 | 32 | `[u8; 32]` | Keccak256(oracle \|\| question_id \|\| num_outcomes) |
| payout_numerators | 120 | 24 | `[u32; 6]` | Payout numerators; only first `num_outcomes` entries are meaningful |
| payout_denominator | 144 | 4 | `u32` | Sum of payout numerators |

### Position (80 bytes)

User's custody account for a specific market.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| discriminator | 0 | 8 | `[u8; 8]` | `POSITION_DISCRIMINATOR` |
| owner | 8 | 32 | `Pubkey` | Position owner |
| market | 40 | 32 | `Pubkey` | Market this position belongs to |
| bump | 72 | 1 | `u8` | PDA bump seed |
| _padding | 73 | 7 | - | Reserved |

### OrderStatus (32 bytes)

Tracks order fill state and cancellation.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| discriminator | 0 | 8 | `[u8; 8]` | `ORDER_STATUS_DISCRIMINATOR` |
| remaining | 8 | 8 | `u64` | Remaining amount_in to be filled |
| base_remaining | 16 | 8 | `u64` | Remaining base-side amount tracked for fills |
| is_cancelled | 24 | 1 | `bool` | Whether order has been cancelled |
| _padding | 25 | 7 | - | Reserved |

### UserNonce (16 bytes)

User's nonce for mass order cancellation.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| discriminator | 0 | 8 | `[u8; 8]` | `USER_NONCE_DISCRIMINATOR` |
| nonce | 8 | 8 | `u64` | Current nonce value |

### Orderbook (144 bytes)

On-chain orderbook metadata and lookup table authority.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| discriminator | 0 | 8 | `[u8; 8]` | `ORDERBOOK_DISCRIMINATOR` |
| market | 8 | 32 | `Pubkey` | Market this orderbook belongs to |
| mint_a | 40 | 32 | `Pubkey` | Canonical mint A |
| mint_b | 72 | 32 | `Pubkey` | Canonical mint B |
| lookup_table | 104 | 32 | `Pubkey` | Address lookup table controlled by the orderbook PDA |
| base_index | 136 | 1 | `u8` | Base mint selector, 0 for mint_a and 1 for mint_b |
| bump | 137 | 1 | `u8` | PDA bump seed |
| _padding | 138 | 6 | - | Reserved |

### GlobalDepositToken (48 bytes)

Whitelisted deposit token metadata for global deposits.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| discriminator | 0 | 8 | `[u8; 8]` | `GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR` |
| mint | 8 | 32 | `Pubkey` | Whitelisted deposit mint |
| active | 40 | 1 | `bool` | Whether global deposits are active for this mint |
| bump | 41 | 1 | `u8` | PDA bump seed |
| index | 42 | 2 | `u16` | Sequential whitelist index used for mint ordering |
| _padding | 44 | 4 | - | Reserved |

## LightconeClient — On-Chain Operations

All on-chain operations are available through `LightconeClient` and its domain sub-clients.
Configure with `.rpc_url()` on the builder for RPC-dependent operations.

### Creation

```rust
use lightcone::prelude::*;

// Standard creation with RPC
let client = LightconeClient::builder()
    .rpc_url("https://api.devnet.solana.com")
    .build()?;

// With custom program ID
let client = LightconeClient::builder()
    .rpc_url("https://api.devnet.solana.com")
    .program_id(custom_program_id)
    .build()?;
```

### Account Fetchers

Account fetchers are on the `client.rpc()` sub-client (require RPC):

```rust
let rpc = client.rpc();

// Exchange (singleton)
let exchange = rpc.get_exchange().await?;

// Market by ID or pubkey
let market = rpc.get_market_by_id(0).await?;
let market = rpc.get_market_onchain(&market_pda).await?;

// Position (returns None if not found)
let position = rpc.get_position_onchain(&owner, &market_pda).await?;

// Order status (returns None if not found)
let status = rpc.get_order_status(&order_hash).await?;

// User nonce (returns 0 if account doesn't exist)
let nonce = rpc.get_user_nonce(&user).await?;
let nonce_u32 = rpc.get_current_nonce(&user).await?;

// Next market ID
let next_id = rpc.get_next_market_id().await?;

// Orderbook by mint pair
let orderbook = rpc.get_orderbook_onchain(&mint_a, &mint_b).await?;

// Global deposit token
let gdt = rpc.get_global_deposit_token(&mint).await?;

// Latest blockhash
let blockhash = rpc.get_latest_blockhash().await?;

// Access the underlying Solana RpcClient
let solana_rpc = rpc.inner()?;
```

### Instruction Builders

Instruction builders are organized by domain sub-client. `_ix` methods return an `Instruction` (or `Result<Instruction, SdkError>` for fallible builders). `_tx` convenience methods wrap the instruction in a `Transaction` and return `Result<Transaction, SdkError>`.

**Admin — Exchange Management (`client.admin()`):**
```rust
// Instructions (for custom transaction composition)
let ix = client.admin().initialize_ix(&authority);
let ix = client.admin().set_paused_ix(&authority, true);
let ix = client.admin().set_operator_ix(&authority, &new_operator);
let ix = client.admin().set_authority_ix(&SetAuthorityParams { ... });
let ix = client.admin().set_manager_ix(&SetManagerParams { ... });
let ix = client.admin().whitelist_deposit_token_ix(&WhitelistDepositTokenParams { ... });

// Transactions (ready to sign)
let tx = client.admin().initialize_tx(&authority)?;
let tx = client.admin().set_paused_tx(&authority, true)?;
```

**Admin — Market Lifecycle (`client.admin()`):**
```rust
// create_market_ix is async (fetches next market ID via RPC)
let ix = client.admin().create_market_ix(CreateMarketParams {
    authority,
    oracle,
    question_id,
    num_outcomes,
}).await?;

let ix = client.admin().add_deposit_mint_ix(&AddDepositMintParams {
    authority,
    deposit_mint: usdc_mint,
    outcome_metadata: vec![
        OutcomeMetadata { name: "Yes".into(), symbol: "YES".into(), uri: "".into() },
        OutcomeMetadata { name: "No".into(), symbol: "NO".into(), uri: "".into() },
    ],
}, &market, num_outcomes)?;

let ix = client.admin().activate_market_ix(&ActivateMarketParams {
    authority,
    market_id,
});

let ix = client.admin().settle_market_ix(&SettleMarketParams {
    oracle,
    market_id,
    payout_numerators: vec![7, 3],
})?;
```

**Admin — Operator (`client.admin()`):**
```rust
let ix = client.admin().match_orders_multi_ix(&MatchOrdersMultiParams { ... })?;
let ix = client.admin().deposit_and_swap_ix(&DepositAndSwapParams { ... })?;
```

**Positions — Deposit/Merge/Withdraw (`client.positions()`):**
```rust
// Deposit (dispatches on deposit source: Global or Market)
let ix = client.positions().deposit().await
    .user(keypair.pubkey())
    .mint(deposit_mint)
    .amount(1_000_000)
    .with_market_deposit_source(&market)
    .build_ix()
    .await?;

// Merge (market-only — burns conditional tokens, returns collateral)
let ix = client.positions().merge()
    .user(keypair.pubkey())
    .market(&market)
    .mint(deposit_mint)
    .amount(1_000_000)
    .build_ix()?;

// Withdraw (dispatches on deposit source: Global or Market)
let ix = client.positions().withdraw().await
    .user(keypair.pubkey())
    .mint(deposit_mint)
    .amount(1_000_000)
    .build_ix()
    .await?;
```

**Orders — On-Chain Order Operations (`client.orders()`):**
```rust
let ix = client.orders().cancel_order_ix(&maker, &market, &order);
let ix = client.orders().increment_nonce_ix(&user);
let ix = client.orders().close_order_status_ix(&CloseOrderStatusParams { operator, order_hash });
```

**Positions — Position Management (`client.positions()`):**
```rust
let ix = client.positions().redeem_winnings_ix(&RedeemWinningsParams {
    user, market, deposit_mint: usdc_mint, amount,
}, outcome_index);

let ix = client.positions().withdraw_from_position_ix(&WithdrawFromPositionParams {
    user, market, mint: conditional_mint, amount, outcome_index,
}, is_token_2022);

let ix = client.positions().init_position_tokens_ix(&InitPositionTokensParams {
    payer, user, market, deposit_mints, recent_slot,
}, num_outcomes);

let ix = client.positions().extend_position_tokens_ix(&ExtendPositionTokensParams {
    payer, user, market, lookup_table, deposit_mints,
}, num_outcomes)?;

let ix = client.positions().deposit_to_global_ix(&DepositToGlobalParams {
    user, mint, amount,
});

let ix = client.positions().global_to_market_deposit_ix(&GlobalToMarketDepositParams {
    user, market, deposit_mint, amount,
}, num_outcomes);

let ix = client.positions().close_position_alt_ix(&ClosePositionAltParams {
    operator, position, market, lookup_table,
});

let ix = client.positions().close_position_token_accounts_ix(
    &ClosePositionTokenAccountsParams { operator, market, position, deposit_mints },
    num_outcomes,
)?;
```

**Orderbooks — Cleanup (`client.orderbooks()`):**
```rust
let ix = client.orderbooks().close_orderbook_alt_ix(&CloseOrderbookAltParams {
    operator, orderbook, market, lookup_table,
});

let ix = client.orderbooks().close_orderbook_ix(&CloseOrderbookParams {
    operator, orderbook, market, lookup_table,
});
```

### Order Helpers

Order helpers are on the `client.orders()` sub-client:

```rust
// Create unsigned orders
let order = client.orders().create_bid_order(BidOrderParams {
    nonce: 1,
    maker: pubkey,
    market: market_pda,
    base_mint: yes_token,
    quote_mint: no_token,
    amount_in: 100_000,
    amount_out: 50_000,
    expiration: 0,
});

let order = client.orders().create_ask_order(AskOrderParams { ... });

// Create and sign in one step (native-auth feature)
let order = client.orders().create_signed_bid_order(params, &keypair);
let order = client.orders().create_signed_ask_order(params, &keypair);

// Manual signing
client.orders().sign_order(&mut order, &keypair);

// Get order hash
let hash = client.orders().hash_order(&order);
```

## PDA Derivation

All functions return `(Pubkey, u8)` (address, bump).

| Function | Seeds | Description |
|----------|-------|-------------|
| `get_exchange_pda(program_id)` | `["central_state"]` | Exchange singleton |
| `get_market_pda(market_id, program_id)` | `["market", market_id.to_le_bytes()]` | Market account |
| `get_condition_tombstone_pda(condition_id, program_id)` | `["condition", condition_id]` | Resolved condition tombstone |
| `get_vault_pda(deposit_mint, market, program_id)` | `["market_deposit_token_account", deposit_mint, market]` | Deposit vault |
| `get_mint_authority_pda(market, program_id)` | `["market_mint_authority", market]` | Conditional token mint authority |
| `get_conditional_mint_pda(market, deposit_mint, outcome, program_id)` | `["conditional_mint", market, deposit_mint, [outcome]]` | Conditional token mint |
| `get_order_status_pda(order_hash, program_id)` | `["order_status", order_hash]` | Order fill status |
| `get_user_nonce_pda(user, program_id)` | `["user_nonce", user]` | User nonce account |
| `get_position_pda(owner, market, program_id)` | `["position", owner, market]` | User position |
| `get_orderbook_pda(mint_a, mint_b, program_id)` | `["orderbook", canonical_mint_a, canonical_mint_b]` | Canonical orderbook |
| `get_global_deposit_token_pda(mint, program_id)` | `["global_deposit", mint]` | Whitelisted global deposit token |
| `get_user_global_deposit_pda(user, mint, program_id)` | `["global_deposit", user, mint]` | User global deposit token account |

```rust
use lightcone::program::*;

let (exchange, _) = get_exchange_pda(&program_id);
let (market, _) = get_market_pda(market_id, &program_id);
let (position, _) = get_position_pda(&owner, &market, &program_id);
let (vault, _) = get_vault_pda(&deposit_mint, &market, &program_id);
let (cond_mint, _) = get_conditional_mint_pda(&market, &deposit_mint, 0, &program_id);
```

## Order Types

### OrderPayload (233 bytes)

Complete order with signature for off-chain storage and API submission.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| nonce | 0 | 8 | `u64` | Unique order identifier (must exceed user's on-chain nonce) |
| salt | 8 | 8 | `u64` | Random salt for order uniqueness |
| maker | 16 | 32 | `Pubkey` | Order creator |
| market | 48 | 32 | `Pubkey` | Market pubkey |
| base_mint | 80 | 32 | `Pubkey` | Token being bought (bids) or sold (asks) |
| quote_mint | 112 | 32 | `Pubkey` | Token being given (bids) or received (asks) |
| side | 144 | 1 | `OrderSide` | Bid=0, Ask=1 |
| amount_in | 145 | 8 | `u64` | Amount maker gives (quote for bids, base for asks) |
| amount_out | 153 | 8 | `u64` | Amount maker receives (base for bids, quote for asks) |
| expiration | 161 | 8 | `i64` | Unix timestamp (0 = no expiration) |
| signature | 169 | 64 | `[u8; 64]` | Ed25519 signature |

**Methods:**
```rust
// Construction
OrderPayload::new_bid(params) -> OrderPayload
OrderPayload::new_ask(params) -> OrderPayload
OrderPayload::new_bid_signed(params, &keypair) -> OrderPayload
OrderPayload::new_ask_signed(params, &keypair) -> OrderPayload

// Hashing and signing
order.hash() -> [u8; 32]           // Keccak256 of fields (excludes signature)
order.sign(&keypair)               // Sign in place
order.verify_signature() -> Result<()>

// Serialization
order.serialize() -> [u8; 225]
OrderPayload::deserialize(data) -> Result<OrderPayload>

// Conversion
order.to_order() -> Order
```

### Order (37 bytes)

On-chain transmission format. Excludes maker/market/base_mint/quote_mint (passed via accounts).

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| nonce | 0 | 4 | `u32` | Order nonce |
| salt | 4 | 8 | `u64` | Random salt for order uniqueness |
| side | 12 | 1 | `OrderSide` | Bid=0, Ask=1 |
| amount_in | 13 | 8 | `u64` | Amount maker gives |
| amount_out | 21 | 8 | `u64` | Amount maker receives |
| expiration | 29 | 8 | `i64` | Expiration timestamp |

```rust
let compact = order.to_order();
```

### Order Utilities

```rust
// Check expiration
let expired = is_order_expired(&order, current_timestamp);

// Check if orders can match (bid price >= ask price)
let can_cross = orders_can_cross(&bid_order, &ask_order);

// Calculate taker fill for a given maker fill
let taker_fill = calculate_taker_fill(&maker_order, maker_fill_amount)?;

// Derive condition ID from market parameters
let condition_id = derive_condition_id(&oracle, &question_id, num_outcomes);

// Get all conditional mints for a market
let mints = client.markets().get_conditional_mints(&market, &deposit_mint, num_outcomes);
```

## Ed25519 Signature Verification

Orders are signed off-chain with Ed25519 over `hex(keccak256(order_message))`, where `order_message` is the 169-byte signed field set without the signature. The SDK exposes this through `OrderPayload::hash()`, `OrderPayload::hash_hex()`, `OrderPayload::sign()`, and `OrderPayload::verify_signature()`.

For on-chain matching, the SDK embeds compact `Order` values plus raw signatures directly in `MatchOrdersMulti` and `DepositAndSwap` instruction data. The Pinocchio program verifies those signatures natively; no separate Ed25519 sysvar instruction builder is required in the SDK.

## Instruction Reference

All instructions use a single-byte discriminator.

| Instruction | Discriminator | Description |
|-------------|---------------|-------------|
| Initialize | 0 | Create exchange singleton |
| CreateMarket | 1 | Create a new market |
| AddDepositMint | 2 | Add deposit token to market |
| MintCompleteSet | 3 | Deposit and mint conditional tokens |
| MergeCompleteSet | 4 | Burn conditionals and withdraw deposit |
| CancelOrder | 5 | Cancel a specific order |
| IncrementNonce | 6 | Invalidate all orders with lower nonces |
| SettleMarket | 7 | Resolve market with winning outcome |
| RedeemWinnings | 8 | Redeem winning tokens for deposit |
| SetPaused | 9 | Pause/unpause exchange |
| SetOperator | 10 | Change exchange operator |
| WithdrawFromPosition | 11 | Withdraw tokens from position |
| ActivateMarket | 12 | Activate pending market |
| MatchOrdersMulti | 13 | Match taker against makers |
| SetAuthority | 14 | Change exchange authority |
| CreateOrderbook | 15 | Create an orderbook for a canonical mint pair |
| WhitelistDepositToken | 16 | Whitelist a global deposit token |
| DepositToGlobal | 17 | Deposit collateral to the global pool |
| GlobalToMarketDeposit | 18 | Move global collateral into a market position |
| InitPositionTokens | 19 | Initialize position token accounts and ALT |
| DepositAndSwap | 20 | Deposit collateral and atomically swap |
| ExtendPositionTokens | 21 | Extend a position ALT with more deposit mints |
| WithdrawFromGlobal | 22 | Withdraw collateral from the global pool |
| ClosePositionAlt | 23 | Deactivate or close a position ALT |
| CloseOrderStatus | 24 | Close a fully-filled order status PDA |
| ClosePositionTokenAccounts | 25 | Close empty position token accounts |
| CloseOrderbookAlt | 26 | Deactivate or close an orderbook ALT |
| CloseOrderbook | 27 | Close an orderbook PDA after its ALT is closed |
| SetManager | 28 | Change exchange manager |

## Constants

### Program IDs

The Lightcone program ID is derived from `LightconeEnv` and accessed via `LightconeEnv::program_id()` or `LightconeClient::program_id()`. Other program IDs are constants:

```rust
TOKEN_PROGRAM_ID: Pubkey              // SPL Token
TOKEN_2022_PROGRAM_ID: Pubkey         // Token-2022 (conditional tokens)
ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey
SYSTEM_PROGRAM_ID: Pubkey
RENT_SYSVAR_ID: Pubkey
```

### Discriminators

```rust
EXCHANGE_DISCRIMINATOR: [u8; 8]
MARKET_DISCRIMINATOR: [u8; 8]
ORDER_STATUS_DISCRIMINATOR: [u8; 8]
USER_NONCE_DISCRIMINATOR: [u8; 8]
POSITION_DISCRIMINATOR: [u8; 8]
ORDERBOOK_DISCRIMINATOR: [u8; 8]
GLOBAL_DEPOSIT_TOKEN_DISCRIMINATOR: [u8; 8]
```

### Seeds

```rust
EXCHANGE_SEED: &[u8]                  // b"central_state"
MARKET_SEED: &[u8]                    // b"market"
VAULT_SEED: &[u8]                     // b"market_deposit_token_account"
MINT_AUTHORITY_SEED: &[u8]            // b"market_mint_authority"
CONDITIONAL_MINT_SEED: &[u8]          // b"conditional_mint"
CONDITION_SEED: &[u8]                 // b"condition"
ORDER_STATUS_SEED: &[u8]              // b"order_status"
USER_NONCE_SEED: &[u8]                // b"user_nonce"
POSITION_SEED: &[u8]                  // b"position"
ORDERBOOK_SEED: &[u8]                 // b"orderbook"
GLOBAL_DEPOSIT_TOKEN_SEED: &[u8]      // b"global_deposit"
```

### Sizes

```rust
EXCHANGE_SIZE: usize                  // 120
MARKET_SIZE: usize                    // 148
POSITION_SIZE: usize                  // 80
ORDER_STATUS_SIZE: usize              // 32
USER_NONCE_SIZE: usize                // 16
ORDERBOOK_SIZE: usize                 // 144
GLOBAL_DEPOSIT_TOKEN_SIZE: usize      // 48
SIGNED_ORDER_SIZE: usize              // 233
ORDER_SIZE: usize                     // 37
SIGNATURE_SIZE: usize                 // 64
```

### Limits

```rust
MAX_OUTCOMES: u8                      // 6
MIN_OUTCOMES: u8                      // 2
MAX_MAKERS: usize                     // 5 (per match_orders_multi instruction)
```

## Type Definitions

### MarketStatus

```rust
#[repr(u8)]
pub enum MarketStatus {
    Pending = 0,   // Market created, not yet active
    Active = 1,    // Market accepting orders
    Resolved = 2,  // Market has winning outcome
    Cancelled = 3, // Market cancelled
}
```

### OrderSide

```rust
#[repr(u8)]
pub enum OrderSide {
    Bid = 0,  // Maker buys base, gives quote
    Ask = 1,  // Maker sells base, receives quote
}
```

### Parameter Structs

All parameter structs use owned types (no lifetimes):

```rust
pub struct CreateMarketParams {
    pub manager: Pubkey,
    pub num_outcomes: u8,
    pub oracle: Pubkey,
    pub question_id: [u8; 32],
}

pub struct AddDepositMintParams {
    pub manager: Pubkey,
    pub deposit_mint: Pubkey,
    pub outcome_metadata: Vec<OutcomeMetadata>,
}

pub struct OutcomeMetadata {
    pub name: String,    // max 32 chars
    pub symbol: String,  // max 18 chars
    pub uri: String,     // max 200 chars
}

pub struct ActivateMarketParams {
    pub manager: Pubkey,
    pub market_id: u64,
}

pub struct SettleMarketParams {
    pub oracle: Pubkey,
    pub market_id: u64,
    pub payout_numerators: Vec<u32>,
}

pub struct BuildDepositParams {
    pub user: Pubkey,
    pub market: Pubkey,
    pub deposit_mint: Pubkey,
    pub amount: u64,
}

pub struct BuildMergeParams {
    pub user: Pubkey,
    pub market: Pubkey,
    pub deposit_mint: Pubkey,
    pub amount: u64,
}

pub struct RedeemWinningsParams {
    pub user: Pubkey,
    pub market: Pubkey,
    pub deposit_mint: Pubkey,
    pub amount: u64,
}

pub struct WithdrawFromPositionParams {
    pub user: Pubkey,
    pub market: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
    pub outcome_index: u8,
}

pub struct MatchOrdersMultiParams {
    pub operator: Pubkey,
    pub market: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub taker_order: OrderPayload,
    pub maker_orders: Vec<OrderPayload>,
    pub maker_fill_amounts: Vec<u64>,
    pub taker_fill_amounts: Vec<u64>,
    pub full_fill_bitmask: u8,
}

pub struct BidOrderParams {
    pub nonce: u64,
    pub maker: Pubkey,
    pub market: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub expiration: i64,
}

pub struct AskOrderParams {
    pub nonce: u64,
    pub maker: Pubkey,
    pub market: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub expiration: i64,
}
```

## Error Types

```rust
pub enum SdkError {
    InvalidDiscriminator { expected: String, actual: String },
    AccountNotFound(String),
    InvalidDataLength { expected: usize, actual: usize },
    InvalidOutcomeCount { count: u8 },
    InvalidOutcomeIndex { index: u8, max: u8 },
    InvalidPayoutNumerators,
    PayoutVectorExceedsU32,
    InvalidScalarRange,
    DuplicateScalarOutcomes,
    TooManyMakers { count: usize },
    SignatureVerificationFailed,
    InvalidSignature,
    Serialization(String),
    InvalidSide(u8),
    InvalidMarketStatus(u8),
    MissingField(String),
    Overflow,
    InvalidMintOrder,
    OrderbookExists,
    InvalidMarket,
    MarketSettled,
    InvalidProgramId,
    InvalidOrderbook,
    FullFillRequired,
    DivisionByZero,
    DepositTokenNotActive,
    InsufficientGlobalDeposit,
    InvalidDepositMintOrder,
    ZeroAmount,
    InvalidAta,
    OrderNotFullyFilled,
    PayoutTooSmall,
    TokenAccountNotEmpty,
    LookupTableNotClosed,
    InvalidManager,
    InvalidPubkey(String),
    Scaling(ScalingError),
    UnsignedOrder,
}

pub type SdkResult<T> = Result<T, SdkError>;
```

---

[← Overview](../../README.md#on-chain-program)
