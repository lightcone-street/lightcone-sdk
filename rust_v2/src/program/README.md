# Program Module

On-chain Solana smart contract interaction for Lightcone markets.

## Overview

The program module provides:
- Account type definitions with serialization/deserialization
- Transaction builders for all 14 instructions
- PDA derivation utilities
- Order creation, signing, and verification
- Ed25519 signature verification strategies

## Account Types

### Exchange (88 bytes)

Singleton account storing global exchange state.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| discriminator | 0 | 8 | `[u8; 8]` | `"exchange"` |
| authority | 8 | 32 | `Pubkey` | Can pause exchange, set operator, create markets |
| operator | 40 | 32 | `Pubkey` | Can match orders |
| market_count | 72 | 8 | `u64` | Number of markets created |
| paused | 80 | 1 | `bool` | Whether exchange is paused |
| bump | 81 | 1 | `u8` | PDA bump seed |
| _padding | 82 | 6 | - | Reserved |

### Market (120 bytes)

Individual market state.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| discriminator | 0 | 8 | `[u8; 8]` | `"market\0\0"` |
| market_id | 8 | 8 | `u64` | Sequential market identifier |
| num_outcomes | 16 | 1 | `u8` | Number of outcomes (2-6) |
| status | 17 | 1 | `MarketStatus` | Pending=0, Active=1, Resolved=2, Cancelled=3 |
| winning_outcome | 18 | 1 | `u8` | Winning outcome index (255 if unresolved) |
| has_winning_outcome | 19 | 1 | `bool` | Whether market has resolved |
| bump | 20 | 1 | `u8` | PDA bump seed |
| _padding | 21 | 3 | - | Reserved |
| oracle | 24 | 32 | `Pubkey` | Resolution authority |
| question_id | 56 | 32 | `[u8; 32]` | Unique question identifier |
| condition_id | 88 | 32 | `[u8; 32]` | Keccak256(oracle \|\| question_id \|\| num_outcomes) |

### Position (80 bytes)

User's custody account for a specific market.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| discriminator | 0 | 8 | `[u8; 8]` | `"position"` |
| owner | 8 | 32 | `Pubkey` | Position owner |
| market | 40 | 32 | `Pubkey` | Market this position belongs to |
| bump | 72 | 1 | `u8` | PDA bump seed |
| _padding | 73 | 7 | - | Reserved |

### OrderStatus (24 bytes)

Tracks order fill state and cancellation.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| discriminator | 0 | 8 | `[u8; 8]` | `"ordstat\0"` |
| remaining | 8 | 8 | `u64` | Remaining amount_in to be filled |
| is_cancelled | 16 | 1 | `bool` | Whether order has been cancelled |
| _padding | 17 | 7 | - | Reserved |

### UserNonce (16 bytes)

User's nonce for mass order cancellation.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| discriminator | 0 | 8 | `[u8; 8]` | `"usrnonce"` |
| nonce | 8 | 8 | `u64` | Current nonce value |

## LightconePinocchioClient

### Creation

```rust
use lightcone_sdk::program::LightconePinocchioClient;

// Standard creation
let client = LightconePinocchioClient::new("https://api.devnet.solana.com");

// With custom program ID
let client = LightconePinocchioClient::with_program_id(rpc_url, program_id);

// From existing RpcClient
let client = LightconePinocchioClient::from_rpc_client(rpc_client);
```

### Account Fetchers

```rust
// Exchange (singleton)
let exchange = client.get_exchange().await?;

// Market by ID
let market = client.get_market(0).await?;

// Market by pubkey
let market = client.get_market_by_pubkey(&market_pda).await?;

// Position (returns None if not found)
let position = client.get_position(&owner, &market_pda).await?;

// Order status (returns None if not found)
let status = client.get_order_status(&order_hash).await?;

// User nonce (returns 0 if account doesn't exist)
let nonce = client.get_user_nonce(&user).await?;

// Next available nonce for new orders
let next_nonce = client.get_next_nonce(&user).await?;

// Next market ID
let next_id = client.get_next_market_id().await?;
```

### Transaction Builders

All builders return `SdkResult<Transaction>`.

**Exchange Management:**
```rust
let tx = client.initialize(&authority).await?;
let tx = client.set_paused(&authority, true).await?;
let tx = client.set_operator(&authority, &new_operator).await?;
```

**Market Lifecycle:**
```rust
// market_id is automatically fetched via get_next_market_id()
let tx = client.create_market(CreateMarketParams {
    authority,
    oracle,
    question_id,
    num_outcomes,
}).await?;

let tx = client.add_deposit_mint(AddDepositMintParams {
    payer,
    market_id,
    deposit_mint: usdc_mint,
    outcome_metadata: vec![
        OutcomeMetadata { name: "Yes".into(), symbol: "YES".into(), uri: "".into() },
        OutcomeMetadata { name: "No".into(), symbol: "NO".into(), uri: "".into() },
    ],
}, &market, num_outcomes).await?;

let tx = client.activate_market(ActivateMarketParams {
    authority,
    market_id,
}).await?;

let tx = client.settle_market(SettleMarketParams {
    oracle,
    market_id,
    winning_outcome,
}).await?;
```

**Position Operations:**
```rust
let tx = client.mint_complete_set(MintCompleteSetParams {
    user,
    market,
    deposit_mint: usdc_mint,
    amount,
}, num_outcomes).await?;

let tx = client.merge_complete_set(MergeCompleteSetParams {
    user,
    market,
    deposit_mint: usdc_mint,
    amount,
}, num_outcomes).await?;

let tx = client.redeem_winnings(RedeemWinningsParams {
    user,
    market,
    deposit_mint: usdc_mint,
    amount,
}, winning_outcome).await?;

let tx = client.withdraw_from_position(WithdrawFromPositionParams {
    user,
    market,
    mint: conditional_mint,
    amount,
}, is_token_2022).await?;
```

**Order Operations:**
```rust
let tx = client.cancel_order(&maker, &order).await?;
let tx = client.increment_nonce(&user).await?;

// With individual Ed25519 verification
let tx = client.match_orders_multi(MatchOrdersMultiParams {
    operator,
    market,
    base_mint: yes_token,
    quote_mint: no_token,
    taker_order: taker,
    maker_orders: makers,
    fill_amounts: fills,
}).await?;

// With batch Ed25519 verification
let tx = client.match_orders_multi_with_verify(params).await?;

// With cross-instruction Ed25519 references (most efficient)
let tx = client.match_orders_multi_cross_ref(params).await?;
```

### Order Helpers

```rust
// Create unsigned orders
let order = client.create_bid_order(BidOrderParams {
    nonce: 1,
    maker: pubkey,
    market: market_pda,
    base_mint: yes_token,
    quote_mint: no_token,
    amount_in: 100_000,
    amount_out: 50_000,
    expiration: 0,
});

let order = client.create_ask_order(AskOrderParams { ... });

// Create and sign in one step
let order = client.create_signed_bid_order(params, &keypair);
let order = client.create_signed_ask_order(params, &keypair);

// Manual signing
client.sign_order(&mut order, &keypair);

// Get order hash
let hash = client.hash_order(&order);
```

## PDA Derivation

All functions return `(Pubkey, u8)` (address, bump).

| Function | Seeds | Description |
|----------|-------|-------------|
| `get_exchange_pda(program_id)` | `["central_state"]` | Exchange singleton |
| `get_market_pda(market_id, program_id)` | `["market", market_id.to_le_bytes()]` | Market account |
| `get_vault_pda(deposit_mint, market, program_id)` | `["market_deposit_token_account", deposit_mint, market]` | Deposit vault |
| `get_mint_authority_pda(market, program_id)` | `["market_mint_authority", market]` | Conditional token mint authority |
| `get_conditional_mint_pda(market, deposit_mint, outcome, program_id)` | `["conditional_mint", market, deposit_mint, [outcome]]` | Conditional token mint |
| `get_order_status_pda(order_hash, program_id)` | `["order_status", order_hash]` | Order fill status |
| `get_user_nonce_pda(user, program_id)` | `["user_nonce", user]` | User nonce account |
| `get_position_pda(owner, market, program_id)` | `["position", owner, market]` | User position |

```rust
use lightcone_sdk::program::*;

let (exchange, _) = get_exchange_pda(&program_id);
let (market, _) = get_market_pda(market_id, &program_id);
let (position, _) = get_position_pda(&owner, &market, &program_id);
let (vault, _) = get_vault_pda(&deposit_mint, &market, &program_id);
let (cond_mint, _) = get_conditional_mint_pda(&market, &deposit_mint, 0, &program_id);
```

## Order Types

### FullOrder (225 bytes)

Complete order with signature for off-chain storage and API submission.

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| nonce | 0 | 8 | `u64` | Unique order identifier (must exceed user's on-chain nonce) |
| maker | 8 | 32 | `Pubkey` | Order creator |
| market | 40 | 32 | `Pubkey` | Market pubkey |
| base_mint | 72 | 32 | `Pubkey` | Token being bought (bids) or sold (asks) |
| quote_mint | 104 | 32 | `Pubkey` | Token being given (bids) or received (asks) |
| side | 136 | 1 | `OrderSide` | Bid=0, Ask=1 |
| amount_in | 137 | 8 | `u64` | Amount maker gives (quote for bids, base for asks) |
| amount_out | 145 | 8 | `u64` | Amount maker receives (base for bids, quote for asks) |
| expiration | 153 | 8 | `i64` | Unix timestamp (0 = no expiration) |
| signature | 161 | 64 | `[u8; 64]` | Ed25519 signature |

**Methods:**
```rust
// Construction
FullOrder::new_bid(params) -> FullOrder
FullOrder::new_ask(params) -> FullOrder
FullOrder::new_bid_signed(params, &keypair) -> FullOrder
FullOrder::new_ask_signed(params, &keypair) -> FullOrder

// Hashing and signing
order.hash() -> [u8; 32]           // Keccak256 of fields (excludes signature)
order.sign(&keypair)               // Sign in place
order.verify_signature() -> Result<()>

// Serialization
order.serialize() -> [u8; 225]
FullOrder::deserialize(data) -> Result<FullOrder>

// Conversion
order.to_compact() -> CompactOrder
```

### CompactOrder (65 bytes)

On-chain transmission format. Excludes market/base_mint/quote_mint (passed via accounts).

| Field | Offset | Size | Type | Description |
|-------|--------|------|------|-------------|
| nonce | 0 | 8 | `u64` | Order nonce |
| maker | 8 | 32 | `Pubkey` | Order creator |
| side | 40 | 1 | `OrderSide` | Bid=0, Ask=1 |
| amount_in | 41 | 8 | `u64` | Amount maker gives |
| amount_out | 49 | 8 | `u64` | Amount maker receives |
| expiration | 57 | 8 | `i64` | Expiration timestamp |

```rust
let compact = order.to_compact();
let full = compact.to_full_order(&market, &base_mint, &quote_mint, &signature);
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
let mints = client.get_conditional_mints(&market, &deposit_mint, num_outcomes);
```

## Ed25519 Signature Verification

Orders are signed off-chain with Ed25519 and verified on-chain. Three strategies available:

### Strategy 1: Individual Instructions (Simplest)

Each order gets its own Ed25519 instruction. 144 bytes per instruction.

```rust
let ix = create_ed25519_verify_instruction(&Ed25519VerifyParams::from_order(&order));
```

### Strategy 2: Batch Verification (More Efficient)

Single Ed25519 instruction verifies multiple signatures.

```rust
let ix = create_batch_ed25519_verify_instruction(&[
    Ed25519VerifyParams::from_order(&order1),
    Ed25519VerifyParams::from_order(&order2),
]);
```

Size: `2 + (num_signatures * 14) + (num_signatures * 128)` bytes

### Strategy 3: Cross-Instruction References (Most Efficient)

Ed25519 instructions reference data embedded in the match instruction. Saves ~128 bytes per order.

```rust
let ixs = create_cross_ref_ed25519_instructions(num_makers, match_ix_index);
```

The match instruction embeds order data at known offsets:
- Taker hash at offset 1
- Taker pubkey at offset 41
- Taker signature at offset 98
- Each maker: hash, pubkey, signature at `162 + (i * 169)`

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

## Constants

### Program IDs

```rust
PROGRAM_ID: Pubkey                    // "EfRvELrn4b5aJRwddD1VUrqzsfm1pewBLPebq3iMPDp2"
TOKEN_PROGRAM_ID: Pubkey              // SPL Token
TOKEN_2022_PROGRAM_ID: Pubkey         // Token-2022 (conditional tokens)
ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey
SYSTEM_PROGRAM_ID: Pubkey
RENT_SYSVAR_ID: Pubkey
INSTRUCTIONS_SYSVAR_ID: Pubkey
ED25519_PROGRAM_ID: Pubkey
```

### Discriminators

```rust
EXCHANGE_DISCRIMINATOR: [u8; 8]       // b"exchange"
MARKET_DISCRIMINATOR: [u8; 8]         // b"market\0\0"
ORDER_STATUS_DISCRIMINATOR: [u8; 8]   // b"ordstat\0"
USER_NONCE_DISCRIMINATOR: [u8; 8]     // b"usrnonce"
POSITION_DISCRIMINATOR: [u8; 8]       // b"position"
```

### Seeds

```rust
EXCHANGE_SEED: &[u8]                  // b"central_state"
MARKET_SEED: &[u8]                    // b"market"
VAULT_SEED: &[u8]                     // b"market_deposit_token_account"
MINT_AUTHORITY_SEED: &[u8]            // b"market_mint_authority"
CONDITIONAL_MINT_SEED: &[u8]          // b"conditional_mint"
ORDER_STATUS_SEED: &[u8]              // b"order_status"
USER_NONCE_SEED: &[u8]                // b"user_nonce"
POSITION_SEED: &[u8]                  // b"position"
```

### Sizes

```rust
EXCHANGE_SIZE: usize                  // 88
MARKET_SIZE: usize                    // 120
POSITION_SIZE: usize                  // 80
ORDER_STATUS_SIZE: usize              // 24
USER_NONCE_SIZE: usize                // 16
FULL_ORDER_SIZE: usize                // 225
COMPACT_ORDER_SIZE: usize             // 65
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
    pub authority: Pubkey,
    pub num_outcomes: u8,
    pub oracle: Pubkey,
    pub question_id: [u8; 32],
}

pub struct AddDepositMintParams {
    pub payer: Pubkey,
    pub market_id: u64,
    pub deposit_mint: Pubkey,
    pub outcome_metadata: Vec<OutcomeMetadata>,
}

pub struct OutcomeMetadata {
    pub name: String,    // max 32 chars
    pub symbol: String,  // max 10 chars
    pub uri: String,     // max 200 chars
}

pub struct ActivateMarketParams {
    pub authority: Pubkey,
    pub market_id: u64,
}

pub struct SettleMarketParams {
    pub oracle: Pubkey,
    pub market_id: u64,
    pub winning_outcome: u8,
}

pub struct MintCompleteSetParams {
    pub user: Pubkey,
    pub market: Pubkey,
    pub deposit_mint: Pubkey,
    pub amount: u64,
}

pub struct MergeCompleteSetParams {
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
}

pub struct MatchOrdersMultiParams {
    pub operator: Pubkey,
    pub market: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub taker_order: FullOrder,
    pub maker_orders: Vec<FullOrder>,
    pub fill_amounts: Vec<u64>,
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
    Rpc(solana_client::client_error::ClientError),
    InvalidDiscriminator { expected: String, actual: String },
    AccountNotFound(String),
    InvalidDataLength { expected: usize, actual: usize },
    InvalidOrderHash,
    InvalidSignature,
    OrderExpired,
    InvalidOutcomeCount { count: u8 },
    InvalidOutcomeIndex { index: u8, max: u8 },
    TooManyMakers { count: usize },
    Serialization(String),
    InvalidSide(u8),
    InvalidMarketStatus(u8),
    SignatureVerificationFailed,
    MissingField(String),
    OrdersDoNotCross,
    FillAmountExceedsRemaining { fill: u64, remaining: u64 },
    Overflow,
    InvalidPubkey(String),
}

pub type SdkResult<T> = Result<T, SdkError>;
```
