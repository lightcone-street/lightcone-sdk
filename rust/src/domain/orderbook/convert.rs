//! Conversion: OrderbookResponse → OrderBookPair (TryFrom + validation).

use super::wire;
use super::{OrderBookPair, OrderBookValidationError};
use crate::domain::market::tokens::{self, Token};
use crate::shared::PubkeyStr;

impl TryFrom<(wire::OrderbookResponse, &Vec<tokens::ConditionalToken>)> for OrderBookPair {
    type Error = OrderBookValidationError;

    fn try_from(
        value: (wire::OrderbookResponse, &Vec<tokens::ConditionalToken>),
    ) -> Result<Self, Self::Error> {
        let (source, tokens) = value;
        let mut errors: Vec<OrderBookValidationError> = Vec::new();

        let base_mint: PubkeyStr = source.base_token.clone().into();
        let base = tokens.iter().find(|t| t.pubkey() == &base_mint);
        if base.is_none() {
            errors.push(OrderBookValidationError::BaseTokenNotFound(format!(
                "orderbook: {}, base: {}",
                source.orderbook_id, base_mint
            )));
        }

        let quote_mint: PubkeyStr = source.quote_token.clone().into();
        let quote = tokens.iter().find(|t| t.pubkey() == &quote_mint);
        if quote.is_none() {
            errors.push(OrderBookValidationError::QuoteTokenNotFound(format!(
                "orderbook: {}, quote: {}",
                source.orderbook_id, quote_mint
            )));
        }

        if !errors.is_empty() {
            return Err(OrderBookValidationError::Multiple(
                source.orderbook_id.clone(),
                errors,
            ));
        }

        let base = base.unwrap().clone();
        let quote = quote.unwrap().clone();

        Ok(OrderBookPair {
            id: source.id,
            outcome_index: source.outcome_index.unwrap_or(base.outcome_index),
            market_pubkey: source.market_pubkey.into(),
            orderbook_id: source.orderbook_id.into(),
            base,
            quote,
            tick_size: source.tick_size,
            total_bids: source.total_bids,
            total_asks: source.total_asks,
            last_trade_price: source.last_trade_price,
            last_trade_time: source.last_trade_time,
            active: source.active,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::market::tokens::ConditionalToken;
    use chrono::Utc;

    fn make_conditional_token(mint: &str, outcome_index: i16) -> ConditionalToken {
        ConditionalToken::test_new(mint, outcome_index)
    }

    #[test]
    fn test_orderbook_pair_valid_conversion() {
        let base_token = "base_mint_abc";
        let quote_token = "quote_mint_xyz";
        let tokens = vec![
            make_conditional_token(base_token, 0),
            make_conditional_token(quote_token, 1),
        ];
        let wire = wire::OrderbookResponse {
            id: 42,
            market_pubkey: "mkt".to_string(),
            orderbook_id: "ob_1".to_string(),
            base_token: base_token.to_string(),
            quote_token: quote_token.to_string(),
            outcome_index: Some(0),
            tick_size: 1,
            total_bids: 10,
            total_asks: 5,
            last_trade_price: None,
            last_trade_time: None,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let pair = OrderBookPair::try_from((wire, &tokens)).unwrap();
        assert_eq!(pair.orderbook_id.as_str(), "ob_1");
        assert_eq!(pair.base.pubkey().as_str(), base_token);
        assert_eq!(pair.quote.pubkey().as_str(), quote_token);
    }

    #[test]
    fn test_orderbook_pair_base_token_not_found() {
        let tokens = vec![make_conditional_token("other_mint", 0)];
        let wire = wire::OrderbookResponse {
            id: 1,
            market_pubkey: "mkt".to_string(),
            orderbook_id: "ob".to_string(),
            base_token: "missing_base".to_string(),
            quote_token: "other_mint".to_string(),
            outcome_index: None,
            tick_size: 1,
            total_bids: 0,
            total_asks: 0,
            last_trade_price: None,
            last_trade_time: None,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let err = OrderBookPair::try_from((wire, &tokens)).unwrap_err();
        assert!(format!("{err}").contains("BaseTokenNotFound") || format!("{err}").contains("base"));
    }

    #[test]
    fn test_orderbook_pair_quote_token_not_found() {
        let tokens = vec![make_conditional_token("base_ok", 0)];
        let wire = wire::OrderbookResponse {
            id: 1,
            market_pubkey: "mkt".to_string(),
            orderbook_id: "ob".to_string(),
            base_token: "base_ok".to_string(),
            quote_token: "missing_quote".to_string(),
            outcome_index: None,
            tick_size: 1,
            total_bids: 0,
            total_asks: 0,
            last_trade_price: None,
            last_trade_time: None,
            active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let err = OrderBookPair::try_from((wire, &tokens)).unwrap_err();
        assert!(format!("{err}").contains("QuoteTokenNotFound") || format!("{err}").contains("quote"));
    }
}
