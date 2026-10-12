//! Conversions from wire types to domain types for trades.

use super::wire::{TradeResponse, WsTrade};
use super::Trade;
use crate::error::SdkError;
use chrono::TimeZone;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Fails on a price, size, or `executed_at` that is not a valid value.
impl TryFrom<TradeResponse> for Trade {
    type Error = SdkError;

    fn try_from(t: TradeResponse) -> Result<Self, Self::Error> {
        let invalid = |field: &str, value: &dyn std::fmt::Display| {
            SdkError::Validation(format!("trade {}: invalid {field} {value}", t.trade_id))
        };
        let timestamp = chrono::Utc
            .timestamp_millis_opt(t.executed_at)
            .single()
            .ok_or_else(|| invalid("executed_at", &t.executed_at))?;
        let price = Decimal::from_str(&t.price).map_err(|_| invalid("price", &t.price))?;
        let size = Decimal::from_str(&t.size).map_err(|_| invalid("size", &t.size))?;
        Ok(Self {
            orderbook_id: t.orderbook_id,
            trade_id: t.trade_id,
            cursor_id: Some(t.id),
            timestamp,
            price,
            size,
            side: t.side,
        })
    }
}

/// Fails when the fill has no base amount to price.
impl TryFrom<WsTrade> for Trade {
    type Error = SdkError;

    fn try_from(t: WsTrade) -> Result<Self, Self::Error> {
        let trade_id = t.trade_id();
        let price = t.price().ok_or_else(|| {
            SdkError::Validation(format!("trade {trade_id}: zero base amount has no price"))
        })?;
        Ok(Self {
            trade_id,
            price,
            orderbook_id: t.orderbook_id,
            cursor_id: None,
            timestamp: t.executed_at,
            size: t.base_amount,
            side: t.taker_side,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::trade::wire::tests::fill_fact;
    use crate::shared::{OrderBookId, Side};

    fn sample_trade_response() -> TradeResponse {
        TradeResponse {
            id: 456,
            trade_id: "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3".to_string(),
            orderbook_id: OrderBookId::from("ob_123"),
            taker_pubkey: "taker123".to_string(),
            maker_pubkey: "maker456".to_string(),
            side: Side::Bid,
            size: "10.000000".to_string(),
            price: "5.000000".to_string(),
            taker_fee_estimate: Some(Decimal::new(3250, 6)),
            maker_fee_estimate: Some(Decimal::ZERO),
            executed_at: 1740076800000,
        }
    }

    #[test]
    fn test_trade_response_conversion() {
        let resp = sample_trade_response();
        let trade = Trade::try_from(resp).unwrap();
        assert_eq!(trade.orderbook_id.as_str(), "ob_123");
        assert_eq!(trade.trade_id, "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3");
        assert_eq!(trade.cursor_id, Some(456));
        assert_eq!(trade.price, Decimal::from_str("5.000000").unwrap());
        assert_eq!(trade.size, Decimal::from_str("10.000000").unwrap());
        assert_eq!(trade.side, Side::Bid);
    }

    #[test]
    fn ws_fill_fact_converts_to_the_rest_trade_identity() {
        let ws: WsTrade = serde_json::from_value(fill_fact()).unwrap();
        let trade = Trade::try_from(ws).unwrap();
        assert_eq!(
            trade.orderbook_id.as_str(),
            "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a"
        );
        assert_eq!(trade.trade_id, "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:3");
        assert_eq!(trade.cursor_id, None);
        assert_eq!(trade.price, Decimal::new(55, 2));
        assert_eq!(trade.size, Decimal::from(4));
        assert_eq!(trade.side, Side::Ask);
    }

    #[test]
    fn invalid_trade_values_are_rejected_not_defaulted() {
        let mut resp = sample_trade_response();
        resp.price = "not-a-price".to_string();
        assert!(matches!(
            Trade::try_from(resp),
            Err(SdkError::Validation(_))
        ));
        let mut resp = sample_trade_response();
        resp.size = String::new();
        assert!(matches!(
            Trade::try_from(resp),
            Err(SdkError::Validation(_))
        ));
        let mut resp = sample_trade_response();
        resp.executed_at = i64::MAX;
        assert!(matches!(
            Trade::try_from(resp),
            Err(SdkError::Validation(_))
        ));

        let mut ws: WsTrade = serde_json::from_value(fill_fact()).unwrap();
        ws.base_amount = Decimal::ZERO;
        assert!(matches!(Trade::try_from(ws), Err(SdkError::Validation(_))));
    }
}
