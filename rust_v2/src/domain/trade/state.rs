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
