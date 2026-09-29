//! Conversions from wire types to domain types for trades.

use super::wire::{TradeResponse, WsTrade};
use super::Trade;
use chrono::TimeZone;
use rust_decimal::Decimal;
use std::str::FromStr;

impl From<TradeResponse> for Trade {
    fn from(t: TradeResponse) -> Self {
        Self {
            orderbook_id: t.orderbook_id,
            trade_id: t.trade_id,
            cursor_id: Some(t.id),
            timestamp: chrono::Utc
                .timestamp_millis_opt(t.executed_at)
                .single()
                .unwrap_or_else(chrono::Utc::now),
            price: Decimal::from_str(&t.price).unwrap_or_default(),
            size: Decimal::from_str(&t.size).unwrap_or_default(),
            side: t.side,
        }
    }
}

impl From<WsTrade> for Trade {
    fn from(t: WsTrade) -> Self {
        Self {
            trade_id: t.trade_id(),
            price: t.price().unwrap_or_default(),
            orderbook_id: t.orderbook_id,
            cursor_id: None,
            timestamp: t.executed_at,
            size: t.base_amount,
            side: t.taker_side,
        }
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
        let trade: Trade = resp.into();
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
        let trade: Trade = ws.into();
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
}
