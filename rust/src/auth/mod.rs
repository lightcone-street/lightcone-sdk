#![doc = include_str!("README.md")]

#[cfg(feature = "http")]
pub mod client;

#[cfg(feature = "native-auth")]
pub mod native;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
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

/// Email account data for a passwordless Email login identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmailAccountData {
    pub email: String,
}

pub use crate::shared::LinkedIdentityType;

/// Bounded ownership conflict returned by register-or-sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterPrivyConflict {
    IdentityOwned {
        existing_method: Option<LinkedIdentityType>,
    },
    MultipleAccounts,
    WalletOwned {
        existing_method: Option<LinkedIdentityType>,
    },
}

/// Classifies only stable register-or-sync ownership rejection codes.
pub fn classify_register_privy_conflict(
    error: &crate::error::SdkError,
) -> Option<RegisterPrivyConflict> {
    let crate::error::SdkError::ApiRejected(details) = error else {
        return None;
    };
    match details.error_code.as_deref()? {
        "IDENTITY_OWNED_BY_ANOTHER_ACCOUNT" => Some(RegisterPrivyConflict::IdentityOwned {
            existing_method: details.existing_method,
        }),
        "IDENTITIES_OWNED_BY_MULTIPLE_ACCOUNTS" => Some(RegisterPrivyConflict::MultipleAccounts),
        "WALLET_OWNED_BY_ANOTHER_ACCOUNT" => Some(RegisterPrivyConflict::WalletOwned {
            existing_method: details.existing_method,
        }),
        _ => None,
    }
}

/// A verified login identity connected to an Account, without repeated Privy data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LinkedIdentity {
    Email { account: EmailAccountData },
    Google { account: GoogleAccountData },
    X { account: XAccountData },
    Wallet { address: String, chain: ChainType },
}

impl LinkedIdentity {
    /// Returns the public login-method tag for this Connected Login Identity.
    pub fn identity_type(&self) -> LinkedIdentityType {
        match self {
            Self::Email { .. } => LinkedIdentityType::Email,
            Self::Google { .. } => LinkedIdentityType::Google,
            Self::X { .. } => LinkedIdentityType::X,
            Self::Wallet { .. } => LinkedIdentityType::Wallet,
        }
    }
}

/// The login identity that initiated an interactive Privy authentication.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LinkedIdentitySelector {
    Email { email: String },
    Google { email: String },
    X { username: String },
    Wallet { address: String, chain: ChainType },
}

/// Register-or-sync request naming the verified attempted identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegisterPrivyRequest {
    /// Method and canonical identifier that initiated interactive authentication.
    pub attempted_identity: LinkedIdentitySelector,
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
    Email {
        account: EmailAccountData,
        privy: UserPrivyData,
    },
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
            Self::Email { .. } => "Email",
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
    /// Every verified login identity connected to the Account, primary first.
    #[serde(default)]
    pub linked_identities: Vec<LinkedIdentity>,
    /// Remembered account-wide percentage strictly below 10%. `None` means no
    /// such preference is stored; clients decide their own display fallback.
    /// Missing values from an older backend are normalized to `None` during a
    /// rolling deployment; present values must still be an exact string or null.
    #[serde(
        default,
        deserialize_with = "crate::shared::serde_util::deserialize_required_nullable_decimal"
    )]
    pub max_slippage_preference: Option<Decimal>,
    /// X account connected by a non-X-identity user; `None` when identity is X.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connected_x: Option<XAccountData>,
}

/// Keeps Email labels compact while preserving the recognizable address ends.
fn email_display_name(email: &str) -> String {
    const MAX_CHARS: usize = 20;
    const ELLIPSIS_CHARS: usize = 3;

    let char_count = email.chars().count();
    if char_count <= MAX_CHARS {
        return email.to_string();
    }

    let visible_chars = MAX_CHARS - ELLIPSIS_CHARS;
    let prefix_chars = visible_chars / 2;
    let suffix_chars = visible_chars - prefix_chars;
    let prefix = email.chars().take(prefix_chars).collect::<String>();
    let suffix = email
        .chars()
        .skip(char_count - suffix_chars)
        .collect::<String>();
    format!("{prefix}...{suffix}")
}

impl User {
    /// Privy account data, regardless of identity type.
    pub fn privy(&self) -> Option<&UserPrivyData> {
        match &self.identity {
            UserIdentity::Email { privy, .. }
            | UserIdentity::Google { privy, .. }
            | UserIdentity::X { privy, .. } => Some(privy),
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
            UserIdentity::Email { privy, .. }
            | UserIdentity::Google { privy, .. }
            | UserIdentity::X { privy, .. } => &privy.wallet.address,
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

    /// Best display name for the user. Email addresses are limited to 20
    /// characters; Google uses `name` with an email fallback; X uses
    /// `display_name` with a username fallback; wallet identities show the
    /// shortened address (`FRGk...WcPR`).
    pub fn display_name(&self) -> String {
        match &self.identity {
            UserIdentity::Email { account, .. } => email_display_name(&account.email),
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
            UserIdentity::Email { .. } => None,
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

/// Exact decimal-string body used to update and return max slippage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MaxSlippagePreferenceBody {
    pub max_slippage_preference: Decimal,
}

#[cfg(test)]
mod tests {
    use super::{
        classify_register_privy_conflict, AuthMethod, ChainType, EmailAccountData,
        GoogleAccountData, LinkedIdentity, LinkedIdentitySelector, LinkedIdentityType,
        PrivyEmbeddedWallet, RegisterPrivyConflict, RegisterPrivyRequest, User, UserIdentity,
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
            linked_identities: Vec::new(),
            max_slippage_preference: None,
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

    #[test]
    fn email_identity_and_linked_methods_round_trip() {
        let email = user(UserIdentity::Email {
            account: EmailAccountData {
                email: "verified@example.com".to_string(),
            },
            privy: privy("FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR"),
        });
        let mut email = email;
        email.linked_identities = vec![
            LinkedIdentity::Email {
                account: EmailAccountData {
                    email: "verified@example.com".to_string(),
                },
            },
            LinkedIdentity::Google {
                account: GoogleAccountData {
                    email: "verified@example.com".to_string(),
                    name: None,
                    given_name: None,
                    family_name: None,
                    avatar_url: None,
                },
            },
        ];

        let encoded = serde_json::to_value(&email).unwrap();
        let decoded: User = serde_json::from_value(encoded).unwrap();
        assert_eq!(decoded, email);
        assert_eq!(decoded.display_name(), "verified@example.com");
        assert_eq!(decoded.identity.text(), "Email");
    }

    #[test]
    fn email_display_name_is_limited_to_twenty_characters() {
        let email = user(UserIdentity::Email {
            account: EmailAccountData {
                email: "lightconewebtesting@gmail.com".to_string(),
            },
            privy: privy("FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR"),
        });

        assert_eq!(email.display_name(), "lightcon...gmail.com");
        assert_eq!(email.display_name().chars().count(), 20);
    }

    #[test]
    fn register_privy_request_uses_tagged_selector() {
        let request = RegisterPrivyRequest {
            attempted_identity: LinkedIdentitySelector::Email {
                email: "verified@example.com".to_string(),
            },
        };
        assert_eq!(
            serde_json::to_value(request).unwrap(),
            serde_json::json!({
                "attempted_identity": {
                    "type": "email",
                    "email": "verified@example.com"
                }
            })
        );
    }

    #[test]
    fn register_privy_conflicts_require_exact_codes_and_typed_methods() {
        let error = crate::error::SdkError::ApiRejected(crate::shared::ApiRejectedDetails {
            reason: "Identity belongs to another account".to_string(),
            rejection_code: None,
            error_code: Some("IDENTITY_OWNED_BY_ANOTHER_ACCOUNT".to_string()),
            existing_method: Some(LinkedIdentityType::Google),
            error_log_id: None,
            request_id: None,
            http_status: Some(409),
        });

        assert_eq!(
            classify_register_privy_conflict(&error),
            Some(RegisterPrivyConflict::IdentityOwned {
                existing_method: Some(LinkedIdentityType::Google),
            })
        );

        let unrelated = crate::error::SdkError::ApiRejected(crate::shared::ApiRejectedDetails {
            reason: "Conflict".to_string(),
            rejection_code: None,
            error_code: Some("RESOURCE_CONFLICT".to_string()),
            existing_method: Some(LinkedIdentityType::Email),
            error_log_id: None,
            request_id: None,
            http_status: Some(409),
        });
        assert_eq!(classify_register_privy_conflict(&unrelated), None);
    }

    #[test]
    fn max_slippage_preference_deserializes_null_or_exact_string() {
        let null_user: User = serde_json::from_value(serde_json::json!({
            "user_id": "user:test",
            "identity": {
                "type": "wallet",
                "address": "11111111111111111111111111111111",
                "chain": "solana"
            },
            "max_slippage_preference": null
        }))
        .unwrap();
        assert_eq!(null_user.max_slippage_preference, None);

        let stored_user: User = serde_json::from_value(serde_json::json!({
            "user_id": "user:test",
            "identity": {
                "type": "wallet",
                "address": "11111111111111111111111111111111",
                "chain": "solana"
            },
            "max_slippage_preference": "5.50"
        }))
        .unwrap();
        assert_eq!(
            stored_user.max_slippage_preference,
            Some(rust_decimal::Decimal::new(550, 2))
        );

        let missing: User = serde_json::from_value(serde_json::json!({
            "user_id": "user:test",
            "identity": {
                "type": "wallet",
                "address": "11111111111111111111111111111111",
                "chain": "solana"
            }
        }))
        .unwrap();
        assert_eq!(missing.max_slippage_preference, None);

        let numeric = serde_json::json!({
            "user_id": "user:test",
            "identity": {
                "type": "wallet",
                "address": "11111111111111111111111111111111",
                "chain": "solana"
            },
            "max_slippage_preference": 10
        });
        assert!(serde_json::from_value::<User>(numeric).is_err());
    }
}
