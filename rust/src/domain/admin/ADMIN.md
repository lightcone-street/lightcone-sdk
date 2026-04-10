# Admin Operations

Internal admin operations for the Lightcone team. These endpoints require cookie-based authentication obtained via the admin login flow.

[<- Overview](../../../README.md)

## Table of Contents

- [Authentication](#authentication)
- [Client Methods](#client-methods)
- [Wire Types](#wire-types)
- [TargetSpec](#targetspec)

## Authentication

Admin endpoints use cookie-based auth (same pattern as user auth). Before calling any admin method, you must complete the login flow:

1. Call `get_admin_nonce()` to get a nonce and message to sign.
2. Sign the message with an ED25519 keypair authorized in the backend.
3. Call `admin_login()` with the signed message — the backend sets an `admin_token` HttpOnly cookie, which the SDK captures automatically on native and the browser handles on WASM.
4. All subsequent admin methods automatically attach the cookie.

```rust
use lightcone::LightconeClient;

let client = LightconeClient::new("https://api.example.com");
let admin = client.admin();

// Step 1: Get the nonce and message
let nonce_response = admin.get_admin_nonce().await?;

// Step 2: Sign the message with your ED25519 keypair (application-specific)
let signature_bs58 = sign_message(&nonce_response.message, &keypair);

// Step 3: Login — admin cookie is captured automatically for future requests
let login_response = admin.admin_login(
    &nonce_response.message,
    &signature_bs58,
    &keypair.pubkey().to_bytes(),
).await?;

// Step 4: Admin methods now work
let response = admin.upsert_metadata(&metadata_request).await?;
```

## Client Methods

Access via `client.admin()`.

### `get_admin_nonce`

```rust
async fn get_admin_nonce(&self) -> Result<AdminNonceResponse, SdkError>
```

Fetch the nonce and message to sign for admin login.

### `admin_login`

```rust
async fn admin_login(
    &self,
    message: &str,
    signature_bs58: &str,
    pubkey_bytes: &[u8],
) -> Result<AdminLoginResponse, SdkError>
```

Verify the signature and establish an admin session. The backend sets an `admin_token` HttpOnly cookie. Returns the wallet address and expiration timestamp.

### `admin_logout`

```rust
async fn admin_logout(&self) -> Result<(), SdkError>
```

Log out the admin session — attempts to clear the server-side cookie and always clears the internal token.

### `upsert_metadata`

```rust
async fn upsert_metadata(
    &self,
    request: &UnifiedMetadataRequest,
) -> Result<UnifiedMetadataResponse, SdkError>
```

Upsert metadata for markets, outcomes, conditional tokens, and deposit tokens in a single batch operation. Requires prior `admin_login()`.

### `allocate_codes`

```rust
async fn allocate_codes(
    &self,
    request: &AllocateCodesRequest,
) -> Result<AllocateCodesResponse, SdkError>
```

Allocate referral codes to users. Can target all users, a specific user, or use vanity codes. Requires prior `admin_login()`.

### `whitelist`

```rust
async fn whitelist(
    &self,
    request: &WhitelistRequest,
) -> Result<WhitelistResponse, SdkError>
```

Whitelist wallet addresses for beta access, optionally allocating referral codes. Requires prior `admin_login()`.

### `revoke`

```rust
async fn revoke(
    &self,
    request: &RevokeRequest,
) -> Result<RevokeResponse, SdkError>
```

Revoke a user's beta access and/or referral codes. Requires prior `admin_login()`.

### `unrevoke`

```rust
async fn unrevoke(
    &self,
    request: &UnrevokeRequest,
) -> Result<UnrevokeResponse, SdkError>
```

Restore a previously revoked user's access. Requires prior `admin_login()`.

### `create_notification`

```rust
async fn create_notification(
    &self,
    request: &CreateNotificationRequest,
) -> Result<CreateNotificationResponse, SdkError>
```

Create a notification for users. Requires prior `admin_login()`.

### `dismiss_notification`

```rust
async fn dismiss_notification(
    &self,
    request: &DismissNotificationRequest,
) -> Result<DismissNotificationResponse, SdkError>
```

Dismiss a notification. Requires prior `admin_login()`.

### On-Chain Instruction & Transaction Builders

Each operation has an `_ix` method returning an `Instruction` (or `Result<Instruction, SdkError>` for fallible builders) and a `_tx` convenience method returning `Result<Transaction, SdkError>`.

#### `initialize_ix` / `initialize_tx`

```rust
fn initialize_ix(&self, authority: &Pubkey) -> Instruction
fn initialize_tx(&self, authority: &Pubkey) -> Result<Transaction, SdkError>
```

Build an Initialize instruction/transaction — create the exchange singleton account.

#### `create_market_ix` / `create_market_tx`

```rust
async fn create_market_ix(&self, params: CreateMarketParams) -> Result<Instruction, SdkError>
async fn create_market_tx(&self, params: CreateMarketParams) -> Result<Transaction, SdkError>
```

Build a CreateMarket instruction/transaction. **Async** — fetches the next market ID from on-chain state via RPC. Requires `solana-rpc` feature.

#### `add_deposit_mint_ix` / `add_deposit_mint_tx`

```rust
fn add_deposit_mint_ix(
    &self,
    params: &AddDepositMintParams,
    market: &Pubkey,
    num_outcomes: u8,
) -> Result<Instruction, SdkError>

fn add_deposit_mint_tx(
    &self,
    params: AddDepositMintParams,
    market: &Pubkey,
    num_outcomes: u8,
) -> Result<Transaction, SdkError>
```

Build an AddDepositMint instruction/transaction — add a deposit token (e.g., USDC) to a market and create conditional token mints.

#### `activate_market_ix` / `activate_market_tx`

```rust
fn activate_market_ix(&self, params: &ActivateMarketParams) -> Instruction
fn activate_market_tx(&self, params: ActivateMarketParams) -> Result<Transaction, SdkError>
```

Build an ActivateMarket instruction/transaction — transition a market from Pending to Active.

#### `settle_market_ix` / `settle_market_tx`

```rust
fn settle_market_ix(&self, params: &SettleMarketParams) -> Instruction
fn settle_market_tx(&self, params: SettleMarketParams) -> Result<Transaction, SdkError>
```

Build a SettleMarket instruction/transaction — resolve a market with the winning outcome.

#### `set_paused_ix` / `set_paused_tx`

```rust
fn set_paused_ix(&self, authority: &Pubkey, paused: bool) -> Instruction
fn set_paused_tx(&self, authority: &Pubkey, paused: bool) -> Result<Transaction, SdkError>
```

Build a SetPaused instruction/transaction — pause or unpause the exchange.

#### `set_operator_ix` / `set_operator_tx`

```rust
fn set_operator_ix(&self, authority: &Pubkey, new_operator: &Pubkey) -> Instruction
fn set_operator_tx(&self, authority: &Pubkey, new_operator: &Pubkey) -> Result<Transaction, SdkError>
```

Build a SetOperator instruction/transaction — change the exchange operator.

#### `set_authority_ix` / `set_authority_tx`

```rust
fn set_authority_ix(&self, params: &SetAuthorityParams) -> Instruction
fn set_authority_tx(&self, params: SetAuthorityParams) -> Result<Transaction, SdkError>
```

Build a SetAuthority instruction/transaction — transfer exchange authority to a new key.

#### `whitelist_deposit_token_ix` / `whitelist_deposit_token_tx`

```rust
fn whitelist_deposit_token_ix(&self, params: &WhitelistDepositTokenParams) -> Instruction
fn whitelist_deposit_token_tx(&self, params: WhitelistDepositTokenParams) -> Result<Transaction, SdkError>
```

Build a WhitelistDepositToken instruction/transaction — whitelist a deposit token for the exchange.

#### `create_orderbook_ix` / `create_orderbook_tx`

```rust
fn create_orderbook_ix(&self, params: &CreateOrderbookParams) -> Instruction
fn create_orderbook_tx(&self, params: CreateOrderbookParams) -> Result<Transaction, SdkError>
```

Build a CreateOrderbook instruction/transaction — create an orderbook for a token pair.

#### `match_orders_multi_ix` / `match_orders_multi_tx`

```rust
fn match_orders_multi_ix(&self, params: &MatchOrdersMultiParams) -> Result<Instruction, SdkError>
fn match_orders_multi_tx(&self, params: MatchOrdersMultiParams) -> Result<Transaction, SdkError>
```

Build a MatchOrdersMulti instruction/transaction — match a taker order against one or more maker orders.

#### `deposit_and_swap_ix` / `deposit_and_swap_tx`

```rust
fn deposit_and_swap_ix(&self, params: &DepositAndSwapParams) -> Result<Instruction, SdkError>
fn deposit_and_swap_tx(&self, params: DepositAndSwapParams) -> Result<Transaction, SdkError>
```

Build a DepositAndSwap instruction/transaction — deposit collateral and atomically swap into a conditional token position.

## Wire Types

### `AdminNonceResponse`

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | `String` | Server-generated nonce |
| `message` | `String` | Message to sign with ED25519 keypair |

### `AdminLoginRequest`

| Field | Type | Description |
|-------|------|-------------|
| `message` | `String` | The message that was signed |
| `signature_bs58` | `String` | Base58-encoded ED25519 signature |
| `pubkey_bytes` | `Vec<u8>` | Public key bytes of the signing keypair |

### `AdminLoginResponse`

| Field | Type | Description |
|-------|------|-------------|
| `wallet_address` | `String` | Wallet address of the authenticated admin |
| `expires_at` | `i64` | Session expiration timestamp |

### `UnifiedMetadataRequest`

Batch metadata upsert payload. All arrays are optional — only include the entities you want to update.

| Field | Type | Description |
|-------|------|-------------|
| `markets` | `Vec<MarketMetadataPayload>` | Market metadata updates |
| `outcomes` | `Vec<OutcomeMetadataPayload>` | Outcome metadata updates |
| `conditional_tokens` | `Vec<ConditionalTokenMetadataPayload>` | Token metadata updates |
| `deposit_tokens` | `Vec<DepositTokenMetadataPayload>` | Deposit token metadata updates |

Each payload struct uses `Option<T>` fields — only non-`None` fields are updated, leaving other fields unchanged.

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

[<- Overview](../../../README.md)
