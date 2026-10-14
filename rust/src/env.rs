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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LightconeEnv {
    /// Local development environment.
    Local,
    /// Staging / test environment.
    Staging,
    /// Production environment (default).
    #[default]
    Prod,
}

impl LightconeEnv {
    /// REST API base URL for this environment.
    ///
    /// If the `SDK_API_URL` environment variable is set, its value is used
    /// regardless of the selected environment.
    pub fn api_url(&self) -> String {
        if let Ok(override_url) = std::env::var("SDK_API_URL") {
            return override_url;
        }
        match self {
            Self::Local => "https://api.local.internalcone.com",
            Self::Staging => "https://api.staging.internalcone.com",
            Self::Prod => "https://api.lightcone.xyz",
        }
        .to_string()
    }

    /// WebSocket URL for this environment.
    ///
    /// If the `SDK_WS_URL` environment variable is set, its value is used
    /// regardless of the selected environment.
    pub fn ws_url(&self) -> String {
        if let Ok(override_url) = std::env::var("SDK_WS_URL") {
            return override_url;
        }
        match self {
            Self::Local => "wss://ws.local.internalcone.com/ws",
            Self::Staging => "wss://ws.staging.internalcone.com/ws",
            Self::Prod => "wss://ws.lightcone.xyz/ws",
        }
        .to_string()
    }

    /// Solana RPC URL for this environment.
    ///
    /// If the `SDK_RPC_URL` environment variable is set, its value is used
    /// regardless of the selected environment.
    pub fn rpc_url(&self) -> String {
        if let Ok(override_url) = std::env::var("SDK_RPC_URL") {
            return override_url;
        }
        match self {
            Self::Local => "https://api.devnet.solana.com",
            Self::Staging => "https://api.devnet.solana.com",
            Self::Prod => "https://api.mainnet-beta.solana.com",
        }
        .to_string()
    }

    /// On-chain Lightcone program ID for this environment.
    ///
    /// If the `SDK_PROGRAM_ID` environment variable is set, its value is used
    /// regardless of the selected environment.
    pub fn program_id(&self) -> Pubkey {
        if let Ok(override_id) = std::env::var("SDK_PROGRAM_ID") {
            return Pubkey::from_str(&override_id).expect("SDK_PROGRAM_ID must be a valid pubkey");
        }
        self.default_program_id()
    }

    /// The built-in program ID for this environment, ignoring the
    /// `SDK_PROGRAM_ID` override.
    fn default_program_id(&self) -> Pubkey {
        match self {
            Self::Local => Pubkey::from_str("HQZW84F7WbpDLDdd6eaDsBh6LjDQ2uCxpkZgkLakcago")
                .expect("valid program id"),
            Self::Staging => Pubkey::from_str("5G2fWZGHB5BA8gbABVBuR1bU4Ziri9cRxFoojz5C5Rxk")
                .expect("valid program id"),
            Self::Prod => Pubkey::from_str("F5zrhPGL9REfsQ4vHR3ESYo9zaqiVvmHoHjqRwd8cPrz")
                .expect("valid program id"),
        }
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
    fn environment_defaults_target_expected_backends() -> Result<(), Box<dyn std::error::Error>> {
        // Isolate overrides in a child rather than mutate the parallel test process.
        if std::env::var_os("SDK_API_URL").is_some() || std::env::var_os("SDK_WS_URL").is_some() {
            let output = std::process::Command::new(std::env::current_exe()?)
                .args([
                    "--exact",
                    "env::tests::environment_defaults_target_expected_backends",
                ])
                .env_remove("SDK_API_URL")
                .env_remove("SDK_WS_URL")
                .output()?;
            assert!(
                output.status.success(),
                "isolated environment defaults test failed"
            );
            assert!(
                String::from_utf8(output.stdout)?.contains("test result: ok. 1 passed;"),
                "isolated environment defaults test did not run exactly one test"
            );
            return Ok(());
        }

        for (environment, api_url, ws_url) in [
            (
                LightconeEnv::Local,
                "https://api.local.internalcone.com",
                "wss://ws.local.internalcone.com/ws",
            ),
            (
                LightconeEnv::Staging,
                "https://api.staging.internalcone.com",
                "wss://ws.staging.internalcone.com/ws",
            ),
            (
                LightconeEnv::Prod,
                "https://api.lightcone.xyz",
                "wss://ws.lightcone.xyz/ws",
            ),
        ] {
            assert_eq!(environment.api_url(), api_url, "{environment} API URL");
            assert_eq!(environment.ws_url(), ws_url, "{environment} WebSocket URL");
        }
        Ok(())
    }

    #[test]
    fn local_environment_uses_deployed_program() {
        // default_program_id, not program_id: the latter honors the
        // SDK_PROGRAM_ID env override and would make this test depend on
        // the ambient shell environment.
        let program_id = LightconeEnv::Local.default_program_id();
        let (exchange_pda, _) = crate::program::pda::get_exchange_pda(&program_id);

        assert_eq!(
            program_id,
            Pubkey::from_str("HQZW84F7WbpDLDdd6eaDsBh6LjDQ2uCxpkZgkLakcago").unwrap()
        );
        assert_eq!(
            exchange_pda,
            Pubkey::from_str("B6Y3DF25exUTk2j7ocjYfoBY6r3tc6shyXTVhnHhrbk9").unwrap()
        );
    }
}
