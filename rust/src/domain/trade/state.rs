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

    /// Push a trade, keeping the buffer newest-first by execution time.
    ///
    /// Trades are deduplicated by `trade_id`, which REST and WS share, so live
    /// trades can be merged into REST history (a duplicate only fills in a
    /// missing REST `cursor_id`). Equal timestamps keep arrival order, newest
    /// first. When full, a trade older than everything retained is dropped.
    pub fn push(&mut self, trade: Trade) {
        // Treat a zero-capacity history as disabled.
        if self.max_size == 0 {
            return;
        }
        if let Some(existing) = self
            .trades
            .iter_mut()
            .find(|existing| existing.trade_id == trade.trade_id)
        {
            if existing.cursor_id.is_none() {
                existing.cursor_id = trade.cursor_id;
            }
            return;
        }
        let position = self
            .trades
            .iter()
            .position(|existing| existing.timestamp <= trade.timestamp)
            .unwrap_or(self.trades.len());
        if self.trades.len() >= self.max_size && position == self.trades.len() {
            return;
        }
        self.trades.insert(position, trade);
        if self.trades.len() > self.max_size {
            self.trades.pop_back();
        }
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
    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;

    fn at(seconds: i64) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp(1_790_000_000 + seconds, 0).unwrap()
    }

    fn make_trade(id: &str, price: i64, size: i64, seconds: i64) -> Trade {
        Trade {
            orderbook_id: OrderBookId::from("ob1"),
            trade_id: id.to_string(),
            cursor_id: None,
            timestamp: at(seconds),
            price: Decimal::from(price),
            size: Decimal::from(size),
            side: Side::Bid,
        }
    }

    fn ids(history: &TradeHistory) -> Vec<&str> {
        history
            .trades()
            .iter()
            .map(|trade| trade.trade_id.as_str())
            .collect()
    }

    #[test]
    fn test_push_adds_trades_newest_first() {
        let mut th = TradeHistory::new(OrderBookId::from("ob1"), 10);
        th.push(make_trade("t1", 50, 5, 1));
        th.push(make_trade("t2", 51, 3, 2));
        assert_eq!(th.len(), 2);
        assert_eq!(th.latest().unwrap().trade_id, "t2");
    }

    #[test]
    fn test_rolling_buffer_evicts_oldest() {
        let mut th = TradeHistory::new(OrderBookId::from("ob1"), 3);
        th.push(make_trade("t1", 50, 1, 1));
        th.push(make_trade("t2", 51, 2, 2));
        th.push(make_trade("t3", 52, 3, 3));
        th.push(make_trade("t4", 53, 4, 4));
        assert_eq!(ids(&th), ["t4", "t3", "t2"]);
    }

    #[test]
    fn test_push_orders_by_execution_time() {
        let mut th = TradeHistory::new(OrderBookId::from("ob1"), 10);
        th.push(make_trade("t3", 52, 3, 3));
        th.push(make_trade("t1", 50, 1, 1));
        th.push(make_trade("t2", 51, 2, 2));
        assert_eq!(ids(&th), ["t3", "t2", "t1"]);
    }

    #[test]
    fn test_push_equal_timestamps_keep_newest_arrival_first() {
        let mut th = TradeHistory::new(OrderBookId::from("ob1"), 10);
        th.push(make_trade("a", 50, 1, 5));
        th.push(make_trade("b", 50, 1, 5));
        assert_eq!(ids(&th), ["b", "a"]);
    }

    #[test]
    fn test_push_drops_older_trade_when_full() {
        let mut th = TradeHistory::new(OrderBookId::from("ob1"), 3);
        th.push(make_trade("t5", 54, 5, 5));
        th.push(make_trade("t4", 53, 4, 4));
        th.push(make_trade("t3", 52, 3, 3));
        th.push(make_trade("t2", 51, 2, 2));
        assert_eq!(ids(&th), ["t5", "t4", "t3"]);
    }

    #[test]
    fn test_push_deduplicates_rest_and_ws_trades() {
        let mut th = TradeHistory::new(OrderBookId::from("ob1"), 10);
        th.push(make_trade("exec:0:1", 50, 1, 1));
        let mut rest = make_trade("exec:0:1", 50, 1, 1);
        rest.cursor_id = Some(77);
        th.push(rest);
        assert_eq!(th.len(), 1);
        assert_eq!(th.latest().unwrap().cursor_id, Some(77));
    }

    #[test]
    fn test_replace_clears_and_fills() {
        let mut th = TradeHistory::new(OrderBookId::from("ob1"), 10);
        th.push(make_trade("t1", 50, 1, 1));
        th.replace(vec![make_trade("a", 49, 1, 2), make_trade("b", 50, 2, 1)]);
        assert_eq!(th.len(), 2);
        // replace uses push_back, so first vec element is at front (latest)
        assert_eq!(th.latest().unwrap().trade_id, "a");
    }

    #[test]
    fn test_zero_capacity_is_disabled() {
        let mut th = TradeHistory::new(OrderBookId::from("ob1"), 0);
        th.push(make_trade("t1", 50, 1, 1));
        assert!(th.is_empty());
    }
}
