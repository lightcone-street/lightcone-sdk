//! Authentication module for Lightcone WebSocket.
//!
//! Provides functionality for authenticating with the Lightcone API
//! to access private user streams (orders, balances, fills).
//!
//! # Authentication Flow
//!
//! 1. Generate a sign-in message with timestamp
//! 2. Sign the message with an Ed25519 keypair
//! 3. POST to the authentication endpoint
//! 4. Extract `auth_token` from response cookie
//! 5. Connect to WebSocket with the auth token

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ed25519_dalek::{Signer, SigningKey};
use reqwest::cookie::Jar;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::websocket::error::{WebSocketError, WsResult};

/// Authentication API base URL
pub const AUTH_API_URL: &str = "https://lightcone.xyz/api";

/// Authentication request timeout
const AUTH_TIMEOUT: Duration = Duration::from_secs(10);

/// Authentication credentials returned after successful login
#[derive(Debug, Clone)]
pub struct AuthCredentials {
    /// The authentication token to use for WebSocket connection
    pub auth_token: String,
    /// The user's public key (Base58 encoded)
    pub user_pubkey: String,
}

/// Request body for login endpoint
#[derive(Debug, Serialize)]
struct LoginRequest {
    /// Base58 encoded public key
    public_key: String,
    /// The message that was signed
    message: String,
    /// Base58 encoded signature
    signature: String,
}

/// Response from login endpoint
#[derive(Debug, Deserialize)]
struct LoginResponse {
    /// Whether the login was successful
    success: bool,
    /// Error message if login failed
    #[serde(default)]
    error: Option<String>,
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
pub fn generate_signin_message() -> WsResult<String> {
    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| WebSocketError::Protocol("System time before UNIX epoch".to_string()))?
        .as_millis();

    Ok(format!("Sign in to Lightcone\n\nTimestamp: {}", timestamp_ms))
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

/// Authenticate with Lightcone and obtain credentials.
///
/// # Arguments
///
/// * `signing_key` - The Ed25519 signing key to use for authentication
///
/// # Returns
///
/// `AuthCredentials` containing the auth token and user public key
///
/// # Example
///
/// ```ignore
/// use ed25519_dalek::SigningKey;
/// use lightcone_sdk::websocket::auth::authenticate;
///
/// let signing_key = SigningKey::from_bytes(&secret_key_bytes);
/// let credentials = authenticate(&signing_key).await?;
/// println!("Auth token: {}", credentials.auth_token);
/// ```
pub async fn authenticate(signing_key: &SigningKey) -> WsResult<AuthCredentials> {
    // Generate the message
    let message = generate_signin_message()?;

    // Sign the message
    let signature = signing_key.sign(message.as_bytes());
    let signature_b58 = bs58::encode(signature.to_bytes()).into_string();

    // Get the public key
    let public_key = signing_key.verifying_key();
    let public_key_b58 = bs58::encode(public_key.to_bytes()).into_string();

    // Create the request body
    let request = LoginRequest {
        public_key: public_key_b58.clone(),
        message,
        signature: signature_b58,
    };

    // Create client with cookie jar to capture auth_token and timeout
    let jar = std::sync::Arc::new(Jar::default());
    let client = Client::builder()
        .cookie_provider(jar.clone())
        .timeout(AUTH_TIMEOUT)
        .build()
        .map_err(|e| WebSocketError::HttpError(e.to_string()))?;

    // Send the authentication request
    let url = format!("{}/auth/login_or_register_with_message", AUTH_API_URL);
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .map_err(|e| WebSocketError::HttpError(e.to_string()))?;

    // Check for HTTP errors
    if !response.status().is_success() {
        return Err(WebSocketError::AuthenticationFailed(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    // Extract auth_token from cookies
    let auth_token = response
        .cookies()
        .find(|c| c.name() == "auth_token")
        .map(|c| c.value().to_string())
        .ok_or_else(|| {
            WebSocketError::AuthenticationFailed("No auth_token cookie in response".to_string())
        })?;

    // Parse the response body
    let login_response: LoginResponse = response.json().await.map_err(|e| {
        WebSocketError::AuthenticationFailed(format!("Failed to parse response: {}", e))
    })?;

    if !login_response.success {
        return Err(WebSocketError::AuthenticationFailed(
            login_response
                .error
                .unwrap_or_else(|| "Unknown error".to_string()),
        ));
    }

    Ok(AuthCredentials {
        auth_token,
        user_pubkey: public_key_b58,
    })
}

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
