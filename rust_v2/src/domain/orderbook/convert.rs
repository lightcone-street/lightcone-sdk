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
