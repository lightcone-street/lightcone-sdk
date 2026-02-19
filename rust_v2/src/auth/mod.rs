//! Authentication — message generation, credentials, login/logout, user profile.
//!
//! ## Security Model
//!
//! - **Wasm/Browser**: Token lives ONLY in the HTTP-only cookie set by the backend.
//!   The SDK never reads, stores, or exposes it. Browser auto-includes cookies.
//! - **Native/CLI**: SDK stores the token internally (private field) and injects it
//!   as a `Cookie: auth_token=<token>` header, matching the backend's cookie-only auth.
//!   Token is NEVER exposed via public API — no `.token()` accessor.
//! - **Logout**: MUST call `POST /api/auth/logout` to clear server-side cookie.
//!   On native, also clears internal token + caches.
//!
//! ## Session Hydration
//!
//! Use `client.auth().check_session()` to validate the current cookie and
//! retrieve the full user profile. Works identically on WASM (browser cookie)
//! and native (injected cookie header). Returns [`User`] on success.

#[cfg(feature = "http")]
pub mod client;

#[cfg(feature = "native-auth")]
pub mod native;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::PubkeyStr;

// ============================================================================
// User profile types
// ============================================================================

/// Full user profile from the Lightcone platform.
///
/// Returned by `client.auth().check_session()` and `client.auth().login_with_message()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub wallet_address: String,
    pub linked_account: LinkedAccount,
    pub privy_id: Option<String>,
    pub embedded_wallet: Option<EmbeddedWallet>,
    pub x_username: Option<String>,
    pub x_user_id: Option<String>,
    pub x_display_name: Option<String>,
}

/// A linked identity (wallet, Google OAuth, X OAuth) associated with a user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinkedAccount {
    pub id: String,
    #[serde(rename = "type")]
    pub account_type: LinkedAccountType,
    pub chain: Option<ChainType>,
    pub address: String,
}

/// Type of linked account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkedAccountType {
    Wallet,
    TwitterOauth,
    GoogleOauth,
}

impl LinkedAccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Wallet => "wallet",
            Self::TwitterOauth => "twitter_oauth",
            Self::GoogleOauth => "google_oauth",
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            Self::Wallet => "Solana",
            Self::TwitterOauth => "X",
            Self::GoogleOauth => "Google",
        }
    }
}

impl std::fmt::Display for LinkedAccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A Privy-managed embedded wallet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddedWallet {
    pub privy_id: String,
    pub chain: ChainType,
    pub address: String,
}

/// Blockchain network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChainType {
    Solana,
    Ethereum,
}

impl ChainType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Solana => "solana",
            Self::Ethereum => "ethereum",
        }
    }
}

impl std::fmt::Display for ChainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Auth session types
// ============================================================================

/// Internal auth session state. Token is NEVER exposed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub user_id: String,
    pub wallet_address: PubkeyStr,
    pub expires_at: DateTime<Utc>,
}

impl AuthCredentials {
    /// Whether the session is still valid (not expired).
    pub fn is_authenticated(&self) -> bool {
        Utc::now() < self.expires_at
    }
}

// ============================================================================
// Wire types
// ============================================================================

/// Generate the sign-in message that must be signed by the user's wallet.
///
/// This is always available (no feature gates). The caller signs this message
/// externally (wallet adapter on WASM, keypair on native) and passes the
/// signature back to `client.auth().login_with_message(...)`.
pub fn generate_signin_message(timestamp: u64) -> Vec<u8> {
    let message = format!("Sign in to Lightcone\nTimestamp: {}", timestamp);
    message.into_bytes()
}

/// Login request body sent to the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub message: String,
    pub signature_bs58: String,
    pub pubkey_bytes: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_embedded_wallet: Option<bool>,
}

/// Login response from the backend.
///
/// Includes the full user profile so no separate `check_session()` is needed
/// after login. The backend uses direct joins for new users (guaranteed fresh)
/// and the materialized view for existing users (fast).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub wallet_address: String,
    pub expires_at: i64,
    pub linked_account: LinkedAccount,
    pub privy_id: Option<String>,
    pub embedded_wallet: Option<EmbeddedWallet>,
    pub x_username: Option<String>,
    pub x_user_id: Option<String>,
    pub x_display_name: Option<String>,
}

/// Response from `GET /api/auth/me`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeResponse {
    pub user_id: String,
    pub wallet_address: String,
    pub linked_account: LinkedAccount,
    pub privy_id: Option<String>,
    pub embedded_wallet: Option<EmbeddedWallet>,
    pub x_username: Option<String>,
    pub x_user_id: Option<String>,
    pub x_display_name: Option<String>,
    pub expires_at: i64,
}
