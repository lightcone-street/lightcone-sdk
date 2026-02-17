//! Auth sub-client — login, logout, credential management.

use crate::auth::{AuthCredentials, LoginRequest, LoginResponse};
use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::shared::PubkeyStr;

/// Sub-client for authentication operations.
pub struct Auth<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Auth<'a> {
    /// Login with a pre-signed message.
    ///
    /// - On native: stores the token internally for header injection.
    /// - On WASM: the backend sets an HTTP-only cookie; the SDK never touches the token.
    pub async fn login_with_message(
        &self,
        message: &str,
        signature: &str,
        pubkey: &str,
    ) -> Result<AuthCredentials, SdkError> {
        let request = LoginRequest {
            message: message.to_string(),
            signature: signature.to_string(),
            pubkey: pubkey.to_string(),
        };

        let resp: serde_json::Value = self.client.http.login(&request).await?;

        let login_resp: LoginResponse = serde_json::from_value(resp)?;

        // On native, store the token internally (never exposed)
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(ref token) = login_resp.token {
            self.client.http.set_auth_token(Some(token.clone())).await;
        }

        let credentials = AuthCredentials {
            user_id: login_resp.user_id,
            wallet_address: PubkeyStr::from(pubkey),
            expires_at: None,
        };

        *self.client.auth_credentials.write().await = Some(credentials.clone());
        Ok(credentials)
    }

    /// Logout — clears server-side cookie + internal token + all caches.
    pub async fn logout(&self) -> Result<(), SdkError> {
        // Call backend to clear HTTP-only cookie
        let _ = self.client.http.logout().await;

        // Clear internal token (native)
        #[cfg(not(target_arch = "wasm32"))]
        self.client.http.clear_auth_token().await;

        // Clear auth credentials
        *self.client.auth_credentials.write().await = None;

        // Clear all HTTP caches
        self.client.clear_all_caches().await;

        Ok(())
    }

    /// Get current auth credentials (if authenticated).
    pub async fn credentials(&self) -> Option<AuthCredentials> {
        self.client.auth_credentials.read().await.clone()
    }

    /// Check if currently authenticated.
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
