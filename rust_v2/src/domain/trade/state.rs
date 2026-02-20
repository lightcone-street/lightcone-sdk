//! Trade state containers — app-owned, SDK-provided update logic.

use super::Trade;
use crate::shared::OrderBookId;
use std::collections::VecDeque;

/// Rolling trade history buffer for an orderbook.
///
/// The app owns instances of this type. The SDK provides update methods.
#[derive(Debug, Clone)]
pub struct TradeHistory {
    pub orderbook_id: OrderBookId,
    trades: VecDeque<Trade>,
    max_size: usize,
}

impl TradeHistory {
    pub fn new(orderbook_id: OrderBookId, max_size: usize) -> Self {
        Self {
            orderbook_id,
            trades: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Push a new trade, evicting the oldest if at capacity.
    pub fn push(&mut self, trade: Trade) {
        if self.trades.len() >= self.max_size {
            self.trades.pop_back();
        }
        self.trades.push_front(trade);
    }

    /// Replace all trades (e.g. from a REST fetch).
    pub fn replace(&mut self, trades: Vec<Trade>) {
        self.trades.clear();
        for trade in trades.into_iter().take(self.max_size) {
            self.trades.push_back(trade);
        }
    }

    pub fn trades(&self) -> &VecDeque<Trade> {
        &self.trades
    }

    pub fn latest(&self) -> Option<&Trade> {
        self.trades.front()
    }

    pub fn clear(&mut self) {
        self.trades.clear();
    }

    pub fn len(&self) -> usize {
        self.trades.len()
    }

    pub fn is_empty(&self) -> bool {
        self.trades.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{OrderBookId, Side};
    use chrono::Utc;
    use rust_decimal::Decimal;

    fn make_trade(id: &str, price: f64, size: f64) -> Trade {
        Trade {
            orderbook_id: OrderBookId::from("ob1"),
            trade_id: id.to_string(),
            timestamp: Utc::now(),
            price: Decimal::try_from(price).unwrap(),
            size: Decimal::try_from(size).unwrap(),
            side: Side::Bid,
        }
    }

    #[test]
    fn test_push_adds_trades() {
        let mut th = TradeHistory::new(OrderBookId::from("ob1"), 10);
        th.push(make_trade("t1", 50.0, 5.0));
        th.push(make_trade("t2", 51.0, 3.0));
        assert_eq!(th.len(), 2);
        assert_eq!(th.latest().unwrap().trade_id, "t2");
    }

    #[test]
    fn test_rolling_buffer_evicts_oldest() {
        let mut th = TradeHistory::new(OrderBookId::from("ob1"), 3);
        th.push(make_trade("t1", 50.0, 1.0));
        th.push(make_trade("t2", 51.0, 2.0));
        th.push(make_trade("t3", 52.0, 3.0));
        assert_eq!(th.len(), 3);
        th.push(make_trade("t4", 53.0, 4.0));
        assert_eq!(th.len(), 3);
        let ids: Vec<_> = th.trades().iter().map(|t| t.trade_id.as_str()).collect();
        assert_eq!(ids, ["t4", "t3", "t2"]);
    }

    #[test]
    fn test_replace_clears_and_fills() {
        let mut th = TradeHistory::new(OrderBookId::from("ob1"), 10);
        th.push(make_trade("t1", 50.0, 1.0));
        th.replace(vec![
            make_trade("a", 49.0, 1.0),
            make_trade("b", 50.0, 2.0),
        ]);
        assert_eq!(th.len(), 2);
        // replace uses push_back, so first vec element is at front (latest)
        assert_eq!(th.latest().unwrap().trade_id, "a");
    }
}
