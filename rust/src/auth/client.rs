//! Auth sub-client — login, logout, session validation, user profile.

use chrono::{DateTime, TimeZone, Utc};

use crate::auth::{
    AuthCredentials, LoginRequest, MaxSlippagePreferenceBody, NonceResponse, RegisterPrivyRequest,
    SessionResponse,
};
use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::shared::PubkeyStr;
use rust_decimal::Decimal;

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
        // Credential-management endpoint: opts out of the transport's
        // 401 restore-and-replay. The backend consumes the login nonce before
        // verifying the signature, so a replayed login deterministically
        // fails — and restoring credentials in order to log in is circular.
        let session: SessionResponse = self
            .client
            .http
            .post_without_credential_restore(&url, &request, RetryPolicy::None)
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
    ///
    /// Local state (token, credentials) is cleared even when the server call
    /// fails — the caller asked to be signed out locally regardless — but the
    /// failure is then returned: callers gating security decisions on
    /// teardown (e.g. whether an app may restart an authenticated transport)
    /// must be able to see that the server-side cookie may still be valid.
    /// A 401 counts as success: it means "already logged out".
    pub async fn logout(&self) -> Result<(), SdkError> {
        let url = format!("{}/api/auth/logout", self.client.http.base_url());
        // Credential-management endpoint: opts out of the transport's
        // 401 restore-and-replay — a 401 here means "already logged out", and
        // restoring credentials just to log out again would be absurd.
        let logout_result = self
            .client
            .http
            .post_without_credential_restore::<serde_json::Value, _>(
                &url,
                &serde_json::json!({}),
                RetryPolicy::None,
            )
            .await;

        #[cfg(not(target_arch = "wasm32"))]
        self.client.http.clear_auth_token().await;

        *self.client.auth_credentials.write().await = None;

        match logout_result {
            Ok(_) => Ok(()),
            Err(error) if error.is_unauthorized() => Ok(()),
            Err(error) => Err(error),
        }
    }

    /// Create or synchronize a Privy Account and install the resulting session.
    pub async fn register_privy(
        &self,
        request: &RegisterPrivyRequest,
    ) -> Result<SessionResponse, SdkError> {
        let url = format!("{}/api/auth/register-privy", self.client.http.base_url());
        let session: SessionResponse = self
            .client
            .http
            .post(&url, request, RetryPolicy::Idempotent)
            .await?;
        let credentials = credentials_from_session(&session);
        *self.client.auth_credentials.write().await = Some(credentials);
        Ok(session)
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

    /// Persist an account-wide max-slippage preference strictly below 10%.
    pub async fn update_max_slippage_preference(
        &self,
        max_slippage_preference: Decimal,
    ) -> Result<Decimal, SdkError> {
        let url = format!(
            "{}/api/auth/max_slippage_preference",
            self.client.http.base_url()
        );
        let response: MaxSlippagePreferenceBody = self
            .client
            .http
            .post(
                &url,
                &MaxSlippagePreferenceBody {
                    max_slippage_preference,
                },
                RetryPolicy::Idempotent,
            )
            .await?;
        Ok(response.max_slippage_preference)
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

#[cfg(test)]
mod tests {
    use crate::client::LightconeClient;
    use rust_decimal::Decimal;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::sync::oneshot;

    // Minimal single-response server: the http-layer harness lives in a
    // private test module, and logout only needs one canned reply.
    async fn spawn_single_response_server(status: u16, body: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("logout test server failed to bind 127.0.0.1:0");
        let addr = listener
            .local_addr()
            .expect("logout test server has no local addr");
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buffer = [0_u8; 4096];
                    let _ = socket.read(&mut buffer).await;
                    let raw_response = format!(
                        "HTTP/1.1 {} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        status,
                        body.len(),
                        body
                    );
                    let _ = socket.write_all(raw_response.as_bytes()).await;
                });
            }
        });
        format!("http://{}", addr)
    }

    async fn client_with_token(base_url: &str) -> LightconeClient {
        let client = LightconeClient::builder()
            .base_url(base_url)
            .build()
            .expect("failed to build the LightconeClient under test");
        client
            .http()
            .user_session()
            .set_token("live-cookie".to_string())
            .await;
        client
    }

    async fn spawn_capturing_response_server(
        body: &'static str,
    ) -> (String, oneshot::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let (request_tx, request_rx) = oneshot::channel();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buffer = [0_u8; 4096];
            let read = socket.read(&mut buffer).await.unwrap();
            let request = String::from_utf8_lossy(&buffer[..read]).into_owned();
            let _ = request_tx.send(request);
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        });
        (format!("http://{}", addr), request_rx)
    }

    #[tokio::test]
    async fn logout_failure_propagates_after_clearing_local_state() {
        // The app's logout teardown gate reads this result to decide whether
        // the WebSocket may reconnect — a swallowed failure would let it
        // restart with a still-valid server-side cookie.
        let base_url = spawn_single_response_server(
            500,
            r#"{"status":"error","error_details":{"reason":"session store down"}}"#,
        )
        .await;
        let client = client_with_token(&base_url).await;

        let result = client.auth().logout().await;

        assert!(result.is_err());
        assert!(client.auth_token().await.is_none());
        assert!(client.auth().credentials().await.is_none());
    }

    #[tokio::test]
    async fn logout_401_counts_as_success() {
        // 401 means "already logged out" — the goal state, not a failure.
        let base_url = spawn_single_response_server(401, "Unauthorized").await;
        let client = client_with_token(&base_url).await;

        let result = client.auth().logout().await;

        assert!(result.is_ok());
        assert!(client.auth_token().await.is_none());
    }

    #[tokio::test]
    async fn update_max_slippage_preference_uses_exact_contract() {
        let (base_url, request) = spawn_capturing_response_server(
            r#"{"status":"success","body":{"max_slippage_preference":"5.50"}}"#,
        )
        .await;
        let client = LightconeClient::builder()
            .base_url(&base_url)
            .build()
            .unwrap();

        let persisted = client
            .auth()
            .update_max_slippage_preference(Decimal::new(550, 2))
            .await
            .unwrap();
        let request = request.await.unwrap();

        assert_eq!(persisted, Decimal::new(550, 2));
        assert!(request.starts_with("POST /api/auth/max_slippage_preference "));
        assert!(request.contains(r#"{"max_slippage_preference":"5.50"}"#));
    }

    /// Verifies Privy registration returns and installs the backend-refreshed session.
    #[tokio::test]
    async fn register_privy_returns_session_and_installs_refreshed_credentials() {
        let (base_url, request) = spawn_capturing_response_server(
            r#"{"status":"success","body":{"user":{"user_id":"user:test","identity":{"type":"email","account":{"email":"verified@example.com"},"privy":{"id":"did:privy:test","wallet":{"privy_id":"wallet:test","chain":"solana","address":"11111111111111111111111111111111"}}},"max_slippage_preference":null},"expires_at":2000000000,"auth_method":"privy","is_beta":false}}"#,
        )
        .await;
        let client = LightconeClient::builder()
            .base_url(&base_url)
            .build()
            .unwrap();
        let registration = crate::auth::RegisterPrivyRequest {
            attempted_identity: crate::auth::LinkedIdentitySelector::Email {
                email: "verified@example.com".to_string(),
            },
        };

        let session = client.auth().register_privy(&registration).await.unwrap();
        let request = request.await.unwrap();
        let credentials = client.auth().credentials().await.unwrap();

        assert_eq!(session.user.user_id, "user:test");
        assert_eq!(credentials.user_id, "user:test");
        assert_eq!(
            credentials.wallet_address.as_str(),
            "11111111111111111111111111111111"
        );
        assert!(request.starts_with("POST /api/auth/register-privy "));
        assert!(request.contains("verified@example.com"));
    }
}
