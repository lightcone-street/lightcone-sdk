# Admin Operations

Internal admin operations for the Lightcone team. These endpoints require cookie-based authentication obtained via the admin login flow.

[<- Overview](../../../README.md)

## Table of Contents

- [Authentication](#authentication)
- [Client Methods](#client-methods)
- [Metadata Categories](#metadata-categories)
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

Market `category` values are validated by the backend. For new market metadata rows, `category` is required and must already exist in the category whitelist. For existing market metadata rows, `category` may be omitted in partial updates and the existing database value is preserved. When a category is provided, the backend matches it against the whitelist case-insensitively and stores the canonical whitelist casing.

Potential category validation error codes include `MARKET_CATEGORY_REQUIRED`, `MARKET_CATEGORY_INVALID`, `MARKET_CATEGORY_NOT_WHITELISTED`, and `MARKET_CATEGORY_LOOKUP_FAILED`.

### `get_market_metadata`

```rust
async fn get_market_metadata(
    &self,
    market_id: i64,
) -> Result<AdminMarketMetadataResponse, SdkError>
```

Fetch full admin metadata for one market, including the canonical market row, optional market metadata row, deposit assets, outcomes, conditional token rows, and missing metadata indicators. Requires prior `admin_login()`.

### `update_market_metadata`

```rust
async fn update_market_metadata(
    &self,
    market_id: i64,
    request: &UpdateMarketMetadataRequest,
) -> Result<UpdateMarketMetadataResponse, SdkError>
```

Update database metadata for one market, its outcomes, and conditional tokens. This does not upload image bytes. At least one of `market`, `outcomes`, or `conditional_tokens` must be non-empty. Requires prior `admin_login()`.

```rust
let request = UpdateMarketMetadataRequest {
    market: Some(UpdateMarketMetadataPayload {
        market_name: Some("Will BTC close above $100k?".into()),
        category: Some("Crypto".into()),
        ..Default::default()
    }),
    ..Default::default()
};

let response = client.admin().update_market_metadata(42, &request).await?;
```

### `update_market_images`

```rust
async fn update_market_images(
    &self,
    market_id: i64,
    request: &UpdateMarketImagesRequest,
) -> Result<MetadataImageUpdateResponse, SdkError>
```

Replace existing market, outcome, and conditional token image bytes at the metadata URLs already stored in the database. URL columns are not changed. Image variants must be WebP data URLs. Requires prior `admin_login()`.

### `get_deposit_token_metadata`

```rust
async fn get_deposit_token_metadata(
    &self,
    deposit_asset: &str,
) -> Result<AdminDepositTokenMetadataResponse, SdkError>
```

Fetch database metadata for one deposit token. Requires prior `admin_login()`.

### `update_deposit_token_metadata`

```rust
async fn update_deposit_token_metadata(
    &self,
    deposit_asset: &str,
    request: &UpdateDepositTokenMetadataRequest,
) -> Result<UpdateDepositTokenMetadataResponse, SdkError>
```

Update database metadata for one deposit token. This does not upload image bytes. Requires prior `admin_login()`.

### `update_deposit_token_images`

```rust
async fn update_deposit_token_images(
    &self,
    deposit_asset: &str,
    request: &UpdateDepositTokenImagesRequest,
) -> Result<MetadataImageUpdateResponse, SdkError>
```

Replace existing deposit token icon bytes at the metadata URLs already stored in the database. URL columns are not changed. Image variants must be WebP data URLs. Requires prior `admin_login()`.

### `list_metadata_categories`

```rust
async fn list_metadata_categories(&self) -> Result<MetadataCategoriesResponse, SdkError>
```

List all whitelisted market metadata categories. Requires prior `admin_login()`.

### `add_metadata_category`

```rust
async fn add_metadata_category(
    &self,
    request: &AddMetadataCategoryRequest,
) -> Result<AddMetadataCategoryResponse, SdkError>
```

Add a category to the whitelist. Requires prior `admin_login()`. The operation is idempotent and case-insensitive: if `Politics` already exists, posting `politics` returns the canonical `Politics`.

### `upload_market_deployment_assets`

```rust
async fn upload_market_deployment_assets(
    &self,
    request: &UploadMarketDeploymentAssetsRequest,
) -> Result<UploadMarketDeploymentAssetsResponse, SdkError>
```

Upload banner/icon/outcome/conditional-token images and metadata for a newly created market. Uploads are quality-specific WebP data URLs (`low`, `medium`, `high`) plus matching `image/webp` content-type fields; the backend stores them and returns the resulting URLs. Requires prior `admin_login()`.

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

### `markets`

```rust
async fn markets(
    &self,
    query: &AdminMarketsQuery,
) -> Result<AdminMarketsResponse, SdkError>
```

List the main admin markets table from cached metrics. Supports offset cursor pagination, sorting, status/category/search filters, and numeric range filters. Requires prior `admin_login()`.

Defaults are applied by the backend when omitted: `cursor=0`, `limit=200`, `sort_by=volume_24h_usd`, `sort_direction=desc`, and `market_status=all`. The `market_status` filter supports only `all`, `active`, and `resolved`; do not send `settled`.

### `markets_to_settle_count`

```rust
async fn markets_to_settle_count(&self) -> Result<MarketsToSettleCountResponse, SdkError>
```

Count active, unsettled markets whose metadata `resolution_by` is set and in the past. Requires prior `admin_login()`.

### `markets_to_settle`

```rust
async fn markets_to_settle(
    &self,
    query: &MarketsToSettleQuery,
) -> Result<MarketsToSettleResponse, SdkError>
```

List active markets ready to settle using the existing market response shape. Pagination uses `market_id` as the cursor: pass the previous response's `next_cursor` to continue. The backend default limit is `200` and max limit is `1000`. Requires prior `admin_login()`.

### `get_referral_config`

```rust
async fn get_referral_config(&self) -> Result<ReferralConfig, SdkError>
```

Fetch the platform-wide referral configuration (`default_code_count` + `updated_at`). Requires prior `admin_login()`.

### `update_referral_config`

```rust
async fn update_referral_config(
    &self,
    request: &UpdateConfigRequest,
) -> Result<ReferralConfig, SdkError>
```

Update the platform-wide referral configuration. Passing `default_code_count: None` is accepted as a no-op. Requires prior `admin_login()`.

### `list_referral_codes`

```rust
async fn list_referral_codes(
    &self,
    request: &ListCodesRequest,
) -> Result<ListCodesResponse, SdkError>
```

List referral codes with optional owner/batch/code filters and offset+limit pagination. Requires prior `admin_login()`.

### `update_referral_code`

```rust
async fn update_referral_code(
    &self,
    request: &UpdateCodeRequest,
) -> Result<UpdateCodeResponse, SdkError>
```

Update the `max_uses` on an existing referral code. Requires prior `admin_login()`.

### `list_log_events`

```rust
async fn list_log_events(
    &self,
    query: &AdminLogEventsQuery,
) -> Result<AdminLogEventsResponse, SdkError>
```

List structured log events with optional filters (time range, service, component, severity, error code, rejection code, user/market/orderbook, etc.). Pagination is cursor-based — pass the previous response's `next_cursor` on the next call. Requires prior `admin_login()`.

### `get_log_event`

```rust
async fn get_log_event(&self, public_id: &str) -> Result<AdminLogEvent, SdkError>
```

Fetch a single log event by its `public_id`. Requires prior `admin_login()`.

### `log_metrics`

```rust
async fn log_metrics(
    &self,
    query: &AdminLogMetricsQuery,
) -> Result<AdminLogMetricsResponse, SdkError>
```

Fetch rolled-up log metrics broken down by window and scope. `windows` and `scopes` are CSV lists (`"1h,24h"` or `"service,component"`). Requires prior `admin_login()`.

### `log_metric_history`

```rust
async fn log_metric_history(
    &self,
    query: &AdminLogMetricHistoryQuery,
) -> Result<AdminLogMetricHistoryResponse, SdkError>
```

Fetch a time-series of log metric buckets for a given scope (optionally narrowed to a `scope_key`). Use `AdminLogMetricHistoryQuery::new("service")` for default `"1h"` resolution. Requires prior `admin_login()`.

### `critical_log_errors_24h_count`

```rust
async fn critical_log_errors_24h_count(&self) -> Result<CriticalLogErrors24hCountResponse, SdkError>
```

Count critical log errors from the previous 24 hours. Requires prior `admin_login()`. If the backend logging service is unavailable or not configured, this returns the API error response as `SdkError`.

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
fn settle_market_ix(&self, params: &SettleMarketParams) -> Result<Instruction, SdkError>
fn settle_market_tx(&self, params: SettleMarketParams) -> Result<Transaction, SdkError>
```

Build a SettleMarket instruction/transaction — resolve a market with payout numerators.

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

#### `set_manager_ix` / `set_manager_tx`

```rust
fn set_manager_ix(&self, params: &SetManagerParams) -> Instruction
fn set_manager_tx(&self, params: SetManagerParams) -> Result<Transaction, SdkError>
```

Build a SetManager instruction/transaction — rotate the exchange manager role.

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

## Metadata Categories

Category whitelist endpoints live under `/api/admin/metadata/categories` and use the same admin cookie auth as the rest of the admin API. They do not require a per-call signature.

`GET /api/admin/metadata/categories` returns the current whitelist:

```json
{
  "categories": ["Politics", "Crypto", "Sports"]
}
```

`POST /api/admin/metadata/categories` accepts the category directly:

```json
{
  "category": "Crypto"
}
```

The response contains the canonical stored category:

```json
{
  "category": "Crypto"
}
```

Validation is performed by the backend: `category` is required, whitespace is trimmed, empty categories are rejected, the maximum length is 100 characters, and duplicates are matched case-insensitively.

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

Batch metadata upsert payload. All arrays are optional, but at least one section must be non-empty. Only include the entities you want to update.

| Field | Type | Description |
|-------|------|-------------|
| `markets` | `Vec<MarketMetadataPayload>` | Market metadata updates |
| `outcomes` | `Vec<OutcomeMetadataPayload>` | Outcome metadata updates |
| `conditional_tokens` | `Vec<ConditionalTokenMetadataPayload>` | Token metadata updates |
| `deposit_tokens` | `Vec<DepositTokenMetadataPayload>` | Deposit token metadata updates |

Each payload struct uses optional fields for partial updates. Omitted fields are left unchanged. `MarketMetadataPayload::resolution_by` is nested so it can also send an explicit JSON `null`.

### `MarketMetadataPayload`

Market metadata updates are keyed by `market_id`. Optional fields may be sent independently for partial updates.

| Field | Type | Description |
|-------|------|-------------|
| `market_id` | `i64` | Required market ID |
| `market_name`, `slug`, `description`, `definition` | `Option<String>` | Display metadata |
| `banner_image_url_low` / `_medium` / `_high` | `Option<String>` | Banner URLs by quality |
| `icon_url_low` / `_medium` / `_high` | `Option<String>` | Icon URLs by quality |
| `category`, `subcategory` | `Option<String>` | Market category metadata |
| `tags` | `Option<Vec<String>>` | Market tags |
| `featured_rank` | `Option<i16>` | Optional featured ordering |
| `metadata_uri` | `Option<String>` | Optional market metadata URI |
| `resolution_by` | `Option<Option<i64>>` | `None` omits the field and preserves the backend value. `Some(Some(ms))` sends a non-negative Unix timestamp in milliseconds and sets/updates the resolution deadline. `Some(None)` sends JSON `null` and clears the deadline |

Market metadata no longer uses a separate `resolution` boolean. The presence of `resolution_by` determines whether a market has a configured resolution deadline. Metadata responses include `resolution_by` as either a Unix timestamp in milliseconds or `null`.

### `DepositTokenMetadataPayload`

Deposit token metadata is global per `deposit_asset`, not per market. `min_order_size` is raw integer token units, not user-scaled decimal units. For a 6-decimal token such as USDC, `1 USDC` is sent as `1_000_000`.

| Field | Type | Description |
|-------|------|-------------|
| `deposit_asset` | `String` | Required mint/address identifier |
| `display_name` / `symbol` / `token_symbol` | `Option<String>` | Display and token symbols |
| `description` | `Option<String>` | Optional token description |
| `icon_url_low` / `_medium` / `_high` | `Option<String>` | Icon URLs by quality |
| `metadata_uri` | `Option<String>` | Optional token metadata URI |
| `decimals` | `Option<i16>` | Token decimals; backend validates `0..=18` when present |
| `min_order_size` | `Option<i64>` | Raw integer token units. `0` means no minimum. Omitted preserves existing value on update and defaults to `0` on insert |
| `binance_symbol` | `Option<String>` | Uppercase ASCII alphanumeric symbol such as `BTCUSDT` |
| `binance_enabled` | `Option<bool>` | Enables Binance integration. If `true`, a Binance symbol must exist in this request or already in the DB |
| `okx_inst_id` | `Option<String>` | Uppercase instrument id with dash, such as `BTC-USDT` |

Negative `min_order_size` values are not currently rejected by the backend; SDK callers should avoid sending them.

### `DepositTokenMetadataResponse`

Returned in `UnifiedMetadataResponse.deposit_tokens` after `upsert_metadata`.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `i64` | Metadata row ID |
| `deposit_asset` | `String` | Mint/address identifier |
| `display_name` / `symbol` | `String` | Non-null display fields |
| `token_symbol` / `binance_symbol` / `okx_inst_id` | `Option<String>` | Optional symbol fields |
| `binance_enabled` | `bool` | Binance integration flag |
| `description`, icon URLs, `metadata_uri` | `Option<String>` | Optional metadata fields |
| `decimals` | `Option<i16>` | Token decimals |
| `min_order_size` | `i64` | Raw integer token units; non-null in responses |
| `created_at` / `updated_at` | `DateTime<Utc>` | Backend timestamps |

### `MetadataCategoriesResponse`

| Field | Type | Description |
|-------|------|-------------|
| `categories` | `Vec<String>` | Whitelisted market categories |

### `AddMetadataCategoryRequest`

| Field | Type | Description |
|-------|------|-------------|
| `category` | `String` | Category to add to the whitelist |

### `AddMetadataCategoryResponse`

| Field | Type | Description |
|-------|------|-------------|
| `category` | `String` | Canonical category stored by the backend |

### `UploadMarketDeploymentAssetsRequest`

| Field | Type | Description |
|-------|------|-------------|
| `market_id` | `i64` | On-chain market ID |
| `market_pubkey` | `String` | On-chain market public key |
| `market` | `MarketDeploymentMarket` | Market-level fields and image payloads |
| `outcomes` | `Vec<MarketDeploymentOutcome>` | Per-outcome metadata + images (default empty) |
| `deposit_assets` | `Vec<MarketDeploymentDepositAsset>` | Deposit assets referenced by this market (default empty) |
| `conditional_tokens` | `Vec<MarketDeploymentConditionalToken>` | Per-token metadata + image (default empty) |

#### `MarketDeploymentMarket`

| Field | Type | Description |
|-------|------|-------------|
| `name` / `slug` | `String` | Required display name + URL slug |
| `description` / `definition` | `Option<String>` | Long description and resolution definition |
| `banner_image_url_low` / `_medium` / `_high` | `Option<String>` | Existing hosted banner URLs by quality (used when no matching upload data URL is supplied) |
| `icon_url_low` / `_medium` / `_high` | `Option<String>` | Existing hosted icon URLs by quality (used when no matching upload data URL is supplied) |
| `category` / `subcategory` | `Option<String>` | Categorization |
| `tags` | `Vec<String>` | Free-form tags (default empty) |
| `featured_rank` | `Option<i32>` | Rank on the featured list, if any |
| `banner_image_data_url_low` / `_medium` / `_high` | `Option<String>` | New banner WebP upload data URLs by quality |
| `banner_image_content_type_low` / `_medium` / `_high` | `Option<String>` | Matching banner content types; must be `image/webp` when the matching data URL is supplied |
| `icon_image_data_url_low` / `_medium` / `_high` | `Option<String>` | New market icon WebP upload data URLs by quality |
| `icon_image_content_type_low` / `_medium` / `_high` | `Option<String>` | Matching market icon content types; must be `image/webp` when the matching data URL is supplied |

#### `MarketDeploymentOutcome`

| Field | Type | Description |
|-------|------|-------------|
| `index` | `i32` | Outcome index within the market |
| `name` / `symbol` | `String` | Display name and short symbol |
| `description` | `Option<String>` | Optional long description |
| `icon_url_low` / `_medium` / `_high` | `Option<String>` | Existing hosted icon URLs by quality (used when no matching upload data URL is supplied) |
| `icon_image_data_url_low` / `_medium` / `_high` | `Option<String>` | New outcome icon WebP upload data URLs by quality |
| `icon_image_content_type_low` / `_medium` / `_high` | `Option<String>` | Matching outcome icon content types; must be `image/webp` when the matching data URL is supplied |

#### `MarketDeploymentDepositAsset`

| Field | Type | Description |
|-------|------|-------------|
| `mint` | `String` | Deposit asset mint (base58) |
| `display_name` / `symbol` | `String` | Display name and ticker symbol |
| `decimals` | `i32` | Token decimals |
| `description` | `Option<String>` | Optional description |
| `icon_url_low` / `_medium` / `_high` | `Option<String>` | Icon URLs by quality |

#### `MarketDeploymentConditionalToken`

| Field | Type | Description |
|-------|------|-------------|
| `outcome_index` | `i32` | Associated outcome index |
| `deposit_mint` / `conditional_mint` | `String` | Underlying deposit mint and derived conditional mint |
| `name` / `symbol` | `String` | Display name and ticker symbol for the conditional token |
| `description` | `Option<String>` | Optional description |
| `image_data_url_low` / `_medium` | `Option<String>` | Optional conditional token WebP upload data URLs |
| `image_content_type_low` / `_medium` | `Option<String>` | Matching optional content types; required when the matching low/medium data URL is supplied |
| `image_data_url_high` / `image_content_type_high` | `String` | Required conditional token WebP upload; content type must be `image/webp` |

All upload data URLs must start with `data:image/webp;base64,`.

### `UploadMarketDeploymentAssetsResponse`

| Field | Type | Description |
|-------|------|-------------|
| `market_metadata_uri` | `String` | URI of the uploaded market metadata JSON |
| `market` | `UploadedMarketImages` | Resolved banner/icon URLs for the market |
| `outcomes` | `Vec<UploadedOutcomeImages>` | Resolved icon URLs per outcome |
| `deposit_assets` | `Vec<UploadedDepositAssetImages>` | Resolved icon URLs per deposit asset |
| `tokens` | `Vec<UploadedConditionalToken>` | Resolved image + metadata URIs per conditional token |

#### `UploadedMarketImages`

| Field | Type | Description |
|-------|------|-------------|
| `banner_image_url_low` / `_medium` / `_high` | `Option<String>` | Uploaded banner URLs by quality (or `None` if not supplied) |
| `icon_url_low` / `_medium` / `_high` | `Option<String>` | Uploaded icon URLs by quality (or `None` if not supplied) |

#### `UploadedOutcomeImages`

| Field | Type | Description |
|-------|------|-------------|
| `index` | `i32` | Outcome index |
| `icon_url_low` / `_medium` / `_high` | `Option<String>` | Uploaded icon URLs by quality (or `None` if not supplied) |

#### `UploadedDepositAssetImages`

| Field | Type | Description |
|-------|------|-------------|
| `mint` | `String` | Deposit asset mint |
| `icon_url_low` / `_medium` / `_high` | `Option<String>` | Resolved icon URLs by quality |

#### `UploadedConditionalToken`

| Field | Type | Description |
|-------|------|-------------|
| `conditional_mint` | `String` | Conditional token mint |
| `metadata_uri` | `String` | Uploaded metadata JSON URI |
| `image_url_low` / `_medium` / `_high` | `Option<String>` | Uploaded conditional token image URLs by quality |

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

### `ReferralConfig`

| Field | Type | Description |
|-------|------|-------------|
| `default_code_count` | `i32` | Codes allocated per new user |
| `updated_at` | `DateTime<Utc>` | Last time the config changed |

### `UpdateConfigRequest`

| Field | Type | Description |
|-------|------|-------------|
| `default_code_count` | `Option<i32>` | New default; `None` is a no-op |

### `ListCodesRequest`

| Field | Type | Description |
|-------|------|-------------|
| `owner_user_id` / `batch_id` / `code` | `Option<String>` | Filters |
| `limit` | `u32` | Max codes per page |
| `offset` | `u32` | Offset for pagination |

### `ListCodesResponse`

| Field | Type | Description |
|-------|------|-------------|
| `codes` | `Vec<CodeListEntry>` | Codes matching the filter |
| `count` | `usize` | Total matching codes (across pages) |

### `CodeListEntry`

| Field | Type | Description |
|-------|------|-------------|
| `code` | `String` | The referral code |
| `owner_user_id` | `String` | User who owns the code |
| `batch_id` | `String` | Batch identifier |
| `is_vanity` | `bool` | Whether it was manually assigned |
| `max_uses` | `i32` | Maximum redemptions |
| `use_count` | `i64` | Redemptions so far |
| `created_at` | `DateTime<Utc>` | When the code was created |

### `UpdateCodeRequest` / `UpdateCodeResponse`

| Field | Type | Description |
|-------|------|-------------|
| `code` | `String` | The referral code to update |
| `max_uses` | `i32` | New maximum redemption count |

### `AdminLogEventsQuery`

Filter set for the events listing endpoint. All fields optional. Keys include `from_ms`, `to_ms`, `service_name`, `service_names`, `environment`, `environments`, `category`, `categories`, `severity`, `severities`, `component`, `components`, `operation`, `operations`, `fingerprint`, `fingerprints`, `response_status`, `response_statuses`, `error_code`, `error_codes`, `rejection_code`, `rejection_codes`, `user_visible`, `request_id`, `user_pubkey` (`PubkeyStr`), `market_pubkey` (`PubkeyStr`), `orderbook_id` (`OrderBookId`), `order_hash`, `trigger_order_id`, `tx_signature`, `checkpoint_signature`, `limit`, and `cursor`.

Plural filter fields are comma-separated strings. Different filter dimensions are combined with `AND`; multiple values within one dimension are combined with `OR`. Singular and plural filters for the same dimension are merged by the backend. Keep the same filters when reusing `cursor` for pagination.

The logging service persists only `error` and `critical` events. Queries for `warning` or `info` severity should not be expected to return newly ingested logs.

### `AdminLogEvent`

A single structured log event. Key fields: `id`, `public_id`, `service_name`, `environment`, `component`, `operation`, `category`, `severity`, `occurred_at_ms`, `created_at_ms`, `user_visible`, optional `error_code`, optional `rejection_code`, and free-form `context: serde_json::Value`. Entity bindings (`user_pubkey`, `market_pubkey`, `orderbook_id`, etc.) are all optional.

### `AdminLogEventsResponse`, `AdminLogMetricsResponse`, `AdminLogMetricHistoryResponse`

Envelope types holding paged events, breakdown rows, and history points respectively. See [`wire.rs`](./wire.rs) for the exact field list.

### `AdminMarketsQuery`

Query type for `GET /api/admin/markets`. All fields are optional; omitted fields let the backend apply its defaults.

| Field | Type | Description |
|-------|------|-------------|
| `cursor` | `Option<u64>` | Offset cursor. Reset to `0` when changing sort or filters |
| `limit` | `Option<u32>` | Page size. Backend default is `200` |
| `sort_by` | `Option<String>` | Sort field. Backend default is `volume_24h_usd` |
| `sort_direction` | `Option<String>` | `asc` or `desc`. Backend default is `desc` |
| `market_status` | `Option<AdminMarketStatusFilter>` | `All`, `Active`, or `Resolved`. Serializes as `all`, `active`, or `resolved` |
| `category` | `Option<String>` | Category filter |
| `search` | `Option<String>` | Search filter |

Every sortable numeric field also has `min_` and `max_` filters. USD filters use `Option<Decimal>` and unique-trader filters use `Option<u64>`.

Sortable/range fields: `volume_24h_usd`, `volume_7d_usd`, `volume_30d_usd`, `volume_total_usd`, `unique_traders_24h`, `unique_traders_7d`, `unique_traders_30d`, `unique_traders_total`, `open_interest_usd`, `fees_24h_usd`, `fees_7d_usd`, `fees_30d_usd`, and `fees_total_usd`.

Do not send `settled` as a market status filter. Resolved markets are selected with `AdminMarketStatusFilter::Resolved`.

### `AdminMarketsResponse` / `AdminMarketRow`

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | `i64` | Metrics cache timestamp in milliseconds |
| `sort_by` / `sort_direction` | `String` | Sort actually used by the backend |
| `total` | `u64` | Total matching rows |
| `limit` | `u32` | Page limit |
| `next_cursor` | `Option<u64>` | Offset cursor for the next page |
| `has_more` | `bool` | Whether another page is available |
| `markets` | `Vec<AdminMarketRow>` | Current page of admin market table rows |

`AdminMarketRow` includes market identity/display fields, `market_status: AdminMarketStatus` (`Active` or `Resolved`), optional `resolution_by`, outcome count, decimal USD metrics, unique-trader metrics, and lifecycle timestamps. USD values deserialize from API strings into `Decimal`.

### `MarketsToSettleQuery`

| Field | Type | Description |
|-------|------|-------------|
| `cursor` | `Option<i64>` | Previous page's `next_cursor`, using `market_id` |
| `limit` | `Option<u32>` | Page size. Backend default is `200`; backend max is `1000` |

### `MarketsToSettleCountResponse` / `MarketsToSettleResponse`

| Field | Type | Description |
|-------|------|-------------|
| `markets_to_settle_count` | `u64` | Count of active markets past their resolution time |
| `markets` | `Vec<MarketResponse>` | Ready-to-settle markets using the existing market response shape |
| `next_cursor` | `Option<i64>` | Cursor for the next page |
| `has_more` | `bool` | Whether another page is available |

### `CriticalLogErrors24hCountResponse`

| Field | Type | Description |
|-------|------|-------------|
| `critical_log_errors_24h` | `u64` | Critical logging error count for the previous 24 hours |

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
