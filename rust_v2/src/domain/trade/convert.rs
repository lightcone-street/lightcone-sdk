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
            trade_id: t.id.to_string(),
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
            orderbook_id: t.orderbook_id,
            trade_id: t.trade_id,
            timestamp: t.timestamp,
            price: t.price,
            size: t.size,
            side: t.side,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{OrderBookId, Side};
    use chrono::Utc;

    fn sample_trade_response() -> TradeResponse {
        TradeResponse {
            id: 456,
            orderbook_id: OrderBookId::from("ob_123"),
            taker_pubkey: "taker123".to_string(),
            maker_pubkey: "maker456".to_string(),
            side: Side::Bid,
            size: "10.000000".to_string(),
            price: "5.000000".to_string(),
            taker_fee: Some("0.003250".to_string()),
            maker_fee: None,
            executed_at: 1740076800000,
        }
    }

    fn sample_ws_trade() -> WsTrade {
        WsTrade {
            orderbook_id: OrderBookId::from("ob_789"),
            trade_id: "ws_trade_999".to_string(),
            timestamp: Utc::now(),
            price: Decimal::new(75, 1),
            size: Decimal::new(5, 0),
            side: Side::Ask,
        }
    }

    #[test]
    fn test_trade_response_conversion() {
        let resp = sample_trade_response();
        let trade: Trade = resp.into();
        assert_eq!(trade.orderbook_id.as_str(), "ob_123");
        assert_eq!(trade.trade_id, "456");
        assert_eq!(trade.price, Decimal::from_str("5.000000").unwrap());
        assert_eq!(trade.size, Decimal::from_str("10.000000").unwrap());
        assert_eq!(trade.side, Side::Bid);
    }

    #[test]
    fn test_ws_trade_conversion() {
        let ws = sample_ws_trade();
        let trade: Trade = ws.into();
        assert_eq!(trade.orderbook_id.as_str(), "ob_789");
        assert_eq!(trade.trade_id, "ws_trade_999");
        assert_eq!(trade.price, Decimal::new(75, 1));
        assert_eq!(trade.size, Decimal::new(5, 0));
        assert_eq!(trade.side, Side::Ask);
    }
}
