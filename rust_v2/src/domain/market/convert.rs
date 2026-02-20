//! Conversion: MarketResponse → Market (TryFrom + validation).

use super::outcome;
use super::tokens;
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
        let icon_url = source.icon_url.clone().unwrap_or_else(|| {
            errors.push(ValidationError::MissingThumbnailImage);
            String::new()
        });
        let banner_image_url = source.banner_image_url.clone().unwrap_or_else(|| {
            errors.push(ValidationError::MissingBannerImage);
            String::new()
        });

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
            winning_outcome: source.winning_outcome,
            description,
            definition,
            tags: source.tags.unwrap_or_default(),
            outcomes,
            icon_url,
            banner_image_url,
            category: source.category,
            orderbook_ids: orderbook_pairs.iter().map(|p| p.orderbook_id.clone()).collect(),
            orderbook_pairs,
            deposit_assets,
            conditional_tokens,
            token_metadata,
        })
    }
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
                icon_url: Some("https://example.com/yes.png".to_string()),
            }],
            banner_image_url: Some("https://example.com/banner.png".to_string()),
            icon_url: Some("https://example.com/icon.png".to_string()),
            category: None,
            tags: None,
            featured_rank: None,
            market_pubkey: "mkt123".to_string(),
            market_id: 1,
            oracle: "oracle".to_string(),
            question_id: "q1".to_string(),
            condition_id: "c1".to_string(),
            market_status: "Active".to_string(),
            winning_outcome: None,
            has_winning_outcome: false,
            created_at: Utc::now(),
            activated_at: None,
            settled_at: None,
            deposit_assets: vec![],
            orderbooks: vec![],
        }
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
}
