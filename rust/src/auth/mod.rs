#![doc = include_str!("README.md")]

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

/// How a session authenticated, as reported by the backend (derived from which
/// token verified the request).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    Privy,
    Lightcone,
}

/// A Privy-managed embedded wallet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrivyEmbeddedWallet {
    pub privy_id: String,
    pub chain: ChainType,
    pub address: String,
}

/// Privy account data attached to an identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPrivyData {
    /// The Privy DID (`did:privy:...`).
    pub id: String,
    /// Always present: Privy registration provisions the embedded wallet in
    /// the same transaction that creates the user.
    pub wallet: PrivyEmbeddedWallet,
}

/// X account data — the same shape whether X is the login identity or a
/// connected account on a Google/wallet identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct XAccountData {
    /// X numeric user id (Privy `subject`); absent on legacy rows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// Google account data for a Google login identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoogleAccountData {
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
}

/// The login identity — how the user authenticates.
/// Serializes as a tagged union: `{"type": "google", "account": ..., "privy": ...}`.
///
/// Privy data lives on the variant because its presence is determined by the
/// identity type: Google/X login only exists via Privy OAuth (guaranteed DID +
/// embedded wallet), while wallet users opt into Privy (SIWS) or stay
/// self-custody (RS256).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserIdentity {
    Google {
        account: GoogleAccountData,
        privy: UserPrivyData,
    },
    X {
        account: XAccountData,
        privy: UserPrivyData,
    },
    Wallet {
        address: String,
        chain: ChainType,
        #[serde(skip_serializing_if = "Option::is_none")]
        privy: Option<UserPrivyData>,
    },
}

impl UserIdentity {
    /// Human-readable login-method label ("Google" / "X" / "Solana"),
    /// e.g. for "Signed-in via {…}".
    pub fn text(&self) -> &'static str {
        match self {
            Self::Google { .. } => "Google",
            Self::X { .. } => "X",
            Self::Wallet { .. } => "Solana",
        }
    }
}

/// Full user profile — the `user` object of [`SessionResponse`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub user_id: String,
    pub identity: UserIdentity,
    /// X account connected by a non-X-identity user; `None` when identity is X.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected_x: Option<XAccountData>,
}

impl User {
    /// Privy account data, regardless of identity type.
    pub fn privy(&self) -> Option<&UserPrivyData> {
        match &self.identity {
            UserIdentity::Google { privy, .. } | UserIdentity::X { privy, .. } => Some(privy),
            UserIdentity::Wallet { privy, .. } => privy.as_ref(),
        }
    }

    /// The X account, whether it is the login identity or a connected account.
    pub fn x_account(&self) -> Option<&XAccountData> {
        match &self.identity {
            UserIdentity::X { account, .. } => Some(account),
            _ => self.connected_x.as_ref(),
        }
    }

    /// The wallet this session operates as.
    ///
    /// Google/X identities only exist via Privy registration, which always
    /// provisions an embedded wallet — that wallet is the answer regardless
    /// of auth method. Wallet identities depend on the session: a Privy
    /// (SIWS) session trades via the embedded wallet, a Lightcone (RS256)
    /// session trades via the wallet that signed in.
    pub fn trading_wallet(&self, auth_method: AuthMethod) -> &str {
        match &self.identity {
            UserIdentity::Google { privy, .. } | UserIdentity::X { privy, .. } => {
                &privy.wallet.address
            }
            UserIdentity::Wallet { address, privy, .. } => match auth_method {
                AuthMethod::Privy => privy
                    .as_ref()
                    .map(|privy_data| privy_data.wallet.address.as_str())
                    .unwrap_or(address),
                AuthMethod::Lightcone => address,
            },
        }
    }

    /// Short display label for the wallet this session operates as.
    pub fn wallet_display_name(&self, auth_method: AuthMethod) -> String {
        crate::shared::fmt::str::shorten(self.trading_wallet(auth_method), 8)
    }

    /// Best display name for the user. Google: `name`, falling back to the
    /// email; X: `display_name`, falling back to the username; wallet
    /// identities show the shortened address (`FRGk...WcPR`).
    pub fn display_name(&self) -> String {
        match &self.identity {
            UserIdentity::Google { account, .. } => account
                .name
                .clone()
                .unwrap_or_else(|| account.email.clone()),
            UserIdentity::X { account, .. } => account
                .display_name
                .clone()
                .unwrap_or_else(|| account.username.clone()),
            UserIdentity::Wallet { address, .. } => crate::shared::fmt::str::shorten(address, 8),
        }
    }

    /// Avatar URL from the login identity's OAuth provider, if any.
    pub fn avatar_url(&self) -> Option<&str> {
        match &self.identity {
            UserIdentity::Google { account, .. } => account.avatar_url.as_deref(),
            UserIdentity::X { account, .. } => account.avatar_url.as_deref(),
            UserIdentity::Wallet { .. } => None,
        }
    }
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
/// The `nonce` must be obtained from `GET /api/auth/nonce` first.
/// The caller signs this message externally (wallet adapter on WASM, keypair
/// on native) and passes the signature back to
/// `client.auth().login_with_message(...)`.
pub fn generate_signin_message(nonce: &str) -> Vec<u8> {
    let message = format!("Sign in to Lightcone\nNonce: {}", nonce);
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

/// Session envelope returned by `login_with_message`, `register-privy`, and
/// `GET /api/auth/me`: the durable user profile plus the session-scoped facts.
///
/// There is no `wallet_address` field — the session's trading wallet is
/// derived via [`User::trading_wallet`] from `user` + `auth_method`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionResponse {
    pub user: User,
    pub expires_at: i64,
    pub auth_method: AuthMethod,
    pub is_beta: bool,
}

/// Response from `GET /api/auth/nonce`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceResponse {
    pub nonce: String,
}

#[cfg(test)]
mod tests {
    use super::{
        AuthMethod, ChainType, GoogleAccountData, PrivyEmbeddedWallet, User, UserIdentity,
        UserPrivyData, XAccountData,
    };

    fn privy(address: &str) -> UserPrivyData {
        UserPrivyData {
            id: "did:privy:test".to_string(),
            wallet: PrivyEmbeddedWallet {
                privy_id: "wallet:test".to_string(),
                chain: ChainType::Solana,
                address: address.to_string(),
            },
        }
    }

    fn user(identity: UserIdentity) -> User {
        User {
            user_id: "user:test".to_string(),
            identity,
            connected_x: None,
        }
    }

    #[test]
    fn wallet_display_name_uses_the_session_trading_wallet() {
        let google_wallet = "FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR";
        let x_wallet = "So11111111111111111111111111111111111111112";
        let sign_in_wallet = "11111111111111111111111111111111";
        let embedded_wallet = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

        let google = user(UserIdentity::Google {
            account: GoogleAccountData {
                email: "user@example.com".to_string(),
                name: Some("Google User".to_string()),
                given_name: None,
                family_name: None,
                avatar_url: None,
            },
            privy: privy(google_wallet),
        });
        let x = user(UserIdentity::X {
            account: XAccountData {
                user_id: Some("123".to_string()),
                username: "x_user".to_string(),
                display_name: Some("X User".to_string()),
                avatar_url: None,
            },
            privy: privy(x_wallet),
        });
        let wallet = user(UserIdentity::Wallet {
            address: sign_in_wallet.to_string(),
            chain: ChainType::Solana,
            privy: Some(privy(embedded_wallet)),
        });
        let wallet_no_privy = user(UserIdentity::Wallet {
            address: sign_in_wallet.to_string(),
            chain: ChainType::Solana,
            privy: None,
        });

        assert_eq!(google.wallet_display_name(AuthMethod::Privy), "FRGk...WcPR");
        assert_eq!(x.wallet_display_name(AuthMethod::Privy), "So11...1112");
        assert_eq!(
            wallet.wallet_display_name(AuthMethod::Lightcone),
            "1111...1111"
        );
        assert_eq!(wallet.wallet_display_name(AuthMethod::Privy), "Toke...Q5DA");
        assert_eq!(
            wallet_no_privy.wallet_display_name(AuthMethod::Privy),
            "1111...1111"
        );
    }
}
