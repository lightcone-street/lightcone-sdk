//! Auth sub-client — login, logout, session validation, user profile.

use chrono::{DateTime, TimeZone, Utc};

use crate::auth::{AuthCredentials, LoginRequest, LoginResponse, MeResponse, NonceResponse, User};
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

    /// Login with a pre-signed message and return the full user profile.
    ///
    /// The caller signs a message externally (wallet adapter on WASM, keypair
    /// on native) and passes the result here.
    ///
    /// - On native: stores the token internally for cookie header injection.
    /// - On WASM: the backend sets an HTTP-only cookie; the SDK never touches the token.
    ///
    /// The backend returns the full user profile in the login response, so no
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
    ) -> Result<User, SdkError> {
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
        let login_resp: LoginResponse = self
            .client
            .http
            .post(&url, &request, RetryPolicy::None)
            .await?;

        let expires_at = parse_expires_at(login_resp.expires_at);
        let credentials = AuthCredentials {
            user_id: login_resp.user_id.clone(),
            wallet_address: PubkeyStr::from(login_resp.wallet_address.as_str()),
            expires_at,
        };
        *self.client.auth_credentials.write().await = Some(credentials);

        Ok(User {
            id: login_resp.user_id,
            wallet_address: login_resp.wallet_address,
            linked_account: login_resp.linked_account,
            privy_id: login_resp.privy_id,
            embedded_wallet: login_resp.embedded_wallet,
            x_username: login_resp.x_username,
            x_user_id: login_resp.x_user_id,
            x_display_name: login_resp.x_display_name,
            google_email: login_resp.google_email,
        })
    }

    /// Validate the current session and return the full user profile.
    ///
    /// Calls `GET /api/auth/me` — works on both WASM (browser sends cookie
    /// automatically) and native (SDK injects cookie header).
    ///
    /// On success, updates internal `AuthCredentials` so `is_authenticated()`
    /// returns correct results. On failure (401, expired, no cookie), clears
    /// internal credentials and returns an error.
    pub async fn check_session(&self) -> Result<User, SdkError> {
        let url = format!("{}/api/auth/me", self.client.http.base_url());

        let me: MeResponse = match self
            .client
            .http
            .get::<MeResponse>(&url, RetryPolicy::Idempotent)
            .await
        {
            Ok(body) => body,
            Err(error) => {
                *self.client.auth_credentials.write().await = None;
                return Err(error);
            }
        };

        let expires_at = parse_expires_at(me.expires_at);

        let credentials = AuthCredentials {
            user_id: me.user_id.clone(),
            wallet_address: PubkeyStr::from(me.wallet_address.as_str()),
            expires_at,
        };
        *self.client.auth_credentials.write().await = Some(credentials);

        Ok(User {
            id: me.user_id,
            wallet_address: me.wallet_address,
            linked_account: me.linked_account,
            privy_id: me.privy_id,
            embedded_wallet: me.embedded_wallet,
            x_username: me.x_username,
            x_user_id: me.x_user_id,
            x_display_name: me.x_display_name,
            google_email: me.google_email,
        })
    }

    /// Same as [`Self::check_session`], but uses the supplied `auth_token`
    /// for this call instead of the SDK's process-wide token store, and does
    /// **not** mutate the shared `auth_credentials` (safe under concurrent
    /// SSR). Returns both the validated `User` and the parsed
    /// `AuthCredentials` so SSR consumers can read the wallet + token
    /// expiry without making a follow-up call.
    pub async fn check_session_with_auth(
        &self,
        auth_token: &str,
    ) -> Result<(User, AuthCredentials), SdkError> {
        let url = format!("{}/api/auth/me", self.client.http.base_url());

        let me: MeResponse = self
            .client
            .http
            .get_with_auth::<MeResponse>(&url, RetryPolicy::Idempotent, auth_token)
            .await?;

        let expires_at = parse_expires_at(me.expires_at);

        let credentials = AuthCredentials {
            user_id: me.user_id.clone(),
            wallet_address: PubkeyStr::from(me.wallet_address.as_str()),
            expires_at,
        };

        let user = User {
            id: me.user_id,
            wallet_address: me.wallet_address,
            linked_account: me.linked_account,
            privy_id: me.privy_id,
            embedded_wallet: me.embedded_wallet,
            x_username: me.x_username,
            x_user_id: me.x_user_id,
            x_display_name: me.x_display_name,
            google_email: me.google_email,
        };

        Ok((user, credentials))
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

fn parse_expires_at(timestamp: i64) -> DateTime<Utc> {
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .unwrap_or_else(Utc::now)
}
