//! Authentication module for Lightcone.
//!
//! Provides functionality for authenticating with the Lightcone API
//! to obtain JWT tokens for private endpoints (REST API, WebSocket user streams).
//!
//! # Authentication Flow
//!
//! 1. Generate a sign-in message with timestamp
//! 2. Sign the message with an Ed25519 keypair
//! 3. POST to the authentication endpoint
//! 4. Extract token from JSON response
//! 5. Use the token with API client or WebSocket connection

use std::time::{SystemTime, UNIX_EPOCH};

use thiserror::Error;

/// Authentication-specific errors
#[derive(Debug, Clone, Error)]
pub enum AuthError {
    /// System time error (before UNIX epoch)
    #[error("System time error: {0}")]
    SystemTime(String),

    /// HTTP request failed
    #[error("HTTP request error: {0}")]
    HttpError(String),

    /// Authentication was rejected by the server
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
}

/// Result type alias for authentication operations
pub type AuthResult<T> = Result<T, AuthError>;

/// Authentication credentials returned after successful login
#[derive(Debug, Clone)]
pub struct AuthCredentials {
    /// The authentication token (JWT) for authenticated requests
    pub auth_token: String,
    /// The user's public key (Base58 encoded)
    pub user_pubkey: String,
    /// The user's ID
    pub user_id: String,
    /// Token expiration timestamp (Unix seconds)
    pub expires_at: i64,
}

/// Generate the sign-in message with current timestamp.
///
/// # Returns
///
/// The message to be signed, in the format:
/// ```text
/// Sign in to Lightcone
///
/// Timestamp: {unix_ms}
/// ```
///
/// # Errors
///
/// Returns an error if the system time is before the UNIX epoch.
pub fn generate_signin_message() -> AuthResult<String> {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AuthError::SystemTime("System time before UNIX epoch".to_string()))?
        .as_millis();

    Ok(format!(
        "Sign in to Lightcone\n\nTimestamp: {}",
        timestamp_ms
    ))
}

/// Generate the sign-in message with a specific timestamp.
///
/// # Arguments
///
/// * `timestamp_ms` - Unix timestamp in milliseconds
///
/// # Returns
///
/// The message to be signed
pub fn generate_signin_message_with_timestamp(timestamp_ms: u128) -> String {
    format!("Sign in to Lightcone\n\nTimestamp: {}", timestamp_ms)
}

// ============================================================================
// Feature-gated authentication (requires network + signing)
// ============================================================================

#[cfg(feature = "auth")]
mod login {
    use std::time::Duration;

    use super::*;
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use solana_keypair::Keypair;
    use solana_signer::Signer;

    /// Authentication request timeout
    const AUTH_TIMEOUT: Duration = Duration::from_secs(10);

    /// Request body for login endpoint
    #[derive(Debug, Serialize)]
    struct LoginRequest {
        /// Raw 32-byte public key
        pubkey_bytes: Vec<u8>,
        /// The message that was signed
        message: String,
        /// Base58 encoded signature
        signature_bs58: String,
    }

    /// Response from login endpoint
    #[derive(Debug, Deserialize)]
    struct LoginResponse {
        /// The authentication token
        token: String,
        /// The user's ID
        user_id: String,
        /// Token expiration timestamp (Unix seconds)
        expires_at: i64,
    }

    /// Authenticate with Lightcone and obtain credentials.
    ///
    /// # Arguments
    ///
    /// * `keypair` - The Ed25519 signing key to use for authentication
    ///
    /// # Returns
    ///
    /// `AuthCredentials` containing the auth token and user public key
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solana_keypair::Keypair;
    /// use lightcone_sdk::auth::authenticate;
    ///
    /// let keypair = Keypair::from_bytes(&keypair_bytes).unwrap();
    /// let credentials = authenticate(&keypair).await?;
    /// println!("Auth token: {}", credentials.auth_token);
    /// ```
    pub async fn authenticate(keypair: &Keypair) -> AuthResult<AuthCredentials> {
        // Generate the message
        let message = generate_signin_message()?;

        // Sign the message
        let signature = keypair.sign_message(message.as_bytes());
        let signature_b58 = bs58::encode(signature.as_ref()).into_string();

        // Get the public key
        let public_key = keypair.pubkey();
        let public_key_b58 = public_key.to_string();

        // Create the request body
        let request = LoginRequest {
            pubkey_bytes: public_key.as_ref().to_vec(),
            message,
            signature_bs58: signature_b58,
        };

        // Create client with timeout
        let client = Client::builder()
            .timeout(AUTH_TIMEOUT)
            .build()
            .map_err(|e| AuthError::HttpError(e.to_string()))?;

        // Send the authentication request
        let url = format!("{}/auth/login_or_register_with_message", crate::network::DEFAULT_API_URL);
        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AuthError::HttpError(e.to_string()))?;

        // Check for HTTP errors
        if !response.status().is_success() {
            return Err(AuthError::AuthenticationFailed(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        // Parse the response body
        let login_response: LoginResponse = response.json().await.map_err(|e| {
            AuthError::AuthenticationFailed(format!("Failed to parse response: {}", e))
        })?;

        Ok(AuthCredentials {
            auth_token: login_response.token,
            user_pubkey: public_key_b58,
            user_id: login_response.user_id,
            expires_at: login_response.expires_at,
        })
    }

    /// Request body for transaction-based login endpoint
    #[derive(Debug, Serialize)]
    struct TransactionLoginRequest {
        /// Raw transaction message bytes
        message_bytes: Vec<u8>,
        /// Signature bytes (64 bytes)
        signature_bytes: Vec<u8>,
        /// Raw 32-byte public key
        pubkey_bytes: Vec<u8>,
    }

    /// Authenticate with Lightcone using a signed transaction.
    ///
    /// This is for wallets that don't support message signing.
    ///
    /// # Arguments
    ///
    /// * `message_bytes` - The raw transaction message bytes that were signed
    /// * `signature_bytes` - The Ed25519 signature (64 bytes)
    /// * `pubkey_bytes` - The public key (32 bytes)
    pub async fn authenticate_with_transaction(
        message_bytes: &[u8],
        signature_bytes: &[u8; 64],
        pubkey_bytes: &[u8; 32],
    ) -> AuthResult<AuthCredentials> {
        let public_key_b58 = bs58::encode(pubkey_bytes).into_string();

        let request = TransactionLoginRequest {
            message_bytes: message_bytes.to_vec(),
            signature_bytes: signature_bytes.to_vec(),
            pubkey_bytes: pubkey_bytes.to_vec(),
        };

        let client = Client::builder()
            .timeout(AUTH_TIMEOUT)
            .build()
            .map_err(|e| AuthError::HttpError(e.to_string()))?;

        let url = format!(
            "{}/auth/login_or_register_with_transaction",
            crate::network::DEFAULT_API_URL
        );
        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AuthError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::AuthenticationFailed(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let login_response: LoginResponse = response.json().await.map_err(|e| {
            AuthError::AuthenticationFailed(format!("Failed to parse response: {}", e))
        })?;

        Ok(AuthCredentials {
            auth_token: login_response.token,
            user_pubkey: public_key_b58,
            user_id: login_response.user_id,
            expires_at: login_response.expires_at,
        })
    }
}

#[cfg(feature = "auth")]
pub use login::authenticate;
#[cfg(feature = "auth")]
pub use login::authenticate_with_transaction;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_signin_message() {
        let message = generate_signin_message().unwrap();
        assert!(message.starts_with("Sign in to Lightcone\n\nTimestamp: "));
    }

    #[test]
    fn test_generate_signin_message_with_timestamp() {
        let timestamp = 1234567890123u128;
        let message = generate_signin_message_with_timestamp(timestamp);
        assert_eq!(message, "Sign in to Lightcone\n\nTimestamp: 1234567890123");
    }
}
