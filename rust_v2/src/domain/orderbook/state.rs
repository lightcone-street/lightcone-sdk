//! Orderbook state containers — app-owned, SDK-provided update logic.

use crate::domain::orderbook::wire::OrderBook;
use crate::shared::OrderBookId;
use rust_decimal::Decimal;
use std::collections::BTreeMap;

/// Live orderbook state that can apply snapshots and deltas.
///
/// The app owns instances of this type (e.g. inside a Dioxus `Signal`).
/// The SDK provides the update methods.
#[derive(Debug, Clone, Default)]
pub struct OrderbookSnapshot {
    pub orderbook_id: OrderBookId,
    pub seq: u32,
    bids: BTreeMap<Decimal, Decimal>,
    asks: BTreeMap<Decimal, Decimal>,
}

impl OrderbookSnapshot {
    pub fn new(orderbook_id: OrderBookId) -> Self {
        Self {
            orderbook_id,
            seq: 0,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Apply a WS orderbook message (snapshot replaces, delta merges).
    pub fn apply(&mut self, book: &OrderBook) {
        if book.is_snapshot {
            self.bids.clear();
            self.asks.clear();
        }

        self.seq = book.seq;

        for order in &book.bids {
            if order.size.is_zero() {
                self.bids.remove(&order.price);
            } else {
                self.bids.insert(order.price, order.size);
            }
        }

        for order in &book.asks {
            if order.size.is_zero() {
                self.asks.remove(&order.price);
            } else {
                self.asks.insert(order.price, order.size);
            }
        }
    }

    /// Bids sorted by price descending.
    pub fn bids(&self) -> &BTreeMap<Decimal, Decimal> {
        &self.bids
    }

    /// Asks sorted by price ascending.
    pub fn asks(&self) -> &BTreeMap<Decimal, Decimal> {
        &self.asks
    }

    /// Highest bid price.
    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().copied()
    }

    /// Lowest ask price.
    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().copied()
    }

    /// Mid price (average of best bid and best ask).
    pub fn mid_price(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / Decimal::from(2)),
            _ => None,
        }
    }

    /// Spread between best ask and best bid.
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.seq = 0;
    }
}

impl Default for OrderBookId {
    fn default() -> Self {
        OrderBookId::from("")
    }
}
