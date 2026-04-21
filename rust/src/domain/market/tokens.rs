//! Token types — deposit assets, conditional tokens, validation.
//!
//! Sub-entity of market. Wire types live in `super::wire`.

use super::wire::{DepositAssetResponse, GlobalDepositAssetResponse};
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

    /// Display priority for sorting: lower values come first. BTC/WBTC tie
    /// at 0, ETH/WETH tie at 1, SOL at 2; everything else falls to the
    /// alphabetical tail.
    fn display_priority(&self) -> u8 {
        match self.symbol() {
            "BTC" | "WBTC" => 0,
            "ETH" | "WETH" => 1,
            "SOL" => 2,
            _ => u8::MAX,
        }
    }
}

/// Types that expose a `Token` reference for display-priority sorting.
///
/// Blanket-implemented for every `T: Token`, so any token type sorts directly.
/// Composite types (e.g. `DepositAssetPair`, `OrderBookPair`) can implement
/// this by returning their `base` token, making them sortable with the same
/// `sort_by_display_priority` helper.
pub trait HasDisplayToken {
    fn display_token(&self) -> &dyn Token;
}

impl<T: Token> HasDisplayToken for T {
    fn display_token(&self) -> &dyn Token {
        self
    }
}

/// Returns a new `Vec` ordered for display: priority groups first
/// (BTC/WBTC → ETH/WETH → SOL), then all remaining items alphabetically by
/// the display token's symbol.
pub fn sort_by_display_priority<T: HasDisplayToken + Clone>(items: &[T]) -> Vec<T> {
    let mut sorted = items.to_vec();
    sorted.sort_by(|left, right| {
        let left_token = left.display_token();
        let right_token = right.display_token();
        left_token
            .display_priority()
            .cmp(&right_token.display_priority())
            .then_with(|| left_token.symbol().cmp(right_token.symbol()))
    });
    sorted
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

    #[cfg(test)]
    pub fn test_new(mint: impl Into<String>, outcome_index: i16) -> Self {
        Self::test_new_with_deposit(mint, outcome_index, "deposit")
    }

    #[cfg(test)]
    pub fn test_new_with_deposit(
        mint: impl Into<String>,
        outcome_index: i16,
        deposit_asset: impl Into<String>,
    ) -> Self {
        let mint = PubkeyStr::from(mint.into());
        let deposit_asset = PubkeyStr::from(deposit_asset.into());
        Self {
            id: 1,
            outcome_index,
            outcome: "Yes".to_string(),
            deposit_asset,
            deposit_symbol: "USDC".to_string(),
            mint: mint.clone(),
            name: "Outcome".to_string(),
            symbol: "YES".to_string(),
            description: None,
            decimals: 6,
            icon_url: "https://example.com/icon.png".to_string(),
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

// ─── DepositAssetPair ────────────────────────────────────────────────────────

/// A pair of deposit assets that can be traded against each other as base/quote.
///
/// Populated on `Market::deposit_asset_pairs` during wire→domain conversion;
/// one entry per unique `(base.deposit_asset, quote.deposit_asset)` combination
/// across the market's orderbook pairs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DepositAssetPair {
    /// Stable identifier of the form `"{base_pubkey}-{quote_pubkey}"`.
    pub id: String,
    pub base: DepositAsset,
    pub quote: DepositAsset,
}

impl HasDisplayToken for DepositAssetPair {
    fn display_token(&self) -> &dyn Token {
        &self.base
    }
}

// ─── GlobalDepositAsset ──────────────────────────────────────────────────────

/// A globally whitelisted deposit asset (platform-scoped, not market-bound).
///
/// Distinct from `DepositAsset`, which is bound to a specific market.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalDepositAsset {
    id: i32,
    deposit_asset: PubkeyStr,
    name: String,
    symbol: String,
    description: Option<String>,
    decimals: u16,
    icon_url: String,
    pub whitelist_index: i16,
    pub active: bool,
}

impl Token for GlobalDepositAsset {
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

impl GlobalDepositAsset {
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

impl TryFrom<GlobalDepositAssetResponse> for GlobalDepositAsset {
    type Error = TokenValidationError;

    fn try_from(source: GlobalDepositAssetResponse) -> Result<Self, Self::Error> {
        let mut errors: Vec<TokenValidationError> = Vec::new();
        let mint_str = source.mint.clone();

        let name = source.display_name.unwrap_or_else(|| {
            errors.push(TokenValidationError::MissingDisplayName(mint_str.clone()));
            String::new()
        });
        let symbol = source.symbol.unwrap_or_else(|| {
            errors.push(TokenValidationError::MissingSymbol(mint_str.clone()));
            String::new()
        });
        let icon_url = source.icon_url.unwrap_or_else(|| {
            errors.push(TokenValidationError::MissingIconUrl(mint_str.clone()));
            String::new()
        });
        let decimals = source
            .decimals
            .map(|value| value as u16)
            .unwrap_or_else(|| {
                errors.push(TokenValidationError::MissingDecimals(mint_str.clone()));
                0
            });

        if !errors.is_empty() {
            return Err(TokenValidationError::Multiple(mint_str, errors));
        }

        Ok(Self {
            id: source.id,
            deposit_asset: PubkeyStr::from(source.mint),
            name,
            symbol,
            description: source.description,
            decimals,
            icon_url,
            whitelist_index: source.whitelist_index,
            active: source.active,
        })
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
    MissingShortSymbol(String),
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
            TokenValidationError::MissingShortSymbol(m) => write!(f, "Missing short symbol: {m}"),
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
                ct_errors.push(TokenValidationError::MissingDecimals(ct_pubkey.to_string()));
                0
            });
            let ct_symbol = ct.short_symbol.unwrap_or_else(|| {
                ct_errors.push(TokenValidationError::MissingShortSymbol(
                    ct_pubkey.to_string(),
                ));
                String::new()
            });
            let ct_outcome = ct.outcome.unwrap_or_else(|| {
                ct_errors.push(TokenValidationError::MissingOutcome(ct_pubkey.to_string()));
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
                    name: ct_outcome.clone(),
                },
            );

            conditionals.push(ConditionalToken {
                id: ct.id,
                deposit_symbol: symbol.clone(),
                deposit_asset: pubkey.clone(),
                outcome_index: ct.outcome_index,
                icon_url: icon_url.clone(),
                description: ct.description,
                outcome: ct_outcome.clone(),
                mint: ct_pubkey,
                name: ct_outcome,
                symbol: ct_symbol,
                decimals: ct_decimals,
            });
        }

        if !errors.is_empty() {
            return Err(TokenValidationError::Multiple(source.deposit_asset, errors));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::market::wire::ConditionalTokenResponse;
    use chrono::Utc;

    fn minimal_deposit_asset_response() -> DepositAssetResponse {
        DepositAssetResponse {
            display_name: Some("USD Coin".to_string()),
            token_symbol: Some("USDC".to_string()),
            symbol: Some("USDC".to_string()),
            deposit_asset: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            id: 1,
            market_pubkey: "mkt".to_string(),
            vault: "vault".to_string(),
            num_outcomes: 2,
            description: None,
            icon_url: Some("https://example.com/usdc.png".to_string()),
            metadata_uri: None,
            decimals: Some(6),
            conditional_mints: vec![ConditionalTokenResponse {
                id: 10,
                outcome_index: 0,
                token_address: "cond_yes".to_string(),
                symbol: None,
                uri: None,
                outcome: Some("Yes".to_string()),
                deposit_symbol: None,
                short_symbol: Some("YES".to_string()),
                description: None,
                icon_url: None,
                metadata_uri: None,
                decimals: Some(6),
                created_at: Utc::now(),
            }],
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_validated_tokens_valid_conversion() {
        let resp = minimal_deposit_asset_response();
        let validated = ValidatedTokens::try_from(resp).unwrap();
        assert_eq!(validated.token.symbol, "USDC");
        assert_eq!(validated.conditionals.len(), 1);
        assert_eq!(validated.conditionals[0].symbol(), "YES");
    }

    #[test]
    fn test_validated_tokens_missing_symbol_fails() {
        let mut resp = minimal_deposit_asset_response();
        resp.symbol = None;
        let err = ValidatedTokens::try_from(resp).unwrap_err();
        assert!(format!("{err}").contains("symbol") || format!("{err}").contains("Symbol"));
    }

    #[test]
    fn test_validated_tokens_missing_icon_fails() {
        let mut resp = minimal_deposit_asset_response();
        resp.icon_url = None;
        let err = ValidatedTokens::try_from(resp).unwrap_err();
        assert!(format!("{err}").contains("icon") || format!("{err}").contains("Icon"));
    }

    #[test]
    fn test_conditional_token_is_usd_stablecoin() {
        // ConditionalToken::is_usd_stable_coin checks deposit_asset.
        let ct = ConditionalToken::test_new_with_deposit("mint", 0, USDC_MAINNET);
        assert!(ct.is_usd_stable_coin());
    }

    fn minimal_global_deposit_asset_response() -> GlobalDepositAssetResponse {
        GlobalDepositAssetResponse {
            id: 1,
            mint: USDC_MAINNET.to_string(),
            display_name: Some("USD Coin".to_string()),
            symbol: Some("USDC".to_string()),
            description: Some("Stablecoin".to_string()),
            icon_url: Some("https://example.com/usdc.png".to_string()),
            decimals: Some(6),
            whitelist_index: 0,
            active: true,
        }
    }

    #[test]
    fn test_global_deposit_asset_valid_conversion() {
        let response = minimal_global_deposit_asset_response();
        let asset = GlobalDepositAsset::try_from(response).unwrap();
        assert_eq!(asset.symbol(), "USDC");
        assert_eq!(asset.name(), "USD Coin");
        assert_eq!(asset.decimals(), 6);
        assert_eq!(asset.whitelist_index, 0);
        assert!(asset.active);
        assert!(asset.is_usd_stable_coin());
    }

    #[test]
    fn test_global_deposit_asset_missing_symbol_fails() {
        let mut response = minimal_global_deposit_asset_response();
        response.symbol = None;
        let error = GlobalDepositAsset::try_from(response).unwrap_err();
        let rendered = format!("{error}");
        assert!(rendered.contains("symbol") || rendered.contains("Symbol"));
    }

    #[test]
    fn test_global_deposit_asset_missing_decimals_fails() {
        let mut response = minimal_global_deposit_asset_response();
        response.decimals = None;
        let error = GlobalDepositAsset::try_from(response).unwrap_err();
        let rendered = format!("{error}");
        assert!(rendered.contains("ecimals"));
    }

    fn global_deposit_asset_with_symbol(symbol: &str) -> GlobalDepositAsset {
        let mut response = minimal_global_deposit_asset_response();
        response.symbol = Some(symbol.to_string());
        response.mint = format!("mint_{symbol}");
        GlobalDepositAsset::try_from(response).unwrap()
    }

    #[test]
    fn test_sort_by_display_priority_orders_priority_then_alpha() {
        let assets = vec![
            global_deposit_asset_with_symbol("USDC"),
            global_deposit_asset_with_symbol("SOL"),
            global_deposit_asset_with_symbol("WETH"),
            global_deposit_asset_with_symbol("AAA"),
            global_deposit_asset_with_symbol("WBTC"),
            global_deposit_asset_with_symbol("ETH"),
            global_deposit_asset_with_symbol("BTC"),
            global_deposit_asset_with_symbol("ZZZ"),
        ];

        let sorted = sort_by_display_priority(&assets);
        let symbols: Vec<&str> = sorted.iter().map(|a| a.symbol()).collect();

        assert_eq!(
            symbols,
            vec!["BTC", "WBTC", "ETH", "WETH", "SOL", "AAA", "USDC", "ZZZ"]
        );
    }

    #[test]
    fn test_global_deposit_asset_impls_token_trait() {
        let response = minimal_global_deposit_asset_response();
        let asset = GlobalDepositAsset::try_from(response).unwrap();
        let trait_object: &dyn Token = &asset;
        assert_eq!(trait_object.id(), 1);
        assert_eq!(trait_object.symbol(), "USDC");
        assert_eq!(trait_object.decimals(), 6);
        assert_eq!(trait_object.icon_url(), "https://example.com/usdc.png");
    }
}
