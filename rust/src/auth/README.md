# Authentication

Session management, login flows, and user profiles.

[← Overview](../../README.md#authentication)

## Table of Contents

- [Security Model](#security-model)
- [Types](#types)
- [Client Methods](#client-methods)
- [Native Login Flow](#native-login-flow)
- [OAuth (Browser Only)](#oauth-browser-only)
- [Examples](#examples)

## Security Model

The SDK uses a cookie-based auth model with platform-specific handling:

### WASM / Browser

- Token lives **only** in an HTTP-only cookie set by the backend.
- The SDK **never** reads, stores, or exposes the token.
- Authenticated requests work because the browser auto-includes cookies.
- Never store tokens in localStorage, sessionStorage, or any JS/WASM-accessible location.

### Native / CLI

- The SDK stores the token **internally** (private field) and injects it as a `Cookie: auth_token=<token>` header.
- Token is **never** exposed via public API -- no `.token()` accessor.
- `AuthCredentials` only exposes: `user_id`, `wallet_address`, `expires_at`, `is_authenticated()`.

### Logout

On **both** platforms, `client.auth().logout()`:
1. Calls `POST /api/auth/logout` to clear the server-side cookie.
2. On native: clears the internal token.
3. Clears auth credentials.

Client-side clearing alone is insufficient -- the backend must be told to invalidate.

## Types

### `User`

Full user profile returned by `login_with_message()` and `check_session()`.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | User ID |
| `wallet_address` | `String` | Primary Solana wallet address |
| `linked_account` | `LinkedAccount` | Primary linked identity |
| `privy_id` | `Option<String>` | Privy user ID (if embedded wallet) |
| `embedded_wallet` | `Option<EmbeddedWallet>` | Privy embedded wallet info |
| `x_username` | `Option<String>` | Linked X (Twitter) username |
| `x_user_id` | `Option<String>` | Linked X user ID |
| `x_display_name` | `Option<String>` | Linked X display name |

### `LinkedAccount`

A linked identity associated with a user.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Account ID |
| `account_type` | `LinkedAccountType` | `Wallet`, `TwitterOauth`, or `GoogleOauth` |
| `chain` | `Option<ChainType>` | `Solana` or `Ethereum` (for wallets) |
| `address` | `String` | Account address/identifier |

### `AuthCredentials`

Session state (token is never exposed).

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | `String` | Authenticated user ID |
| `wallet_address` | `PubkeyStr` | User's wallet address |
| `expires_at` | `DateTime<Utc>` | Session expiration time |

**Methods:**
- `is_authenticated()` -- whether the session is still valid (not expired)

### `EmbeddedWallet`

A Privy-managed embedded wallet.

| Field | Type | Description |
|-------|------|-------------|
| `privy_id` | `String` | Privy identifier |
| `chain` | `ChainType` | `Solana` or `Ethereum` |
| `address` | `String` | Wallet address |

## Client Methods

Access via `client.auth()`.

### `get_nonce`

```rust
async fn get_nonce(&self) -> Result<String, SdkError>
```

Fetch a single-use nonce from the server for the sign-in challenge. The nonce has a 5-minute TTL and is consumed on login.

### `login_with_message`

```rust
async fn login_with_message(
    &self,
    message: &str,
    signature_bs58: &str,
    pubkey_bytes: &[u8; 32],
    use_embedded_wallet: Option<bool>,
) -> Result<User, SdkError>
```

Authenticate with a pre-signed message. Returns the full user profile. On native, stores the auth token internally. On WASM, the backend sets an HTTP-only cookie.

Set `use_embedded_wallet` to `Some(true)` to provision a Privy embedded wallet during login.

### `check_session`

```rust
async fn check_session(&self) -> Result<User, SdkError>
```

Validate the current session and return the full user profile. Works on both WASM (browser sends cookie) and native (SDK injects cookie header). Clears credentials on failure.

### `logout`

```rust
async fn logout(&self) -> Result<(), SdkError>
```

Log out -- clears server-side cookie, internal token (native), and auth credentials.

### `credentials`

```rust
async fn credentials(&self) -> Option<AuthCredentials>
```

Get current session state, if authenticated.

### `is_authenticated`

```rust
async fn is_authenticated(&self) -> bool
```

Quick check based on cached credentials. For server-validated check, use `check_session()`.

### `connect_x_url`

```rust
fn connect_x_url(&self) -> String
```

Get the URL for linking an X (Twitter) account via OAuth. Opens in a browser to complete the flow.

### `disconnect_x`

```rust
async fn disconnect_x(&self) -> Result<(), SdkError>
```

Disconnect the user's linked X (Twitter) account.

## Native Login Flow

Native clients authenticate using a nonce-based signature challenge. Requires the `native-auth` feature.

```rust
use lightcone::prelude::*;
use lightcone::auth::native::sign_login_message;
use solana_keypair::Keypair;

async fn login(client: &LightconeClient, keypair: &Keypair) -> Result<User, SdkError> {
    // 1. Fetch a single-use nonce (5-minute TTL, consumed on login)
    let nonce = client.auth().get_nonce().await?;

    // 2. Build message + sign with keypair
    let signed = sign_login_message(keypair, &nonce);

    // 3. Authenticate
    let user = client.auth().login_with_message(
        &signed.message,
        &signed.signature_bs58,
        &signed.pubkey_bytes,
        None,
    ).await?;

    println!("Logged in as: {} ({})", user.id, user.wallet_address);
    Ok(user)
}
```

The `sign_login_message` helper:
1. Builds the message: `"Sign in to Lightcone\nNonce: {nonce}"`
2. Signs it with the keypair's ED25519 key
3. Returns a `SignedLogin` with the message, base58 signature, and public key bytes

If login fails, fetch a **new** nonce -- each nonce can only be used once.

## OAuth (Browser Only)

OAuth login (Google, X/Twitter) is a browser redirect flow handled by the backend -- not an SDK method call.

| Flow | URL |
|------|-----|
| Login with Google | `GET {backend}/api/auth/oauth/google` |
| Login with X | `GET {backend}/api/auth/oauth/x` |
| Link X account | `GET {backend}/api/auth/oauth/link/x` (requires session) |

After the redirect completes, call `check_session()` to hydrate the user profile.

Native/CLI clients use `get_nonce()` + `login_with_message()` instead.

## Examples

### Session management

```rust
use lightcone::prelude::*;

async fn manage_session(client: &LightconeClient) -> Result<(), SdkError> {
    // Check if we have a valid session
    if client.auth().is_authenticated().await {
        let user = client.auth().check_session().await?;
        println!("Authenticated as: {}", user.wallet_address);
    } else {
        println!("Not authenticated");
    }

    // Logout
    client.auth().logout().await?;
    assert!(!client.auth().is_authenticated().await);

    Ok(())
}
```

---

[← Overview](../../README.md#authentication)
