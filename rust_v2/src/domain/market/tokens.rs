//! Token types — deposit assets, conditional tokens, validation.
//!
//! Sub-entity of market. Wire types live in `super::wire`.

use super::wire::DepositAssetResponse;
use crate::shared::PubkeyStr;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ─── Token trait ─────────────────────────────────────────────────────────────

/// Common interface for all token types.
pub trait Token {
    fn id(&self) -> i32;
    fn pubkey(&self) -> &PubkeyStr;
    fn name(&self) -> &str;
    fn symbol(&self) -> &str;
    fn description(&self) -> &Option<String>;
    fn decimals(&self) -> u16;
    fn icon_url(&self) -> &str;
}

// ─── ConditionalToken ────────────────────────────────────────────────────────

/// A conditional token (outcome token) minted against a deposit asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConditionalToken {
    id: i32,
    pub outcome_index: i16,
    pub outcome: String,
    pub deposit_asset: PubkeyStr,
    pub deposit_symbol: String,
    mint: PubkeyStr,
    name: String,
    symbol: String,
    description: Option<String>,
    decimals: u16,
    icon_url: String,
}

impl Token for ConditionalToken {
    fn id(&self) -> i32 {
        self.id
    }
    fn pubkey(&self) -> &PubkeyStr {
        &self.mint
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn symbol(&self) -> &str {
        &self.symbol
    }
    fn description(&self) -> &Option<String> {
        &self.description
    }
    fn decimals(&self) -> u16 {
        self.decimals
    }
    fn icon_url(&self) -> &str {
        &self.icon_url
    }
}

// ─── DepositAsset ────────────────────────────────────────────────────────────

/// A deposit asset (collateral token, e.g. USDC).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositAsset {
    pub id: i32,
    pub market_pda: PubkeyStr,
    pub deposit_asset: PubkeyStr,
    pub num_outcomes: i16,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub decimals: u16,
    pub icon_url: String,
}

impl Token for DepositAsset {
    fn id(&self) -> i32 {
        self.id
    }
    fn pubkey(&self) -> &PubkeyStr {
        &self.deposit_asset
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn symbol(&self) -> &str {
        &self.symbol
    }
    fn description(&self) -> &Option<String> {
        &self.description
    }
    fn decimals(&self) -> u16 {
        self.decimals
    }
    fn icon_url(&self) -> &str {
        &self.icon_url
    }
}

// ─── TokenMetadata ───────────────────────────────────────────────────────────

/// Metadata for any token (deposit or conditional).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenMetadata {
    pub pubkey: PubkeyStr,
    pub symbol: String,
    pub decimals: u16,
    pub icon_url: String,
    pub name: String,
}

// ─── Stablecoin detection ────────────────────────────────────────────────────

pub const USDC_MAINNET: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const USDT_MAINNET: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
pub const USDC_DEVNET_LC: &str = "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez";

fn is_usd_stablecoin(pubkey: &PubkeyStr) -> bool {
    let s = pubkey.as_str();
    s == USDC_MAINNET || s == USDT_MAINNET || s == USDC_DEVNET_LC
}

impl ConditionalToken {
    pub fn is_usd_stable_coin(&self) -> bool {
        is_usd_stablecoin(&self.deposit_asset)
    }

    pub fn currency_symbol(&self) -> &'static str {
        if is_usd_stablecoin(self.pubkey()) {
            "$"
        } else {
            ""
        }
    }
}

impl DepositAsset {
    pub fn is_usd_stable_coin(&self) -> bool {
        is_usd_stablecoin(&self.deposit_asset)
    }

    pub fn currency_symbol(&self) -> &'static str {
        if is_usd_stablecoin(self.pubkey()) {
            "$"
        } else {
            ""
        }
    }
}

impl TokenMetadata {
    pub fn is_usd_stable_coin(&self) -> bool {
        is_usd_stablecoin(&self.pubkey)
    }

    pub fn currency_symbol(&self) -> &'static str {
        if is_usd_stablecoin(&self.pubkey) {
            "$"
        } else {
            ""
        }
    }
}

// ─── ValidatedTokens ─────────────────────────────────────────────────────────

/// Result of validating a `DepositAssetResponse`: the deposit asset, its conditional tokens,
/// and metadata for all.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidatedTokens {
    pub token: DepositAsset,
    pub conditionals: Vec<ConditionalToken>,
    pub metadata: HashMap<PubkeyStr, TokenMetadata>,
}

// ─── Validation ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum TokenValidationError {
    Multiple(String, Vec<TokenValidationError>),
    MissingDisplayName(String),
    MissingShortName(String),
    MissingDecimals(String),
    MissingSymbol(String),
    MissingOutcome(String),
    MissingIconUrl(String),
}

impl fmt::Display for TokenValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenValidationError::Multiple(mint, errors) => {
                writeln!(f, "Token validation errors ({mint}):")?;
                for err in errors {
                    writeln!(f, "  - {}", err)?;
                }
                Ok(())
            }
            TokenValidationError::MissingDisplayName(m) => write!(f, "Missing display name: {m}"),
            TokenValidationError::MissingShortName(m) => write!(f, "Missing short name: {m}"),
            TokenValidationError::MissingDecimals(m) => write!(f, "Missing decimals: {m}"),
            TokenValidationError::MissingSymbol(m) => write!(f, "Missing symbol: {m}"),
            TokenValidationError::MissingOutcome(m) => write!(f, "Missing outcome: {m}"),
            TokenValidationError::MissingIconUrl(m) => write!(f, "Missing icon URL: {m}"),
        }
    }
}

impl std::error::Error for TokenValidationError {}

// ─── Conversion: DepositAssetResponse → ValidatedTokens ─────────────────────

impl TryFrom<DepositAssetResponse> for ValidatedTokens {
    type Error = TokenValidationError;

    fn try_from(source: DepositAssetResponse) -> Result<Self, Self::Error> {
        let mut errors: Vec<TokenValidationError> = Vec::new();
        let mut metadata: HashMap<PubkeyStr, TokenMetadata> = HashMap::new();
        let mut conditionals: Vec<ConditionalToken> = Vec::new();
        let pubkey: PubkeyStr = source.deposit_asset.clone().into();

        let icon_url = source.icon_url.unwrap_or_else(|| {
            errors.push(TokenValidationError::MissingIconUrl(
                source.deposit_asset.clone(),
            ));
            String::new()
        });
        let name = source.display_name.unwrap_or_else(|| {
            errors.push(TokenValidationError::MissingDisplayName(
                source.deposit_asset.clone(),
            ));
            String::new()
        });
        let symbol = source.symbol.unwrap_or_else(|| {
            errors.push(TokenValidationError::MissingSymbol(
                source.deposit_asset.clone(),
            ));
            String::new()
        });
        let decimals = source.decimals.map(|i| i as u16).unwrap_or_else(|| {
            errors.push(TokenValidationError::MissingDecimals(
                source.deposit_asset.clone(),
            ));
            0
        });

        metadata.insert(
            pubkey.clone(),
            TokenMetadata {
                pubkey: pubkey.clone(),
                symbol: symbol.clone(),
                decimals,
                icon_url: icon_url.clone(),
                name: name.clone(),
            },
        );

        for ct in source.conditional_mints {
            let ct_pubkey = PubkeyStr::from(ct.token_address.clone());
            let mut ct_errors: Vec<TokenValidationError> = Vec::new();

            let ct_decimals = ct.decimals.map(|d| d as u16).unwrap_or_else(|| {
                ct_errors.push(TokenValidationError::MissingDecimals(
                    ct_pubkey.to_string(),
                ));
                0
            });
            let ct_symbol = ct.short_name.unwrap_or_else(|| {
                ct_errors.push(TokenValidationError::MissingShortName(
                    ct_pubkey.to_string(),
                ));
                String::new()
            });
            let ct_name = ct.display_name.unwrap_or_else(|| {
                ct_errors.push(TokenValidationError::MissingDisplayName(
                    ct_pubkey.to_string(),
                ));
                String::new()
            });
            let ct_outcome = ct.outcome.unwrap_or_else(|| {
                ct_errors.push(TokenValidationError::MissingOutcome(
                    ct_pubkey.to_string(),
                ));
                String::new()
            });

            if !ct_errors.is_empty() {
                errors.push(TokenValidationError::Multiple(
                    ct_pubkey.to_string(),
                    ct_errors,
                ));
                continue;
            }

            metadata.insert(
                ct_pubkey.clone(),
                TokenMetadata {
                    pubkey: ct_pubkey.clone(),
                    symbol: ct_symbol.clone(),
                    decimals: ct_decimals,
                    icon_url: icon_url.clone(),
                    name: ct_name.clone(),
                },
            );

            conditionals.push(ConditionalToken {
                id: ct.id,
                deposit_symbol: symbol.clone(),
                deposit_asset: pubkey.clone(),
                outcome_index: ct.outcome_index,
                icon_url: icon_url.clone(),
                description: ct.description,
                outcome: ct_outcome,
                mint: ct_pubkey,
                name: ct_name,
                symbol: ct_symbol,
                decimals: ct_decimals,
            });
        }

        if !errors.is_empty() {
            return Err(TokenValidationError::Multiple(
                source.deposit_asset,
                errors,
            ));
        }

        Ok(Self {
            token: DepositAsset {
                deposit_asset: pubkey,
                id: source.id,
                market_pda: source.market_pubkey.into(),
                num_outcomes: source.num_outcomes,
                name,
                symbol,
                description: source.description,
                decimals,
                icon_url,
            },
            conditionals,
            metadata,
        })
    }
}
