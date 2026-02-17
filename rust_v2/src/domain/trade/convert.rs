//! Conversions from wire types to domain types for trades.

use super::wire::{TradeResponse, WsTrade};
use super::Trade;

impl From<TradeResponse> for Trade {
    fn from(t: TradeResponse) -> Self {
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
