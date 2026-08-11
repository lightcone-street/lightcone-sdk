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

- The SDK stores the token **internally** (private field) and injects it as a `Cookie: lightcone-token=<token>` header.
- Token is **never** exposed via public API -- no `.token()` accessor.
- `AuthCredentials` only exposes: `user_id`, `wallet_address`, `expires_at`, `is_authenticated()`.

### Logout

On **both** platforms, `client.auth().logout()`:
1. Calls `POST /api/auth/logout` to clear the server-side cookie.
2. On native: clears the internal token.
3. Clears auth credentials.

Client-side clearing alone is insufficient -- the backend must be told to invalidate.

## Types

### `SessionResponse`

Session envelope returned by `login_with_message()` and `check_session()`:
the durable user profile plus session-scoped facts. There is no
`wallet_address` field — derive the session's trading wallet with
`session.user.trading_wallet(session.auth_method)`.

| Field | Type | Description |
|-------|------|-------------|
| `user` | `User` | Full user profile |
| `expires_at` | `i64` | Session expiry (Unix seconds) |
| `auth_method` | `AuthMethod` | `Privy` or `Lightcone` (which token verified the session) |
| `is_beta` | `bool` | Whether the user has beta access |

### `User`

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | `String` | User ID |
| `identity` | `UserIdentity` | The login identity (tagged union) |
| `max_slippage_preference` | `Option<Decimal>` | Account-wide percentage; `None` until explicitly changed |
| `connected_x` | `Option<XAccountData>` | X account connected by a non-X-identity user; `None` when identity is X |

**Methods:**
- `privy()` — Privy account data regardless of identity type
- `x_account()` — the X account, whether login identity or connected account
- `trading_wallet(auth_method)` — the wallet this session operates as. Google/X identities always trade via their Privy embedded wallet; wallet identities trade via the embedded wallet on Privy (SIWS) sessions and via the sign-in wallet on Lightcone sessions
- `wallet_display_name(auth_method)` — shortened display label for the session's trading wallet (`FRGk...WcPR`)
- `display_name()` — Google: `name` → email fallback; X: `display_name` → username fallback; wallet: shortened address (`FRGk...WcPR`)
- `avatar_url()` — avatar from the login identity's OAuth provider

### `UserIdentity`

How the user authenticates. Serializes as a tagged union on `type`
(`"google"` / `"x"` / `"wallet"`). Privy data lives on the variant because
Google/X login only exists via Privy (always present), while wallet users opt
in (SIWS) or stay self-custody (`None`).

| Variant | Fields |
|---------|--------|
| `Google` | `account: GoogleAccountData`, `privy: UserPrivyData` |
| `X` | `account: XAccountData`, `privy: UserPrivyData` |
| `Wallet` | `address: String`, `chain: ChainType`, `privy: Option<UserPrivyData>` |

**Methods:**
- `text()` — human-readable label: `"Google"` / `"X"` / `"Solana"`

### `GoogleAccountData`

| Field | Type |
|-------|------|
| `email` | `String` |
| `name` | `Option<String>` |
| `given_name` | `Option<String>` |
| `family_name` | `Option<String>` |
| `avatar_url` | `Option<String>` |

### `XAccountData`

Same shape whether X is the login identity or a connected account.

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | `Option<String>` | X numeric user id |
| `username` | `String` | X handle |
| `display_name` | `Option<String>` | Profile display name |
| `avatar_url` | `Option<String>` | Profile picture URL |

### `UserPrivyData`

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Privy DID (`did:privy:...`) |
| `wallet` | `PrivyEmbeddedWallet` | Embedded trading wallet (always provisioned at registration) |

### `PrivyEmbeddedWallet`

| Field | Type | Description |
|-------|------|-------------|
| `privy_id` | `String` | Privy wallet identifier |
| `chain` | `ChainType` | `Solana` or `Ethereum` |
| `address` | `String` | Wallet address |

### `AuthCredentials`

Session state (token is never exposed). The `wallet_address` is derived from
the session via `trading_wallet`.

| Field | Type | Description |
|-------|------|-------------|
| `user_id` | `String` | Authenticated user ID |
| `wallet_address` | `PubkeyStr` | The session's active trading wallet |
| `expires_at` | `DateTime<Utc>` | Session expiration time |

**Methods:**
- `is_authenticated()` -- whether the session is still valid (not expired)

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
) -> Result<SessionResponse, SdkError>
```

Authenticate with a pre-signed message. Returns the session envelope with the full user profile. On native, stores the auth token internally. On WASM, the backend sets an HTTP-only cookie.

Set `use_embedded_wallet` to `Some(true)` to provision a Privy embedded wallet during login.

### `check_session`

```rust
async fn check_session(&self) -> Result<SessionResponse, SdkError>
```

Validate the current session and return the session envelope. Works on both WASM (browser sends cookie) and native (SDK injects cookie header). Clears credentials on failure.

### `check_session_with_cookies`

```rust
async fn check_session_with_cookies(
    &self,
    cookie_header: &str,
) -> Result<(SessionResponse, AuthCredentials), SdkError>
```

Same as `check_session`, but forwards the supplied raw `Cookie` header for this call and does **not** mutate the shared credentials (safe under concurrent SSR). Returns the parsed credentials alongside the envelope.

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

### `update_max_slippage_preference`

```rust
async fn update_max_slippage_preference(
    &self,
    max_slippage_preference: Decimal,
) -> Result<Decimal, SdkError>
```

Persist the authenticated user's account-wide percentage preference. The
backend accepts any decimal greater than zero, with no policy maximum, and
returns the canonical exact decimal value. Session user profiles return `None`
until the first explicit update.

## Native Login Flow

Native clients authenticate using a nonce-based signature challenge. Requires the `native-auth` feature.

```rust
use lightcone::prelude::*;
use lightcone::auth::native::sign_login_message;
use solana_keypair::Keypair;

async fn login(client: &LightconeClient, keypair: &Keypair) -> Result<SessionResponse, SdkError> {
    // 1. Fetch a single-use nonce (5-minute TTL, consumed on login)
    let nonce = client.auth().get_nonce().await?;

    // 2. Build message + sign with keypair
    let signed = sign_login_message(keypair, &nonce);

    // 3. Authenticate
    let session = client.auth().login_with_message(
        &signed.message,
        &signed.signature_bs58,
        &signed.pubkey_bytes,
        None,
    ).await?;

    let wallet = session.user.trading_wallet(session.auth_method);
    println!("Logged in as: {} ({})", session.user.user_id, wallet);
    Ok(session)
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
        let session = client.auth().check_session().await?;
        println!("Authenticated as: {}", session.user.trading_wallet(session.auth_method));
        println!("Signed-in via {}", session.user.identity.text());
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
