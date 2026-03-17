# Admin Operations

Internal admin operations for the Lightcone team. These endpoints require ED25519 signature authorization from a key registered in the backend.

[← Overview](../../../README.md)

## Table of Contents

- [AdminEnvelope](#adminenvelope)
- [Client Methods](#client-methods)
- [Wire Types](#wire-types)
- [TargetSpec](#targetspec)

## AdminEnvelope

All admin requests are wrapped in an `AdminEnvelope<T>` that contains the payload and an ED25519 signature over the canonical JSON representation of the payload.

```rust
use lightcone::domain::admin::{AdminEnvelope, UnifiedMetadataRequest};

let envelope = AdminEnvelope {
    payload: UnifiedMetadataRequest { /* ... */ },
    signature: "base58_encoded_ed25519_signature".to_string(),
};
```

The caller is responsible for:
1. Serializing the payload to canonical JSON (`serde_json::to_string`)
2. Signing the JSON bytes with an authorized ED25519 keypair
3. Base58-encoding the signature

## Client Methods

Access via `client.admin()`.

### `upsert_metadata`

```rust
async fn upsert_metadata(
    &self,
    envelope: &AdminEnvelope<UnifiedMetadataRequest>,
) -> Result<UnifiedMetadataResponse, SdkError>
```

Upsert metadata for markets, outcomes, conditional tokens, and deposit tokens in a single batch operation.

### `allocate_codes`

```rust
async fn allocate_codes(
    &self,
    envelope: &AdminEnvelope<AllocateCodesRequest>,
) -> Result<AllocateCodesResponse, SdkError>
```

Allocate referral codes to users. Can target all users, a specific user, or use vanity codes.

### `whitelist`

```rust
async fn whitelist(
    &self,
    envelope: &AdminEnvelope<WhitelistRequest>,
) -> Result<WhitelistResponse, SdkError>
```

Whitelist wallet addresses for beta access, optionally allocating referral codes.

### `revoke`

```rust
async fn revoke(
    &self,
    envelope: &AdminEnvelope<RevokeRequest>,
) -> Result<RevokeResponse, SdkError>
```

Revoke a user's beta access and/or referral codes.

### `unrevoke`

```rust
async fn unrevoke(
    &self,
    envelope: &AdminEnvelope<UnrevokeRequest>,
) -> Result<UnrevokeResponse, SdkError>
```

Restore a previously revoked user's access.

### `create_notification`

```rust
async fn create_notification(
    &self,
    envelope: &AdminEnvelope<CreateNotificationRequest>,
) -> Result<CreateNotificationResponse, SdkError>
```

Create a notification for users.

### `dismiss_notification`

```rust
async fn dismiss_notification(
    &self,
    envelope: &AdminEnvelope<DismissNotificationRequest>,
) -> Result<DismissNotificationResponse, SdkError>
```

Dismiss a notification.

### On-Chain Transaction Builders

All return `Result<Transaction, SdkError>`.

#### `initialize_ix`

```rust
fn initialize_ix(&self, authority: &Pubkey) -> Result<Transaction, SdkError>
```

Build an Initialize transaction — create the exchange singleton account.

#### `create_market_ix`

```rust
async fn create_market_ix(&self, params: CreateMarketParams) -> Result<Transaction, SdkError>
```

Build a CreateMarket transaction. **Async** — fetches the next market ID from on-chain state via RPC. Requires `solana-rpc` feature.

#### `add_deposit_mint_ix`

```rust
fn add_deposit_mint_ix(
    &self,
    params: AddDepositMintParams,
    market: &Pubkey,
    num_outcomes: u8,
) -> Result<Transaction, SdkError>
```

Build an AddDepositMint transaction — add a deposit token (e.g., USDC) to a market and create conditional token mints.

#### `activate_market_ix`

```rust
fn activate_market_ix(&self, params: ActivateMarketParams) -> Result<Transaction, SdkError>
```

Build an ActivateMarket transaction — transition a market from Pending to Active.

#### `settle_market_ix`

```rust
fn settle_market_ix(&self, params: SettleMarketParams) -> Result<Transaction, SdkError>
```

Build a SettleMarket transaction — resolve a market with the winning outcome.

#### `set_paused_ix`

```rust
fn set_paused_ix(&self, authority: &Pubkey, paused: bool) -> Result<Transaction, SdkError>
```

Build a SetPaused transaction — pause or unpause the exchange.

#### `set_operator_ix`

```rust
fn set_operator_ix(&self, authority: &Pubkey, new_operator: &Pubkey) -> Result<Transaction, SdkError>
```

Build a SetOperator transaction — change the exchange operator.

#### `set_authority_ix`

```rust
fn set_authority_ix(&self, params: SetAuthorityParams) -> Result<Transaction, SdkError>
```

Build a SetAuthority transaction — transfer exchange authority to a new key.

#### `whitelist_deposit_token_ix`

```rust
fn whitelist_deposit_token_ix(&self, params: WhitelistDepositTokenParams) -> Result<Transaction, SdkError>
```

Build a WhitelistDepositToken transaction — whitelist a deposit token for the exchange.

#### `create_orderbook_ix`

```rust
fn create_orderbook_ix(&self, params: CreateOrderbookParams) -> Result<Transaction, SdkError>
```

Build a CreateOrderbook transaction — create an orderbook for a token pair.

#### `match_orders_multi_ix`

```rust
fn match_orders_multi_ix(&self, params: MatchOrdersMultiParams) -> Result<Transaction, SdkError>
```

Build a MatchOrdersMulti transaction — match a taker order against one or more maker orders.

#### `deposit_and_swap_ix`

```rust
fn deposit_and_swap_ix(&self, params: DepositAndSwapParams) -> Result<Transaction, SdkError>
```

Build a DepositAndSwap transaction — deposit collateral and atomically swap into a conditional token position.

## Wire Types

### `UnifiedMetadataRequest`

Batch metadata upsert payload. All arrays are optional -- only include the entities you want to update.

| Field | Type | Description |
|-------|------|-------------|
| `markets` | `Vec<MarketMetadataPayload>` | Market metadata updates |
| `outcomes` | `Vec<OutcomeMetadataPayload>` | Outcome metadata updates |
| `conditional_tokens` | `Vec<ConditionalTokenMetadataPayload>` | Token metadata updates |
| `deposit_tokens` | `Vec<DepositTokenMetadataPayload>` | Deposit token metadata updates |

Each payload struct uses `Option<T>` fields -- only non-`None` fields are updated, leaving other fields unchanged.

### `AllocateCodesRequest`

| Field | Type | Description |
|-------|------|-------------|
| `target` | `TargetSpec` | Who to allocate codes to |
| `batch_id` | `Option<String>` | Optional batch identifier |
| `vanity_codes` | `Option<Vec<String>>` | Specific codes to create |
| `count` | `Option<u32>` | Number of codes to generate (if not vanity) |
| `max_uses` | `Option<i32>` | Maximum redemptions per code |

### `WhitelistRequest`

| Field | Type | Description |
|-------|------|-------------|
| `wallet_addresses` | `Vec<String>` | Wallet addresses to whitelist |
| `allocate_codes` | `Option<bool>` | Whether to also allocate referral codes |

### `RevokeRequest` / `UnrevokeRequest`

| Field | Type | Description |
|-------|------|-------------|
| `target` | `TargetSpec` | Who to revoke/unrevoke |
| `reason` | `Option<String>` | Reason for revocation (revoke only) |

## TargetSpec

`TargetSpec` identifies the target of admin referral operations.

| Constructor | Serializes as | Description |
|-------------|---------------|-------------|
| `TargetSpec::all()` | `"all"` | All users |
| `TargetSpec::user_id(id)` | `{"user_id": "..."}` | Specific user by ID |
| `TargetSpec::wallet_address(addr)` | `{"wallet_address": "..."}` | Specific user by wallet |
| `TargetSpec::code(code)` | `{"code": "..."}` | Specific referral code |
| `TargetSpec::batch_id(id)` | `{"batch_id": "..."}` | All codes in a batch |

```rust
use lightcone::domain::admin::TargetSpec;

let target = TargetSpec::wallet_address("7BgBvyjr...");
let all = TargetSpec::all();
```

---

[← Overview](../../../README.md)
