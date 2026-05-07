//! Conversion: MarketResponse → Market (TryFrom + validation).

use super::outcome;
use super::resolve_icon_urls;
use super::tokens;
use super::tokens::sort_by_display_priority;
use super::wire;
use super::{Market, Status, ValidationError};
use crate::domain::orderbook;
use crate::shared::PubkeyStr;
use rust_decimal::Decimal;
use std::collections::HashMap;

impl TryFrom<wire::MarketResponse> for Market {
    type Error = ValidationError;

    fn try_from(source: wire::MarketResponse) -> Result<Self, Self::Error> {
        let mut errors: Vec<ValidationError> = Vec::new();
        let market_pubkey = source.market_pubkey.clone();

        // Validate outcomes
        let mut outcomes: Vec<outcome::Outcome> = Vec::new();
        for o in source.outcomes.clone() {
            match o.try_into() {
                Ok(validated) => outcomes.push(validated),
                Err(err) => errors.push(ValidationError::Outcome(err)),
            }
        }

        // Validate tokens
        let mut token_metadata: HashMap<PubkeyStr, tokens::TokenMetadata> = HashMap::new();
        let mut deposit_assets: Vec<tokens::DepositAsset> = Vec::new();
        let mut conditional_tokens: Vec<tokens::ConditionalToken> = Vec::new();

        for da in source.deposit_assets.clone() {
            match tokens::ValidatedTokens::try_from(da) {
                Ok(validated) => {
                    deposit_assets.push(validated.token);
                    conditional_tokens.extend(validated.conditionals);
                    token_metadata.extend(validated.metadata);
                }
                Err(err) => errors.push(ValidationError::Token(err)),
            }
        }

        deposit_assets.sort_by(|a, b| a.symbol.cmp(&b.symbol));
        conditional_tokens.sort_by(|a, b| {
            use tokens::Token;
            a.symbol().cmp(b.symbol())
        });

        // Validate orderbooks
        let mut orderbook_pairs = Vec::new();
        for book in source.orderbooks.clone() {
            match orderbook::OrderBookPair::try_from((book, &conditional_tokens)) {
                Ok(pair) => orderbook_pairs.push(pair),
                Err(err) => errors.push(ValidationError::OrderBook(err)),
            }
        }

        let slug = source.slug.clone().unwrap_or_else(|| {
            errors.push(ValidationError::MissingSlug);
            String::new()
        });
        let name = source.market_name.clone().unwrap_or_else(|| {
            errors.push(ValidationError::MarketNameMissing);
            String::new()
        });
        let status = Status::from_str(&source.market_status).unwrap_or_else(|| {
            errors.push(ValidationError::InvalidStatus);
            Status::Pending
        });
        let description = source.description.clone().unwrap_or_else(|| {
            errors.push(ValidationError::MissingDescription);
            String::new()
        });
        let definition = source.definition.clone().unwrap_or_else(|| {
            errors.push(ValidationError::MissingDefinition);
            String::new()
        });
        let (icon_url_low, icon_url_medium, icon_url_high) = resolve_icon_urls(
            source.icon_url_low.clone(),
            source.icon_url_medium.clone(),
            source.icon_url_high.clone(),
        )
        .unwrap_or_else(|| {
            errors.push(ValidationError::MissingIconUrl);
            (String::new(), String::new(), String::new())
        });
        let (banner_image_url_low, banner_image_url_medium, banner_image_url_high) =
            resolve_icon_urls(
                source.banner_image_url_low.clone(),
                source.banner_image_url_medium.clone(),
                source.banner_image_url_high.clone(),
            )
            .unwrap_or_else(|| {
                errors.push(ValidationError::MissingBannerUrl);
                (String::new(), String::new(), String::new())
            });

        let deposit_asset_pairs = sort_by_display_priority(&derive_deposit_asset_pairs(
            &deposit_assets,
            &orderbook_pairs,
        ));

        if deposit_asset_pairs.is_empty() {
            errors.push(ValidationError::MissingDepositAssetPairs);
        }

        if !errors.is_empty() {
            return Err(ValidationError::Multiple(market_pubkey, errors));
        }

        Ok(Market {
            id: source.market_id,
            pubkey: source.market_pubkey.into(),
            volume: Decimal::ZERO,
            featured_rank: source.featured_rank,
            slug,
            name,
            status,
            created_at: source.created_at,
            activated_at: source.activated_at,
            settled_at: source.settled_at,
            resolution: source.resolution,
            description,
            definition,
            tags: source.tags.unwrap_or_default(),
            outcomes,
            icon_url_low,
            icon_url_medium,
            icon_url_high,
            banner_image_url_low,
            banner_image_url_medium,
            banner_image_url_high,
            category: source.category,
            orderbook_ids: orderbook_pairs
                .iter()
                .map(|p| p.orderbook_id.clone())
                .collect(),
            orderbook_pairs,
            deposit_assets,
            deposit_asset_pairs,
            conditional_tokens,
            token_metadata,
        })
    }
}

/// Derive unique base/quote deposit-asset pairs across the market's orderbook
/// pairs. Deduplicated by `(base_pubkey, quote_pubkey)`; orderbook pairs whose
/// base or quote deposit asset is not present in `deposit_assets` are skipped.
fn derive_deposit_asset_pairs(
    deposit_assets: &[tokens::DepositAsset],
    orderbook_pairs: &[orderbook::OrderBookPair],
) -> Vec<tokens::DepositAssetPair> {
    let mut seen: HashMap<(PubkeyStr, PubkeyStr), tokens::DepositAssetPair> = HashMap::new();

    for pair in orderbook_pairs {
        let base = deposit_assets
            .iter()
            .find(|asset| asset.deposit_asset == pair.base.deposit_asset);
        let quote = deposit_assets
            .iter()
            .find(|asset| asset.deposit_asset == pair.quote.deposit_asset);

        if let (Some(base), Some(quote)) = (base, quote) {
            let key = (base.deposit_asset.clone(), quote.deposit_asset.clone());
            seen.entry(key).or_insert_with(|| tokens::DepositAssetPair {
                id: format!("{}-{}", base.deposit_asset, quote.deposit_asset),
                base: base.clone(),
                quote: quote.clone(),
            });
        }
    }

    seen.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn minimal_market_response() -> wire::MarketResponse {
        wire::MarketResponse {
            market_name: Some("Test Market".to_string()),
            slug: Some("test-market".to_string()),
            description: Some("Description".to_string()),
            definition: Some("Definition".to_string()),
            outcomes: vec![wire::OutcomeResponse {
                index: 0,
                name: "Yes".to_string(),
                icon_url_low: Some("https://example.com/yes_low.png".to_string()),
                icon_url_medium: Some("https://example.com/yes_medium.png".to_string()),
                icon_url_high: Some("https://example.com/yes_high.png".to_string()),
            }],
            banner_image_url_low: Some("https://example.com/banner_low.png".to_string()),
            banner_image_url_medium: Some("https://example.com/banner_medium.png".to_string()),
            banner_image_url_high: Some("https://example.com/banner_high.png".to_string()),
            icon_url_low: Some("https://example.com/icon_low.png".to_string()),
            icon_url_medium: Some("https://example.com/icon_medium.png".to_string()),
            icon_url_high: Some("https://example.com/icon_high.png".to_string()),
            category: None,
            tags: None,
            featured_rank: None,
            market_pubkey: "mkt123".to_string(),
            market_id: 1,
            oracle: "oracle".to_string(),
            question_id: "q1".to_string(),
            condition_id: "c1".to_string(),
            market_status: "Active".to_string(),
            resolution: None,
            created_at: Utc::now(),
            activated_at: None,
            settled_at: None,
            deposit_assets: vec![],
            orderbooks: vec![],
        }
    }

    fn valid_market_response(
        resolution: Option<wire::MarketResolutionResponse>,
    ) -> wire::MarketResponse {
        let mut response = minimal_market_response();
        response.market_status = "Resolved".to_string();
        response.resolution = resolution;
        response.deposit_assets = vec![deposit_asset_response()];
        response.orderbooks = vec![orderbook_response()];
        response
    }

    fn deposit_asset_response() -> wire::DepositAssetResponse {
        wire::DepositAssetResponse {
            display_name: Some("USD Coin".to_string()),
            token_symbol: Some("USDC".to_string()),
            symbol: Some("USDC".to_string()),
            deposit_asset: "USDC".to_string(),
            id: 1,
            market_pubkey: "mkt123".to_string(),
            vault: "vault".to_string(),
            num_outcomes: 2,
            description: None,
            icon_url_low: Some("https://example.com/usdc_low.png".to_string()),
            icon_url_medium: Some("https://example.com/usdc_medium.png".to_string()),
            icon_url_high: Some("https://example.com/usdc_high.png".to_string()),
            metadata_uri: None,
            decimals: Some(6),
            conditional_mints: vec![
                conditional_token_response(10, 0, "yes_mint", "Yes", "YES"),
                conditional_token_response(11, 1, "no_mint", "No", "NO"),
            ],
            created_at: Utc::now(),
        }
    }

    fn conditional_token_response(
        id: i32,
        outcome_index: i16,
        token_address: &str,
        outcome: &str,
        short_symbol: &str,
    ) -> wire::ConditionalTokenResponse {
        wire::ConditionalTokenResponse {
            id,
            outcome_index,
            token_address: token_address.to_string(),
            symbol: Some(short_symbol.to_string()),
            uri: None,
            outcome: Some(outcome.to_string()),
            deposit_symbol: Some("USDC".to_string()),
            short_symbol: Some(short_symbol.to_string()),
            description: None,
            icon_url_low: Some(format!("https://example.com/{short_symbol}_low.png")),
            icon_url_medium: Some(format!("https://example.com/{short_symbol}_medium.png")),
            icon_url_high: Some(format!("https://example.com/{short_symbol}_high.png")),
            metadata_uri: None,
            decimals: Some(6),
            created_at: Utc::now(),
        }
    }

    fn orderbook_response() -> orderbook::wire::OrderbookResponse {
        orderbook::wire::OrderbookResponse {
            id: 1,
            market_pubkey: "mkt123".to_string(),
            orderbook_id: "ob_yes_no".to_string(),
            base_token: "yes_mint".to_string(),
            quote_token: "no_mint".to_string(),
            outcome_index: Some(0),
            tick_size: 1,
            total_bids: 0,
            total_asks: 0,
            last_trade_price: None,
            last_trade_time: None,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn single_winner_resolution() -> wire::MarketResolutionResponse {
        wire::MarketResolutionResponse {
            kind: wire::MarketResolutionKind::SingleWinner,
            payout_denominator: 1,
            payouts: vec![
                wire::MarketResolutionPayout {
                    outcome_index: 0,
                    payout_numerator: 0,
                },
                wire::MarketResolutionPayout {
                    outcome_index: 1,
                    payout_numerator: 1,
                },
            ],
            single_winning_outcome: Some(1),
        }
    }

    fn scalar_resolution() -> wire::MarketResolutionResponse {
        wire::MarketResolutionResponse {
            kind: wire::MarketResolutionKind::Scalar,
            payout_denominator: 10,
            payouts: vec![
                wire::MarketResolutionPayout {
                    outcome_index: 0,
                    payout_numerator: 7,
                },
                wire::MarketResolutionPayout {
                    outcome_index: 1,
                    payout_numerator: 3,
                },
            ],
            single_winning_outcome: None,
        }
    }

    #[test]
    fn market_response_resolution_helpers_handle_unresolved_scalar_and_winner() {
        let mut response = minimal_market_response();
        assert!(!response.is_resolved());
        assert_eq!(response.single_winning_outcome(), None);
        assert!(!response.has_single_winning_outcome());

        response.resolution = Some(scalar_resolution());
        assert!(response.is_resolved());
        assert_eq!(response.single_winning_outcome(), None);
        assert!(!response.has_single_winning_outcome());

        response.resolution = Some(single_winner_resolution());
        assert!(response.is_resolved());
        assert_eq!(response.single_winning_outcome(), Some(1));
        assert!(response.has_single_winning_outcome());
    }

    #[test]
    fn market_conversion_preserves_scalar_resolution() {
        let market = Market::try_from(valid_market_response(Some(scalar_resolution()))).unwrap();

        assert!(market.is_resolved());
        assert_eq!(market.single_winning_outcome(), None);
        assert!(!market.has_single_winning_outcome());
        let resolution = market.resolution.unwrap();
        assert_eq!(resolution.kind, wire::MarketResolutionKind::Scalar);
        assert_eq!(resolution.payout_denominator, 10);
        assert_eq!(resolution.payouts.len(), 2);
    }

    #[test]
    fn market_conversion_preserves_single_winner_resolution() {
        let market =
            Market::try_from(valid_market_response(Some(single_winner_resolution()))).unwrap();

        assert!(market.is_resolved());
        assert_eq!(market.single_winning_outcome(), Some(1));
        assert!(market.has_single_winning_outcome());
    }

    #[test]
    fn test_market_missing_slug_fails() {
        let mut resp = minimal_market_response();
        resp.slug = None;
        let err = Market::try_from(resp).unwrap_err();
        assert!(format!("{err}").contains("slug") || format!("{err}").contains("MissingSlug"));
    }

    #[test]
    fn test_market_missing_name_fails() {
        let mut resp = minimal_market_response();
        resp.market_name = None;
        let err = Market::try_from(resp).unwrap_err();
        assert!(
            format!("{err}").contains("name") || format!("{err}").contains("MarketNameMissing")
        );
    }

    #[test]
    fn test_market_invalid_status_fails() {
        let mut resp = minimal_market_response();
        resp.market_status = "UnknownStatus".to_string();
        let result = Market::try_from(resp);
        // Should fail with InvalidStatus in the error chain
        assert!(result.is_err());
    }

    #[test]
    fn test_market_missing_deposit_asset_pairs_fails() {
        // A response with no deposit assets and no orderbooks yields zero pairs.
        let resp = minimal_market_response();
        let err = Market::try_from(resp).unwrap_err();
        assert!(
            format!("{err}").contains("Missing deposit asset pairs"),
            "expected MissingDepositAssetPairs in error, got: {err}",
        );
    }

    #[test]
    fn deposit_asset_pairs_are_sorted_by_display_priority() {
        use tokens::Token;
        let pairs = super::derive_deposit_asset_pairs(
            &[
                deposit_asset("USDC"),
                deposit_asset("SOL"),
                deposit_asset("WETH"),
                deposit_asset("AAA"),
                deposit_asset("WBTC"),
                deposit_asset("ETH"),
                deposit_asset("BTC"),
                deposit_asset("ZZZ"),
            ],
            &[
                orderbook_pair("BTC", "USDC", 0),
                orderbook_pair("WBTC", "USDC", 0),
                orderbook_pair("ETH", "USDC", 0),
                orderbook_pair("WETH", "USDC", 0),
                orderbook_pair("SOL", "USDC", 0),
                orderbook_pair("AAA", "USDC", 0),
                orderbook_pair("ZZZ", "USDC", 0),
            ],
        );

        let sorted = sort_by_display_priority(&pairs);
        let base_symbols: Vec<&str> = sorted.iter().map(|pair| pair.base.symbol()).collect();
        assert_eq!(
            base_symbols,
            vec!["BTC", "WBTC", "ETH", "WETH", "SOL", "AAA", "ZZZ"]
        );
    }

    #[test]
    fn sort_by_display_priority_accepts_orderbook_pairs() {
        // Type-check sanity: `OrderBookPair: HasDisplayToken` lets the same sort
        // helper accept a list of pairs. Fixtures here all share a symbol so we
        // only assert the call type-checks and returns the expected count;
        // meaningful ordering is covered by
        // `deposit_asset_pairs_are_sorted_by_display_priority`.
        let pairs = vec![
            orderbook_pair("BTC", "DAI", 0),
            orderbook_pair("ETH", "DAI", 0),
        ];

        let sorted = sort_by_display_priority(&pairs);
        assert_eq!(sorted.len(), 2);
    }

    fn deposit_asset(mint: &str) -> tokens::DepositAsset {
        tokens::DepositAsset {
            id: 1,
            market_pda: PubkeyStr::from("market".to_string()),
            deposit_asset: PubkeyStr::from(mint.to_string()),
            num_outcomes: 2,
            name: mint.to_string(),
            symbol: mint.to_string(),
            description: None,
            decimals: 6,
            icon_url_low: String::new(),
            icon_url_medium: String::new(),
            icon_url_high: String::new(),
        }
    }

    fn orderbook_pair(
        base_mint: &str,
        quote_mint: &str,
        outcome_index: i16,
    ) -> orderbook::OrderBookPair {
        use crate::shared::OrderBookId;
        orderbook::OrderBookPair {
            id: outcome_index as i32,
            market_pubkey: PubkeyStr::from("market".to_string()),
            orderbook_id: OrderBookId::from(format!("ob-{outcome_index}")),
            base: tokens::ConditionalToken::test_new_with_deposit(
                format!("cond-base-{outcome_index}"),
                outcome_index,
                base_mint,
            ),
            quote: tokens::ConditionalToken::test_new_with_deposit(
                format!("cond-quote-{outcome_index}"),
                outcome_index,
                quote_mint,
            ),
            outcome_index,
            tick_size: 1,
            total_bids: 0,
            total_asks: 0,
            last_trade_price: None,
            last_trade_time: None,
            active: true,
        }
    }

    #[test]
    fn derive_deposit_asset_pairs_deduplicates_across_outcomes() {
        let base = deposit_asset("USDC");
        let quote = deposit_asset("USDT");
        let pairs = super::derive_deposit_asset_pairs(
            &[base.clone(), quote.clone()],
            &[
                orderbook_pair("USDC", "USDT", 0),
                orderbook_pair("USDC", "USDT", 1),
            ],
        );

        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].id, "USDC-USDT");
        assert_eq!(pairs[0].base, base);
        assert_eq!(pairs[0].quote, quote);
    }

    #[test]
    fn derive_deposit_asset_pairs_skips_orderbook_pairs_without_matching_deposit_asset() {
        let base = deposit_asset("USDC");
        let pairs =
            super::derive_deposit_asset_pairs(&[base], &[orderbook_pair("USDC", "MISSING", 0)]);

        assert!(pairs.is_empty());
    }

    #[test]
    fn derive_deposit_asset_pairs_returns_all_distinct_pairs() {
        let usdc = deposit_asset("USDC");
        let usdt = deposit_asset("USDT");
        let dai = deposit_asset("DAI");
        let mut pairs = super::derive_deposit_asset_pairs(
            &[usdc, usdt, dai],
            &[
                orderbook_pair("USDC", "USDT", 0),
                orderbook_pair("USDC", "DAI", 0),
            ],
        );
        pairs.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].id, "USDC-DAI");
        assert_eq!(pairs[1].id, "USDC-USDT");
    }
}
