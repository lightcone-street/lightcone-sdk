//! Authentication — message generation, credentials, login/logout.
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

#[cfg(feature = "http")]
pub mod client;

#[cfg(feature = "native-auth")]
pub mod native;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::PubkeyStr;

/// Public auth credentials. Token is NEVER exposed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub user_id: Option<String>,
    pub wallet_address: PubkeyStr,
    pub expires_at: Option<DateTime<Utc>>,
}

impl AuthCredentials {
    /// Whether the credentials are still valid.
    pub fn is_authenticated(&self) -> bool {
        if let Some(exp) = self.expires_at {
            Utc::now() < exp
        } else {
            true
        }
    }
}

/// Generate the sign-in message that must be signed by the user's wallet.
///
/// This is always available (no feature gates). The caller signs this message
/// externally (wallet adapter on WASM, keypair on native) and passes the
/// signature back to `client.auth().login_with_message(...)`.
pub fn generate_signin_message(timestamp: u64) -> Vec<u8> {
    let message = format!(
        "Sign in to Lightcone\nTimestamp: {}",
        timestamp
    );
    message.into_bytes()
}

/// Login request body sent to the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub message: String,
    pub signature: String,
    pub pubkey: String,
}

/// Login response from the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub wallet: Option<String>,
}
