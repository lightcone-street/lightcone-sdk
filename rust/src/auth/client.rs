//! Auth sub-client — login, logout, session validation, user profile.

use chrono::{DateTime, TimeZone, Utc};

use crate::auth::{AuthCredentials, LoginRequest, NonceResponse, SessionResponse};
use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::shared::PubkeyStr;

/// Sub-client for authentication operations.
pub struct Auth<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Auth<'a> {
    /// Fetch a single-use nonce from the server for the sign-in challenge.
    ///
    /// The nonce must be embedded in the sign-in message before signing.
    /// Use [`generate_signin_message`](crate::auth::generate_signin_message)
    /// to build the message.
    pub async fn get_nonce(&self) -> Result<String, SdkError> {
        let url = format!("{}/api/auth/nonce", self.client.http.base_url());
        let body: NonceResponse = self.client.http.get(&url, RetryPolicy::None).await?;
        Ok(body.nonce)
    }

    /// Login with a pre-signed message and return the session envelope.
    ///
    /// The caller signs a message externally (wallet adapter on WASM, keypair
    /// on native) and passes the result here.
    ///
    /// - On native: stores the token internally for cookie header injection.
    /// - On WASM: the backend sets an HTTP-only cookie; the SDK never touches the token.
    ///
    /// The backend returns the full user profile in the session envelope, so no
    /// separate `check_session()` call is needed. For new users the backend uses
    /// direct DB joins (guaranteed fresh); for existing users it uses the MV.
    ///
    /// Set `use_embedded_wallet` to `Some(true)` to provision a Privy
    /// embedded wallet for the user during login (works on any platform).
    pub async fn login_with_message(
        &self,
        message: &str,
        signature_bs58: &str,
        pubkey_bytes: &[u8; 32],
        use_embedded_wallet: Option<bool>,
    ) -> Result<SessionResponse, SdkError> {
        let request = LoginRequest {
            message: message.to_string(),
            signature_bs58: signature_bs58.to_string(),
            pubkey_bytes: pubkey_bytes.to_vec(),
            use_embedded_wallet,
        };

        let url = format!(
            "{}/api/auth/login_or_register_with_message",
            self.client.http.base_url()
        );
        let session: SessionResponse = self
            .client
            .http
            .post(&url, &request, RetryPolicy::None)
            .await?;

        let credentials = credentials_from_session(&session);
        *self.client.auth_credentials.write().await = Some(credentials);

        Ok(session)
    }

    /// Validate the current session and return the session envelope.
    ///
    /// Calls `GET /api/auth/me` — works on both WASM (browser sends cookie
    /// automatically) and native (SDK injects cookie header).
    ///
    /// On success, updates internal `AuthCredentials` so `is_authenticated()`
    /// returns correct results. On failure (401, expired, no cookie), clears
    /// internal credentials and returns an error.
    pub async fn check_session(&self) -> Result<SessionResponse, SdkError> {
        let url = format!("{}/api/auth/me", self.client.http.base_url());

        let session: SessionResponse = match self
            .client
            .http
            .get::<SessionResponse>(&url, RetryPolicy::Idempotent)
            .await
        {
            Ok(body) => body,
            Err(error) => {
                *self.client.auth_credentials.write().await = None;
                return Err(error);
            }
        };

        let credentials = credentials_from_session(&session);
        *self.client.auth_credentials.write().await = Some(credentials);

        Ok(session)
    }

    /// Same as [`Self::check_session`], but forwards the supplied raw `Cookie`
    /// header for this call instead of the SDK's process-wide token store, and
    /// does **not** mutate the shared `auth_credentials` (safe under concurrent
    /// SSR). The header should carry whichever auth cookies the browser sent
    /// (e.g. `"privy-token=…; lightcone-token=…"`) so the backend authenticates
    /// the SSR request exactly as it would a client request. Returns both the
    /// session envelope and the parsed `AuthCredentials` so SSR consumers can
    /// read the wallet + token expiry without making a follow-up call.
    pub async fn check_session_with_cookies(
        &self,
        cookie_header: &str,
    ) -> Result<(SessionResponse, AuthCredentials), SdkError> {
        let url = format!("{}/api/auth/me", self.client.http.base_url());

        let session: SessionResponse = self
            .client
            .http
            .get_with_cookies::<SessionResponse>(&url, RetryPolicy::Idempotent, cookie_header)
            .await?;

        let credentials = credentials_from_session(&session);

        Ok((session, credentials))
    }

    /// Logout — clears server-side cookie + internal token + all caches.
    pub async fn logout(&self) -> Result<(), SdkError> {
        let url = format!("{}/api/auth/logout", self.client.http.base_url());
        let _ = self
            .client
            .http
            .post::<serde_json::Value, _>(&url, &serde_json::json!({}), RetryPolicy::None)
            .await;

        #[cfg(not(target_arch = "wasm32"))]
        self.client.http.clear_auth_token().await;

        *self.client.auth_credentials.write().await = None;

        Ok(())
    }

    /// Register a Privy-authenticated user in the backend DB.
    /// Called after Privy login when `is_new_user: true`.
    /// Idempotent — safe to call multiple times.
    pub async fn register_privy(&self) -> Result<(), SdkError> {
        let url = format!("{}/api/auth/register-privy", self.client.http.base_url());
        let _: serde_json::Value = self
            .client
            .http
            .post(&url, &serde_json::json!({}), RetryPolicy::None)
            .await?;
        Ok(())
    }

    /// Disconnect the user's linked X (Twitter) account.
    pub async fn disconnect_x(&self) -> Result<(), SdkError> {
        let url = format!("{}/api/auth/disconnect_x", self.client.http.base_url());
        let _: serde_json::Value = self
            .client
            .http
            .post(&url, &serde_json::json!({}), RetryPolicy::None)
            .await?;
        Ok(())
    }

    /// Get the URL for linking an X (Twitter) account via OAuth.
    pub fn connect_x_url(&self) -> String {
        format!("{}/api/auth/oauth/link/x", self.client.http.base_url())
    }

    /// Get current auth credentials (if authenticated).
    pub async fn credentials(&self) -> Option<AuthCredentials> {
        self.client.auth_credentials.read().await.clone()
    }

    /// Check if currently authenticated (based on cached credentials).
    ///
    /// For a server-validated check, use `check_session()` instead.
    pub async fn is_authenticated(&self) -> bool {
        self.client
            .auth_credentials
            .read()
            .await
            .as_ref()
            .map(|c| c.is_authenticated())
            .unwrap_or(false)
    }
}

/// Derive session credentials from the envelope. The trading wallet comes
/// from the identity + auth method.
fn credentials_from_session(session: &SessionResponse) -> AuthCredentials {
    AuthCredentials {
        user_id: session.user.user_id.clone(),
        wallet_address: PubkeyStr::from(session.user.trading_wallet(session.auth_method)),
        expires_at: parse_expires_at(session.expires_at),
    }
}

fn parse_expires_at(timestamp: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .unwrap_or_else(Utc::now)
}
