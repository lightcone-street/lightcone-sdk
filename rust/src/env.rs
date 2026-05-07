//! Environment configuration for the Lightcone SDK.
//!
//! The [`LightconeEnv`] enum determines which Lightcone deployment the SDK
//! connects to. Each variant maps to a specific API URL, WebSocket URL,
//! Solana RPC URL, and on-chain program ID.

use solana_pubkey::Pubkey;
use std::fmt;
use std::str::FromStr;

/// Lightcone deployment environment.
///
/// Pass to [`LightconeClientBuilder::env`](crate::client::LightconeClientBuilder::env)
/// to configure the client for a specific deployment. Defaults to [`Prod`](LightconeEnv::Prod)
/// when not specified.
///
/// # Example
///
/// ```rust
/// use lightcone::prelude::*;
///
/// let client = LightconeClient::builder()
///     .env(LightconeEnv::Staging)
///     .build()?;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightconeEnv {
    /// Local development environment.
    Local,
    /// Staging / test environment.
    Staging,
    /// Production environment (default).
    Prod,
}

impl LightconeEnv {
    /// REST API base URL for this environment.
    pub fn api_url(&self) -> &'static str {
        match self {
            Self::Local => "http://localhost:8080/api",
            Self::Staging => "https://tapi2.lightcone.xyz",
            Self::Prod => "https://tapi.lightcone.xyz",
        }
    }

    /// WebSocket URL for this environment.
    pub fn ws_url(&self) -> &'static str {
        match self {
            Self::Local => "ws://localhost:8081/ws",
            Self::Staging => "wss://tws2.lightcone.xyz/ws",
            Self::Prod => "wss://tws.lightcone.xyz/ws",
        }
    }

    /// Solana RPC URL for this environment.
    pub fn rpc_url(&self) -> &'static str {
        match self {
            Self::Local => "https://api.devnet.solana.com",
            Self::Staging => "https://api.devnet.solana.com",
            Self::Prod => "https://api.devnet.solana.com",
        }
    }

    /// On-chain Lightcone program ID for this environment.
    pub fn program_id(&self) -> Pubkey {
        match self {
            Self::Local => Pubkey::from_str("AU4htPS3tSXA1JFrtA37oPGBLp2yCoi6VH4uF1jdouLK")
                .expect("valid program id"),
            Self::Staging => Pubkey::from_str("AZ8bEUuk8ifpw5EncZqHxiNJauikZtvtbuXdvwxYPfNT")
                .expect("valid program id"),
            Self::Prod => Pubkey::from_str("8nzsoyHZFYig3uN3M717Q47MtLqzx2V2UAKaPTqDy5rV")
                .expect("valid program id"),
        }
    }
}

impl Default for LightconeEnv {
    fn default() -> Self {
        Self::Prod
    }
}

impl fmt::Display for LightconeEnv {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Local => write!(formatter, "local"),
            Self::Staging => write!(formatter, "staging"),
            Self::Prod => write!(formatter, "prod"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_environment_targets_local_backend() {
        let environment = LightconeEnv::Local;

        assert_eq!(environment.api_url(), "http://localhost:8080/api");
        assert_eq!(environment.ws_url(), "ws://localhost:8081/ws");
    }

    #[test]
    fn local_environment_uses_deployed_program() {
        let program_id = LightconeEnv::Local.program_id();
        let (exchange_pda, _) = crate::program::pda::get_exchange_pda(&program_id);

        assert_eq!(
            program_id,
            Pubkey::from_str("AU4htPS3tSXA1JFrtA37oPGBLp2yCoi6VH4uF1jdouLK").unwrap()
        );
        assert_eq!(
            exchange_pda,
            Pubkey::from_str("DWuNmXKorbQxZV7ifCyuVp9jhDNh7YjS2gQgvYSCciue").unwrap()
        );
    }
}
